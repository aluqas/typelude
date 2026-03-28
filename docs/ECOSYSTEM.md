# Ecosystem Ideas

typelude の開発体験を強化するためのエコシステム案を整理したメモ。

目的は単なるデバッグ補助ではなく、次の3つを同時に満たすことにある。

1. 型レベル計算を観測できるようにする
2. 型プログラムの失敗やコストを説明できるようにする
3. typelude 固有の意味論を、利用者にとって理解しやすい形で表現できるようにする

## 1. 基本的な整理

この領域は大きく3層に分けて考えると整理しやすい。

### 1.1 rustc substrate

rustc 側の出力や内部 API を扱う層。

- diagnostics JSON
- `RUSTC_LOG`
- `-Z self-profile`
- `-Z dump-dep-graph`
- `-Z unpretty`
- `-Z dump-mir`
- `rustc_private`
- trait solver proof tree / inspect API

この層は typelude 専用ではなく、型プログラミング一般にも再利用しやすい。

### 1.2 type-tooling core

rustc substrate から得た情報を、汎用的な観測・可視化・分析機能に変換する層。

- trace model
- graph model
- profiler
- type diff
- failing slice minimizer
- diagnostics formatter
- trace formatter

この層は typelude に最適化しすぎず、他の型レベルライブラリにも流用できることが望ましい。

### 1.3 typelude adapter

typelude の概念や記法を知っている層。

- `EIf`, `EWhile`, `EApp` の意味論に基づく explanation
- helper trait を畳んだ表示
- `Array<...>` を list 記法へ簡約
- typenum や内部表現の pretty print
- VM state の専用表示
- typelude-specific diagnostics
- typelude-specific guardrails

この層が typelude の UX を直接決める。

## 2. 2つの大きな方向性

今回の議論で見えてきた方向性は大きく2つある。

### 2.1 typelude-specific な支援

typelude の意味論を知っているからこそできる支援。

- 型と trait だけでは出せないエラーメッセージ
- capability failure の説明
- helper trait / blanket impl の内部事情を隠す
- `EIf`, `EWhile`, `EGet`, `EMap` などの意味論に沿った説明
- typelude notation での pretty print
- typelude 向け guardrail

これは「一般的な Rust ツール」ではなく、typelude 専用の体験層。

### 2.2 型プログラミング一般に寄せたツール

typelude を越えて再利用できる支援。

- trait solving のトレース
- obligation graph 可視化
- compile-time profiler
- rustc diagnostics/log の整形
- proof tree の可視化
- type diff
- failing slice minimization

これは typelude を対象にしつつ、汎用ツールに育てられる可能性がある。

## 3. 中核となる観測対象

エコシステムを組み立てるうえで、まず何を観測するかを明確にする必要がある。

主な観測対象:

- expression
- semantic step
- trait goal
- candidate impl
- nested obligation
- alias expansion
- normalization
- recursion
- branch choice
- type size
- cache hit / miss
- physical time

typelude では、これらを単なる compiler event としてではなく、
意味論上の単位に持ち上げて扱えると強い。

## 4. トレースと整形

trait 解決や型評価の過程を、単なるログではなく構造化された trace として扱う。

### 4.1 欲しいイベントモデル

候補:

- `GoalStarted`
- `GoalResolved`
- `CandidateTried`
- `CandidateSelected`
- `CandidateRejected`
- `NestedObligation`
- `AliasExpanded`
- `Normalization`
- `RecursionEntered`
- `RecursionExited`
- `BranchChosen`
- `CacheHit`
- `CacheMiss`
- `ErrorRaised`

このような共通イベント形式があると、観測系の土台を共有できる。

### 4.2 欲しい表示モード

- raw trace
- collapsed trace
- semantic trace
- diff trace
- hot path only

typelude では `collapsed trace` と `semantic trace` の価値が特に高い。

## 5. 意味論グラフ

trait 解決のグラフ構造を、そのまま solver graph として見るだけでなく、
typelude の意味論に沿って semantic graph として見直せるとよい。

### 5.1 solver graph

rustc に近い低レベル表現。

- goal
- candidate impl
- nested obligation
- success / failure / ambiguity

### 5.2 semantic graph

typelude の計算モデルに近い高レベル表現。

- `EIf`
- `EWhile`
- `EApp`
- `EAdd`
- `EGet`
- helper trait dispatch
- alias expansion
- primitive composition

理想は、solver graph から semantic graph への写像を持つこと。

たとえば:

- ある obligation が `EIf` の条件評価に対応する
- ある impl 選択が `True` 分岐の dispatch に対応する
- ある再帰 obligation が `EWhile` の継続に対応する

この写像があると、「trait 解決グラフで意味論を示す」が可能になる。

## 6. プロファイリング

typelude では、物理時間だけでなく型計算の構造コストを測る必要がある。

### 6.1 欲しいメトリクス

- ステップ回数
- trait obligation 数
- candidate 試行数
- 分岐回数
- 再帰の深さ
- 最大 goal 深さ
- 型の大きさ
- 型サイズの増加率
- 再評価回数
- normalization 回数
- cache hit rate
- 失敗 obligation 数
- 物理時間

### 6.2 欲しいビュー

- type flamegraph
- obligation flamegraph
- recursion tree
- branch profile
- type growth chart
- re-evaluation report
- hot helper traits

### 6.3 意味論プロファイラ

typelude では、compiler-level profiler に加えて semantic profiler がほしい。

例:

- `EWhile` が何回回ったか
- `EMap` が何要素に適用されたか
- `EGet` が何段探索したか
- `EIf` がどちらに倒れたか
- `OpWhile` がどこで停止したか

これは typelude 専用の価値を持つ。

## 7. 値側との接続

観測や診断を実用に引き戻すために、型と値の接続も重要。

### 7.1 Reify / Reflect / Render / Diagnose

整理すると、次の4種がある。

- `Reify`
  - 型を値に落とす
- `Reflect`
  - 値を型に持ち上げる
- `Render`
  - 型を文字列や構造に表示する
- `Diagnose`
  - 失敗を説明する

### 7.2 Render の強化

特に有用そうなもの:

- pretty printer
- compact printer
- debug tree printer
- JSON 出力
- Graphviz / Mermaid 出力
- list / tree / map / VM state の専用表示

typelude では、表記変換だけでもかなり見やすさが変わる。

例:

- `Array<Head, Tail>` を list 記法に変換
- nested `EApp` を applicative な記法に変換
- typenum を自然数表記に変換
- helper trait 名を実際の演算名で表示

## 8. diagnostics の強化

trait 未実装や capability failure を、typelude の語彙で説明したい。

### 8.1 capability model

trait failure を単なる未実装ではなく、要求能力の不足として表現する。

候補:

- `IsBool`
- `IsList`
- `Callable`
- `LookupKey<K>`
- `SupportsAdd`
- `SupportsCompare`

### 8.2 方針

- capability trait に `#[diagnostic::on_unimplemented]`
- blanket impl / helper impl の一部に `#[diagnostic::do_not_recommend]`
- rustc JSON diagnostics を後段で整形
- typelude-specific explanation を重ねる

これにより、低レベルな trait error を高レベルな意味論エラーに持ち上げやすくなる。

## 9. テスト基盤

`static_assertions` は土台として有効だが、それだけでは不足する。

欲しい assertion:

- `assert_type_eq`
- `assert_not_type_eq`
- `assert_eval_to`
- `assert_trait_impl`
- `assert_trait_not_impl`
- `assert_render_eq`
- `assert_compile_error_contains`
- `assert_reduces_in_at_most`

さらに欲しいのは golden test。

- golden trace
- golden diagnostics
- golden graph
- golden render

これにより、tooling の品質を継続的に固定しやすくなる。

## 10. IDE / MCP / 対話的支援

将来的には、観測系を editor や対話ツールに繋げられると強い。

候補:

- hover で `Evaluate<T>` の結果を表示
- hover で 1-step expansion を表示
- inlay hints で中間型を表示
- error 箇所で `why failed` を表示
- サイドパネルで evaluation tree を表示
- MCP で trace / graph / diff / explanation を問い合わせる

typelude は「コードを読む」より「問い合わせる」方が楽になる可能性が高い。

## 11. 成果物候補

### 11.1 typelude-specific

- `typelude-diagnostics`
- `typelude-render`
- `typelude-test`
- `typelude-explain`
- `typelude-lsp`

### 11.2 general tooling

- `type-trace-core`
- `type-profiler`
- `solver-graph-view`
- `rustc-log-formatter`
- `type-diff`

### 11.3 bridge

- `typelude-rustc-driver`
- `typelude-solver-view`

## 12. 優先度の高い核

全体の中で、特に価値が高く波及効果も大きい核は次。

1. trace event の共通表現
2. typelude-aware pretty printer
3. capability failure diagnostics
4. obligation / semantic profiler
5. solver graph から semantic graph への写像

この5つが揃うと、debug, test, docs, IDE, MCP の多くが同じ土台で動く。

## 13. 段階的ロードマップ

### Phase 1

- pretty print / notation simplification
- diagnostics 改善
- `RUSTC_LOG` と JSON diagnostics の整形
- typelude-specific test helpers

### Phase 2

- trace model
- semantic trace formatter
- profiling metrics の整備
- golden trace / diagnostics test

### Phase 3

- rustc_private ベースの解析器
- solver graph / semantic graph viewer
- LSP / MCP backend

## 14. まとめ

typelude のエコシステム強化は、次のように整理できる。

- rustc の出力を取る
- 汎用の trace / graph / profiler に整える
- typelude の意味論に沿って再解釈する
- 利用者向けに見やすく表示・説明する

つまり、最終的に目指すものは単なる補助ツール群ではなく、
型レベル計算のための observatory と explanation layer である。
