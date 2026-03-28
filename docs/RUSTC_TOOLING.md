# Rustc Tooling Notes

typelude のデバッグ・診断・可視化・IDE 支援を強化するために、rustc 側で利用できる機能を整理したメモ。

調査時点のローカル環境:

- stable: `rustc 1.94.1 (2026-03-25)`
- nightly: `rustc 1.96.0-nightly (2026-03-25)`

このドキュメントは「今すぐ使えるもの」と「nightly / rustc_private を前提に使えるもの」を分けて記録し、末尾に実測結果を追記する。

## 1. typelude から見た主な用途

rustc 側の機能は、用途ごとに次の4系統に分かれる。

1. 診断を構造化して取り込む
2. trait 解決や query の挙動を観測する
3. compile time のコストを測る
4. compiler 内部 API から proof tree や obligation を直接読む

typelude にとって特に重要なのは次。

- trait 未実装エラーを typelude の語彙で説明する
- helper trait / blanket impl によるノイズを減らす
- trait solving の爆発点を見つける
- obligation chain や proof tree を可視化する
- LSP / MCP で中間型や失敗理由を返せるようにする

## 2. stable でそのまま使えるもの

### 2.1 JSON diagnostics

`rustc` は `--error-format=json` と `--json` で診断を JSON として出力できる。

主な用途:

- E0277 などの trait bound error を機械的に収集
- span, note, help, suggestion を構造化して保持
- typelude-aware diagnostics への再変換
- LSP / MCP / test golden の入力

使うフラグ:

```bash
cargo check --message-format=json
rustc --error-format=json --json=diagnostic-short,artifacts,future-incompat,timings
```

着目点:

- `rendered` だけでなく span / children / code を使う
- `timings` を有効にするとコンパイル区間の開始・終了も拾える
- `cargo check --message-format=json` も実用上かなり有効

typelude での利用候補:

- `trait bound is not satisfied` を typelude の capability failure に再解釈
- 失敗箇所から `Eval`, `EApp`, helper trait への対応付け
- compile-fail / golden diagnostics test

### 2.2 diagnostic attributes

trait 未実装時のメッセージ改善には `#[diagnostic::on_unimplemented]` が使える。
また、診断の recommendation noise を抑えるために `#[diagnostic::do_not_recommend]` が使える。

`on_unimplemented` の主なキー:

- `message`
- `label`
- `note`

例:

```rust
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be used as a typelude Bool",
    label = "expected a typelude Bool here",
    note = "Cond must evaluate to True or False"
)]
pub trait IsBool {}
```

`do_not_recommend` は blanket impl や内部 helper impl の露出を抑えるのに有効。

typelude での利用候補:

- `IsBool`, `IsList`, `Callable`, `LookupKey`, `SupportsAdd` などの capability trait
- 利用者向けに見せたくない内部 dispatch impl
- blanket impl による誤誘導の抑制

### 2.3 `RUSTC_LOG`

rustc の内部 tracing は `RUSTC_LOG` で有効化できる。
trait 解決のログを見たい場合、この経路が本筋。

試す候補:

```bash
RUSTC_LOG=rustc_trait_selection=debug cargo check
RUSTC_LOG=rustc_trait_selection::traits::select=debug cargo check
RUSTC_LOG=rustc_trait_selection::solve=debug cargo check
RUSTC_LOG=rustc_trait_selection::traits::fulfill=debug cargo check
RUSTC_LOG_ENTRY_EXIT=1 cargo check
```

注意:

- 出力量が非常に多い
- どの target がどの粒度で出るかは compiler 内部実装に依存する
- `debug` / `trace` は rustc の build 条件や内部 instrumentation に依る

typelude での利用候補:

- 調査時の ad-hoc な trait solving トレース
- 将来の `typelude-explain` のログ入力
- helper trait dispatch の爆発箇所の特定

#### 実用上の重要ポイント

`RUSTC_LOG` は module filter だけでなく、出力形式や span 表示も環境変数で変えられる。

主に使うもの:

- `RUSTC_LOG`
- `RUSTC_LOG_COLOR`
- `RUSTC_LOG_ENTRY_EXIT`
- `RUSTC_LOG_THREAD_IDS`
- `RUSTC_LOG_BACKTRACE`
- `RUSTC_LOG_LINES`
- `RUSTC_LOG_FORMAT_JSON`
- `RUSTC_LOG_OUTPUT_TARGET`

実測で確認できたこと:

- `RUSTC_LOG_FORMAT_JSON=1`
  - 構造化ログとして有効
  - `timestamp`, `level`, `target`, `fields`, `span`, `spans` が出る
  - 後段のパーサ実装にはかなり有利
- `RUSTC_LOG_LINES=1`
  - span の入れ子が ASCII/box-drawing で見える
  - 人間が手で読むときにはかなり便利
- `RUSTC_LOG_THREAD_IDS=1`
  - 行頭に thread id が出る
  - 並列実行時の解析に有効
- `RUSTC_LOG_OUTPUT_TARGET=<path>`
  - 相対パス指定でファイル出力できる
  - stderr を diagnostics 専用に残しつつ、ログを別ファイルに退避できる

逆に注意点:

- official nightly では static max level が `info` のことがある
- その場合 `debug` / `trace` ログは出ない
- docs にある function/query/span filter は内部的に `trace` を要求することが多く、この build では警告だけ出て実質効かない

つまり、実運用の基準はまず `module=info` で考えるべき。

#### typelude 向けプリセット候補

人間向け:

```bash
RUSTC_LOG=rustc_trait_selection=info \
RUSTC_LOG_LINES=1 \
cargo +nightly check
```

構造化収集向け:

```bash
RUSTC_LOG=rustc_trait_selection=info \
RUSTC_LOG_FORMAT_JSON=1 \
RUSTC_LOG_OUTPUT_TARGET=rustc-trait.log \
cargo +nightly check
```

normalize に絞る:

```bash
RUSTC_LOG=rustc_trait_selection::traits::normalize=info \
cargo +nightly check
```

wf + normalize に絞る:

```bash
RUSTC_LOG=rustc_trait_selection::traits::wf=info,rustc_trait_selection::traits::normalize=info \
RUSTC_LOG_LINES=1 \
cargo +nightly check
```

## 3. nightly で使えるもの

nightly が入ると `rustc -Z help` で一覧を確認できる。
この環境では nightly 未導入のため、以下は公式 docs ベースの整理。

### 3.1 self-profile

`-Z self-profile` は rustc 内部の profiler。
`measureme` 系ツールで flamegraph や timeline に変換できる。

主な用途:

- compile time のホットスポット特定
- query provider の偏り確認
- query cache hit / miss の観測
- typelude の encoding や helper trait 設計の比較

併用候補:

```bash
cargo +nightly rustc -Z self-profile
cargo +nightly rustc -Z self-profile -Z self-profile-events=query-provider,generic-activity,query-cache-hit
```

typelude での利用候補:

- type flamegraph
- obligation flamegraph
- encoding / implementation pattern の compile-time 比較

### 3.2 dep graph 出力

`-Z dump-dep-graph` で query / dep graph を出力できる。

主な用途:

- 依存関係の可視化
- 特定部分の変更波及の分析
- trait solving の結果そのものではないが、query レベルの構造を見る補助になる

typelude での利用候補:

- helper trait 群がどの query を増やしているかの観測
- 増分コンパイル時の影響分析

### 3.3 HIR / MIR のダンプ

マクロ展開や内部 IR の形を見たい場合に有効。

主な候補:

- `-Z unpretty=hir`
- `-Z unpretty=hir-tree`
- `-Z unpretty=expanded,identified`
- `-Z dump-mir`
- `-Z dump-mir-graphviz`
- `-Z dump-mir-dataflow`

typelude での利用候補:

- `program!` / `ty_fn!` 展開結果の検査
- span とマクロ展開後コードの対応確認
- マクロ起因の diagnostics 改善

### 3.4 その他候補

今後確認したい `-Z` 系:

- `-Z next-solver`
- `-Z macro-backtrace`
- `-Z time-passes`
- `-Z query-dep-graph`
- `-Z print-type-sizes`

ただし typelude で直接効くのは、まず `self-profile`, `dump-dep-graph`, `unpretty`, `dump-mir` の方が優先度は高い。

## 4. rustc_private で直接取れるもの

本格的な可視化や explain をやるなら `rustc_private` が本命。
`rustc_driver`, `rustc_interface`, `TyCtxt` 周辺を使って、外部ツールとして compiler にフックする。

### 4.1 rustc_driver

`rustc_driver::Callbacks` でコンパイル各段階に割り込める。

代表的なフック:

- `after_crate_root_parsing`
- `after_expansion`
- `after_analysis`

typelude での利用候補:

- 専用 CLI
- 専用 LSP / MCP backend
- source span と trait obligation の突合
- compiler 内部の query 実行結果の収集

### 4.2 trait solver inspection

nightly docs には `rustc_trait_selection::solve::inspect` が公開されており、
solver proof tree を解析するための infrastructure が見える。

これが狙えると、単なるログではなく次のような構造化データが取れる可能性がある。

- root goal
- candidate の検討順
- nested goals
- success / ambiguity / no solution
- obligation chain

typelude での利用候補:

- trait resolution graph viewer
- obligation chain viewer
- `why did this evaluate?`
- `why not?`
- helper trait dispatch の可視化

### 4.3 注意点

`rustc_private` を使う場合の注意:

- nightly 前提
- `rustc-dev` と `llvm-tools` の導入が必要
- API 安定性はない
- toolchain バージョン固定がほぼ必須

その代わり、typelude-aware な explain / graph / diagnostics を作るには最短経路になりうる。

## 5. trait 未実装 diagnostics 強化の方向

typelude に最も効くのは、trait failure を「Rust の低レベル事情」ではなく
「typelude の意味論上の失敗」として出し直すこと。

優先候補:

- `IsBool`
- `IsList`
- `Callable`
- `LookupKey<K>`
- `SupportsAdd`
- `SupportsSub`
- `SupportsCompare`

設計方針:

1. 公開 capability trait には `#[diagnostic::on_unimplemented]` を付ける
2. blanket impl / helper impl の一部には `#[diagnostic::do_not_recommend]` を付ける
3. JSON diagnostics を後段で ingest し、typelude 用の補足 explanation を重ねる

これにより、次のような改善が見込める。

- `EIf` の条件が bool でない時の説明
- `EGet` の key 不一致や capability 不足の説明
- `EApp` で callable でない型が来た時の説明
- helper trait dispatch failure を利用者向けメッセージに圧縮

## 6. 優先度付きロードマップ

### Phase 1: すぐ着手できる

- `--error-format=json` / `cargo --message-format=json` の取り込み
- capability trait への `#[diagnostic::on_unimplemented]`
- blanket impl への `#[diagnostic::do_not_recommend]`
- `RUSTC_LOG` の trait solver プリセットを整備

### Phase 2: nightly 観測強化

- `-Z self-profile` を使った compile-time profiling
- `-Z dump-dep-graph` の実験
- `-Z unpretty`, `-Z dump-mir` による macros 観測

### Phase 3: 専用ツール化

- `rustc_driver` ベースの `typelude-rustc-driver`
- trait solver inspection による proof tree / obligation graph 抽出
- LSP / MCP 向け explain backend

## 7. typelude 向けの具体的な成果物候補

この調査から自然に出てくる crate / ツール候補:

- `typelude-diagnostics`
  - rustc JSON diagnostics の再解釈
- `typelude-prof`
  - self-profile と typelude trace の結合
- `typelude-explain`
  - trait failure の explanation 層
- `typelude-rustc-driver`
  - rustc_private ベースの解析 CLI
- `typelude-solver-view`
  - goal / candidate / obligation のグラフ表示

## 8. 参考

- rustc book: command-line arguments
- rustc book: JSON output
- Rust Reference: diagnostic attributes
- rustc-dev-guide: tracing
- rustc-dev-guide: trait resolution
- rustc-dev-guide: trait solving (new)
- rustc-dev-guide: external rustc drivers
- rustc-dev-guide: HIR debugging
- rustc-dev-guide: MIR debugging
- nightly rustdoc: `rustc_trait_selection::solve`
- nightly rustdoc: `rustc_trait_selection::solve::inspect`

## 9. 実測メモ

2026-03-27 に最小サンプルで実際に試した結果の要約。

### 9.1 かなり有望

- `#[diagnostic::on_unimplemented]`
  - E0277 の先頭メッセージを typelude 向けに差し替えられる
  - JSON diagnostics にも素直に反映される
  - ただし rustc 側の help / note は引き続き付くので、後段整形は必要
- `RUSTC_LOG=rustc_trait_selection=info`
  - `wf`, `normalize`, `obligation` の情報が実際に出る
  - `debug` はこの build では静的に無効だったが、`info` でも十分に有益
  - trait solving の流れを把握する最初の入口として使える
- `-Z unpretty=hir`
  - 展開後の構造を人間が読める形で取れる
  - macro 展開や HIR レベルの確認に有効
- `-Z print-type-sizes`
  - type size の出力がそのまま取れる
  - typelude の表現膨張を観察するのに有効
- `-Z time-passes`
  - human-readable な pass timing がそのまま取れる
  - `time`, `rss`, `total` が見えるので粗いホットスポット把握に使える

### 9.2 raw data だが有望

- `-Z self-profile`
  - `*.mm_profdata` が生成された
  - raw データとしては有用だが、そのままでは読みにくい
  - 別途 summarizer / viewer が必要
- `-Z dump-mir=all`
  - `mir_dump/` に `.mir` と `.dot` が大量に出る
  - かなり有効
  - MIR / NLL / region graph の観測に向く
- `-Z query-dep-graph` + `-Z dump-dep-graph`
  - フラグ自体は通る
  - ただし今回の最小サンプルでは期待したファイルの生成を確認しきれなかった
  - 追加の出力条件や環境変数の確認が必要

### 9.3 期待より弱かったもの

- `RUSTC_LOG=rustc_trait_selection::traits::select=debug`
  - `debug` は静的最大レベル制限で出なかった
  - この build では `info` を使う方が現実的
- `-Z next-solver`
  - 最小例では診断の見た目に差はなかった
  - solver 差の観測には有効だが、単独では情報が出ない
  - 実運用では trait resolving ログや proof tree 系と組み合わせる前提

### 9.4 実務的な評価

最初に投資する価値が高い順は次の通り。

1. `#[diagnostic::on_unimplemented]` を typelude の capability trait に広げる
2. `RUSTC_LOG=rustc_trait_selection=info` を解析用プリセットにする
3. `-Z dump-mir` を macro / lowering / meaning graph の観測に使う
4. `-Z time-passes` と `-Z print-type-sizes` を軽量 profiler として使う
5. `-Z self-profile` を後段の analyzer 前提で採用する

この結果から、rustc 側は「そのまま読むログ」より「typelude 用に再構造化する入力源」として使うのが妥当だと分かる。
