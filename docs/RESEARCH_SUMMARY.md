# Research Summary

typelude のデバッグ・診断・可視化・支援ツールに関する調査結果の要約。

詳細は以下を参照:

- [ECOSYSTEM.md](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/docs/ECOSYSTEM.md)
- [RUSTC_TOOLING.md](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/docs/RUSTC_TOOLING.md)
- [TOOLING_ARCHITECTURE.md](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/docs/TOOLING_ARCHITECTURE.md)

## 1. 結論

現時点で有望な方向は次の3本柱に集約できる。

1. typelude-specific な explanation / pretty print / diagnostics
2. rustc 出力を整形する汎用 tracing / profiling / graph tooling
3. trait solver の構造を直接読む `rustc_private` ベースの解析器

現在の方針は `rustc_private first` で、collector の本線は
`typelude-tooling-rustc-private` に置く。

最初に投資する価値が高いのは次。

- capability trait への `#[diagnostic::on_unimplemented]`
- `rustc_trait_selection::solve::inspect` を使った proof tree 収集
- `rustc_private` driver から core event schema への直接出力
- `-Z dump-mir=all` による MIR / region graph 観測
- `-Z time-passes`, `-Z print-type-sizes` による artifact profiling

## 2. エコシステムの整理

全体は次の3層で整理するのがよい。

### 2.1 rustc substrate

- diagnostics JSON
- legacy `RUSTC_LOG`
- `-Z self-profile`
- `-Z dump-dep-graph`
- `-Z unpretty`
- `-Z dump-mir`
- `rustc_private`
- `rustc_trait_selection::solve::inspect`

### 2.2 type-tooling core

- trace IR
- graph IR
- profiler
- diagnostics formatter
- trace formatter
- type diff
- failing slice minimizer

### 2.3 typelude adapter

- typelude-aware pretty print
- helper trait を畳んだ表示
- capability failure explanation
- semantic graph
- typelude-specific guardrail

Source:

- [ECOSYSTEM.md](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/docs/ECOSYSTEM.md)

## 3. typelude-specific に必要なもの

typelude 固有の支援として価値が高いもの:

- `EIf`, `EWhile`, `EApp`, `EGet`, `EMap` の explanation
- `Array<...>` を list 記法へ落とす pretty print
- typenum や helper trait 名の簡約表示
- capability failure を typelude の語彙で説明する diagnostics
- VM state, trace, stack の専用表示

値側との接続も重要。

- `Reify`
- `Reflect`
- `Render`
- `Diagnose`

これにより debug, test, docs, IDE, MCP をつなげやすくなる。

Source:

- [ECOSYSTEM.md](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/docs/ECOSYSTEM.md)

## 4. 型プログラミング一般の tooling

typelude を越えて一般化しやすいもの:

- trait solving trace formatter
- obligation / solver graph viewer
- compile-time profiler
- rustc diagnostics / logs の整形
- proof tree viewer
- type diff
- failing slice minimizer

この層は typelude で試しつつ、他の型レベルライブラリにも再利用しやすい形で切り出すのがよい。

Source:

- [ECOSYSTEM.md](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/docs/ECOSYSTEM.md)

## 5. trait 解決の観測

trait 解決を観測する入口は大きく2つある。

### 5.1 `RUSTC_LOG`

性質:

- すぐ使える
- raw log として取れる
- 調査開始には便利
- 構造は弱い

実測で有効だったもの:

- `RUSTC_LOG=rustc_trait_selection=info`
- `RUSTC_LOG=rustc_trait_selection::traits::normalize=info`
- `RUSTC_LOG=rustc_trait_selection::traits::wf=info`
- `RUSTC_LOG_LINES=1`
- `RUSTC_LOG_FORMAT_JSON=1`
- `RUSTC_LOG_OUTPUT_TARGET=<relative-path>`

実測で分かった制約:

- official nightly では static max level が `info`
- `debug` / `trace` は使えない場面がある
- function/query/span filter は docs 上は存在するが、この build では実質効かないケースがあった

現時点の位置づけ:

- legacy / compatibility backend
- 調査用には使える
- 本線 collector には戻さない

### 5.2 `rustc_trait_selection::solve::inspect`

性質:

- proof tree を構造化して読む API
- `Goal`, `Probe`, `ProbeKind`, `ProofTreeVisitor` などの概念がある
- `rustc_private` 前提
- 実装コストは高いが本命

向いているもの:

- obligation graph
- candidate selection graph
- why / why-not explainer
- semantic graph への写像
- profiler の構造データ源

結論:

- `inspect` は本番 backend
- `RUSTC_LOG` は補助経路

Sources:

- [RUSTC_TOOLING.md](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/docs/RUSTC_TOOLING.md)
- [Using tracing to debug the compiler](https://rustc-dev-guide.rust-lang.org/tracing.html)
- [Trait solving (new)](https://rustc-dev-guide.rust-lang.org/solve/trait-solving.html)
- [rustc_trait_selection::solve](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/solve/index.html)
- [rustc_trait_selection::solve::inspect](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/solve/inspect/index.html)

## 6. profiling で欲しい指標

typelude では、物理時間だけでなく構造コストを測る必要がある。

主要指標:

- step count
- trait obligation count
- candidate count
- branch count
- recursion depth
- max goal depth
- type size
- type growth
- re-evaluation count
- normalization count
- cache hit rate
- failed obligations
- wall time

欲しいビュー:

- type flamegraph
- obligation flamegraph
- recursion tree
- branch profile
- type growth chart
- re-evaluation report

Source:

- [ECOSYSTEM.md](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/docs/ECOSYSTEM.md)

## 7. rustc 側で有望だった機能

### 7.1 強い

- `#[diagnostic::on_unimplemented]`
- `--error-format=json` / `--json`
- `RUSTC_LOG=rustc_trait_selection=info`
- `-Z dump-mir=all`
- `-Z time-passes`
- `-Z print-type-sizes`

### 7.2 raw だが有望

- `-Z self-profile`
- `-Z unpretty=hir`

### 7.3 追加検証が必要

- `-Z dump-dep-graph`
- `-Z next-solver`
- proof tree inspection の実ツール化

Source:

- [RUSTC_TOOLING.md](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/docs/RUSTC_TOOLING.md)

## 8. diagnostics 強化

方向性:

1. capability trait に `#[diagnostic::on_unimplemented]`
2. 内部 helper / blanket impl に `#[diagnostic::do_not_recommend]`
3. rustc JSON diagnostics を typelude-aware に整形

候補 capability:

- `IsBool`
- `IsList`
- `Callable`
- `LookupKey<K>`
- `SupportsAdd`
- `SupportsCompare`

狙い:

- Rust の trait error を typelude の意味論エラーに持ち上げる
- helper trait noise を隠す
- compile-fail / golden diagnostics test を支える

Sources:

- [RUSTC_TOOLING.md](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/docs/RUSTC_TOOLING.md)
- [Rust Reference: diagnostic attributes](https://doc.rust-lang.org/nightly/reference/attributes/diagnostics.html)

## 9. テスト基盤

欲しい assertion:

- `assert_type_eq`
- `assert_not_type_eq`
- `assert_eval_to`
- `assert_trait_impl`
- `assert_trait_not_impl`
- `assert_render_eq`
- `assert_compile_error_contains`
- `assert_reduces_in_at_most`

欲しい golden:

- trace
- diagnostics
- graph
- render

Source:

- [ECOSYSTEM.md](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/docs/ECOSYSTEM.md)

## 10. 成果物候補

typelude-specific:

- `typelude-diagnostics`
- `typelude-render`
- `typelude-test`
- `typelude-explain`
- `typelude-lsp`

general tooling:

- `type-trace-core`
- `type-profiler`
- `solver-graph-view`
- `rustc-log-formatter`
- `type-diff`

bridge:

- `typelude-rustc-driver`
- `typelude-solver-view`

Source:

- [ECOSYSTEM.md](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/docs/ECOSYSTEM.md)

## 11. 直近の実装優先度

優先度順:

1. typelude-aware pretty printer
2. capability diagnostics
3. `RUSTC_LOG` JSON 収集と trace IR
4. `dump-mir` / `time-passes` / `print-type-sizes` の活用
5. `rustc_private` ベースの `inspect` backend

この順なら、短期の UX 改善と中長期の構造化解析の両方がつながる。

### 11.1 詳細順位

より細かく並べると次。

1. `typelude-aware diagnostics`
2. `pretty printer`
3. `typelude test kit`
4. `RUSTC_LOG formatter`
5. `trace IR`
6. `semantic mapper`
7. `semantic trace viewer`
8. `type profiler`
9. `semantic profiler`
10. `solver graph viewer`
11. `self-profile integration`
12. `rustc-driver based analyzer`
13. `solve::inspect backend`
14. `IDE / LSP support`
15. `MCP backend`

判断基準:

- すぐ価値が出るか
- 後続の基盤になるか
- 実装コストと保守コストが暴れにくいか

考え方:

- まずは diagnostics, render, tests の UX 改善を先にやる
- 次に `RUSTC_LOG` を使った lightweight な tracing / profiling を整える
- 最後に `rustc_private` と `solve::inspect` を使う本命 backend へ進む

## 12. アーキテクチャ方針

積み上げ方は次の4層を基本とする。

1. `collectors`
2. `core model`
3. `typelude adapter`
4. `products`

全体像:

```text
rustc outputs / rustc_private
        ↓
    collectors
        ↓
    core model
        ↓
 typelude adapter
        ↓
 products
```

重要な設計原則:

- collector は raw 収集に徹する
- typelude-specific な解釈は adapter に閉じ込める
- product は collector に直接依存しない
- text 出力より先に IR / schema を定義する
- nightly / `rustc_private` は隔離 crate に閉じ込める

最初に固定したい共通基盤:

- `trace IR`
- `diagnostic IR`
- `semantic mapper interface`

Source:

- [TOOLING_ARCHITECTURE.md](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/docs/TOOLING_ARCHITECTURE.md)

## 13. 外部ソース

今回の調査で主に参照した外部ソース:

- [Using tracing to debug the compiler](https://rustc-dev-guide.rust-lang.org/tracing.html)
- [Trait solving (new)](https://rustc-dev-guide.rust-lang.org/solve/trait-solving.html)
- [rustc command-line arguments](https://doc.rust-lang.org/beta/rustc/command-line-arguments.html)
- [Rust JSON output](https://doc.rust-lang.org/beta/rustc/json.html)
- [Rust Reference: diagnostic attributes](https://doc.rust-lang.org/nightly/reference/attributes/diagnostics.html)
- [rustc_trait_selection::solve](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/solve/index.html)
- [rustc_trait_selection::solve::inspect](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_trait_selection/solve/inspect/index.html)
