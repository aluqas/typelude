# Tooling

typelude の tooling に関する統合ドキュメント。
2026-03-28 時点では、この文書を tooling の主な入口とする。


## 1. 要約

現在の本線は `rustc_private first`。

- 主経路は `collect -> solve-tree -> solve-summary`
- 中核は `typelude-tooling-core`、`typelude-tooling-rustc-private`、`typelude-tooling-cli`
- typelude 固有 UX は `typelude-tooling-typelude` に集約しつつあるが、まだ部分実装が多い
- `typelude-tooling-rustc` は artifact/parser 層として存在するが、本線 collector ではない
- LSP / MCP / 高度な explain / 高度な profiler はまだ後段

設計原則は次の通り。

- collector は dumb に保つ
- typelude-specific な解釈は adapter に閉じ込める
- product は collector ではなく IR に依存させる
- nightly / `rustc_private` 依存は専用 crate に隔離する
- 先に構造化 schema を固め、その上に UX を載せる

## 2. 現在の構成

現在の主な crate:

- `typelude-tooling-core`
- `typelude-tooling-rustc-private`
- `typelude-tooling-rustc`
- `typelude-tooling-typelude`
- `typelude-tooling-cli`

現在の基本フロー:

```text
rustc_private driver
  -> trace NDJSON / core event schema
  -> solve tree / solve summary
  -> graph / diagnostics / render / profile
```

現在の user-facing 入口:

- `collect`
- `trace`
- `solve-tree`
- `solve-summary`
- `solve-owner`
- `solve-impl`
- `solve-assoc-item`
- `solve-diff`
- `diag`
- `render`
- `profile`
- `doctor`
- `graph`
- `analyze`

## 3. 進捗ラベル

この文書では進捗を次で表す。

- `主経路`: 現在の主要な利用経路として機能している
- `部分実装`: 機能はあるが限定的、または補助経路
- `雛形あり`: crate / module / interface はあるが、完成度は低い
- `未着手に近い`: 提案段階に近い

## 4. 進捗サマリ

### 4.1 ツール提案

| 項目 | 状態 | 概要 |
|---|---|---|
| `typelude-tooling-core` | 主経路 | trace / solve / graph / diagnostic / metrics / render の共通 IR がある |
| `typelude-tooling-rustc-private` | 主経路 | `rustc_driver` 経由で subject / goal / candidate を収集し NDJSON に出す |
| `typelude-tooling-cli` | 主経路 | `collect`, `solve-tree`, `solve-summary`, `solve-owner`, `solve-diff` が主な入口 |
| `typelude-tooling-rustc` | 部分実装 | diagnostics, MIR, self-profile, time-passes, type-sizes collector がある |
| `typelude-tooling-typelude` | 部分実装 | render, diagnostics enrichment, graph, analysis, mapper, metrics がある |
| `typelude-tooling-lsp` | 未着手に近い | 設計候補のみ |
| `typelude-tooling-mcp` | 未着手に近い | 設計候補のみ |
| `typelude-diagnostics` / `typelude-prof` / `typelude-explain` などの分離 product | 未着手に近い | 役割は見えているが独立 product 化はまだ |

### 4.2 UX提案

| 項目 | 状態 | 概要 |
|---|---|---|
| solve tree / solve summary 表示 | 主経路 | trait solving flow の観測の中心 |
| focused owner query | 主経路 | `solve-owner` で build-wide collect を経由せずに focused 実行できる |
| summary diff | 部分実装 | `solve-diff` はあるが比較粒度はまだ粗い |
| pretty print | 部分実装 | `Array<...>` の list 記法化、typenum 表示、nested `EApp` 簡約がある |
| helper trait / internal symbol の圧縮表示 | 部分実装 | compact formatter と symbol compression がある |
| semantic mapper | 部分実装 | `EIf`, `EWhile`, `EGet`, `EMap`, `EApp` などへの分類はあるが heuristic 寄り |
| VM state / trace / stack の専用表示 | 未着手に近い | 提案はあるが主経路ではない |
| IDE / LSP / MCP 接続 | 未着手に近い | まだ CLI 中心 |

### 4.3 診断・説明機能

| 項目 | 状態 | 概要 |
|---|---|---|
| rustc JSON diagnostics 取り込み | 部分実装 | collector と `diag` コマンドがある |
| typelude-aware diagnostics enrichment | 部分実装 | capability failure を正規化し補足 note を付けられる |
| `why failed` explanation | 部分実装 | `IsBool`, `IsList`, `Callable`, `LookupKey`, `SupportsAdd`, `SupportsCompare` に説明がある |
| helper trait noise の抑制 | 部分実装 | 表示圧縮はあるが compiler 側 attribute 適用は未完 |
| `#[diagnostic::on_unimplemented]` の広域適用 | 雛形あり | 方針は固いが、workspace 全体での展開はこれから |
| `#[diagnostic::do_not_recommend]` の広域適用 | 雛形あり | 方針は固いが、実装の広がりはまだ小さい |
| `why not?` / 完全な semantic diagnostics | 未着手に近い | 後段の機能 |

### 4.4 可視化・分析・profiling

| 項目 | 状態 | 概要 |
|---|---|---|
| obligation / goal / candidate の tree 表示 | 主経路 | `solve-tree` が現在の中心 |
| summary / diff / focused compare | 主経路 | `solve-summary`, `solve-owner`, `solve-diff` がある |
| graph builder | 部分実装 | trace から graph を構成できる |
| graph export | 部分実装 | DOT / Mermaid 出力がある |
| graph analysis | 部分実装 | root, max depth, critical path, hot nodes, cycle 判定, kind 分布がある |
| profiling artifact ingestion | 部分実装 | `time-passes`, `type-sizes`, `self-profile`, `MIR` collector がある |
| typelude metric enrichment | 部分実装 | `step_count`, `obligation_count`, `recursion_depth`, `re_eval_count` などがある |
| proof tree viewer | 雛形あり | solver flow の基盤はあるが専用 viewer としては未完成 |
| obligation graph viewer | 雛形あり | graph はあるが UX はまだ粗い |
| flamegraph / branch profile / growth chart | 未着手に近い | 指標候補はあるが product 化されていない |
| failing slice minimizer | 未着手に近い | 提案段階 |

## 5. 現在の本線

現時点で最も進んでいるのは、typelude の意味論を直接見せることよりも、
trait solving flow を構造化して観測・比較する流れである。

中心機能:

1. `collect`
2. `solve-tree`
3. `solve-summary`

focused 実行:

- `solve-owner`
- `solve-impl`
- `solve-assoc-item`

比較:

- `solve-diff`

このため、現在の tooling は
「typelude の完全な semantic debugger」より
「rustc trait solver の構造化ビューア」に近い。

## 6. 主要モジュールごとの現在地

### 6.1 `typelude-tooling-core`

すでに安定化を進めている中心層。

主な実体:

- `Trace`, `TraceEvent`, `TraceEventKind`
- `GoalTree`
- `SolveSummary`
- `Graph`, `GraphNode`, `GraphEdge`
- `DiagnosticRecord`
- `MetricRecord`
- shared ids

現状評価:

- 共通 schema としてかなり前進している
- 今後も product はこの層への依存を優先する

### 6.2 `typelude-tooling-rustc-private`

現在の本線 collector / analysis engine。

主な実体:

- session
- frontends
- hooks
- subjects
- focused queries
- filters

現状評価:

- 現在の最重要 crate
- `RUSTC_LOG` ではなく `rustc_private` 経由でイベントを取る方針が反映されている

### 6.3 `typelude-tooling-rustc`

artifact/parser 層。

主な実体:

- rustc diagnostics parser
- MIR artifact collector
- self-profile collector
- time-passes collector
- type-sizes collector

現状評価:

- 補助層として有用
- ただし trait solving の本線 collector ではない
- 現在の `rustc-private` 側では、これらの同等機能はまだ吸収していない

### 6.3.1 `rustc-private` へ寄せる場合の実装ルート

`typelude-tooling-rustc` にある各機能を `rustc_private` 側へ寄せる場合に、
参照する rustc 側 API / 設定面は次。

#### diagnostics

- compiler session 入口:
  [`rustc_interface::interface::Compiler`](https://doc.rust-lang.org/beta/nightly-rustc/rustc_interface/interface/struct.Compiler.html)
  は `sess: Session` を持つ
- diagnostics context:
  [`rustc_errors::DiagCtxtHandle`](https://doc.rust-lang.org/beta/nightly-rustc/rustc_errors/struct.DiagCtxtHandle.html)
- emitted diagnostics の通過点:
  [`rustc_errors::TRACK_DIAGNOSTIC`](https://doc.rust-lang.org/beta/nightly-rustc/rustc_errors/index.html)
- JSON emitter:
  [`rustc_errors::json::JsonEmitter`](https://doc.rust-lang.org/beta/nightly-rustc/rustc_errors/json/index.html)
- error 出力種別:
  [`rustc_session::config::ErrorOutputType`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/config/index.html)

`rustc_private` 側へ寄せる場合の API ルートは、
`Compiler.sess` / diagnostics context で診断を捕捉するか、
あるいは rustc の JSON emitter 経路を driver 内で制御して
構造化出力へ流すことになる。

#### MIR

- 中心 query context:
  [`rustc_middle::ty::TyCtxt`](https://doc.rust-lang.org/beta/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html)
- MIR body 取得:
  `TyCtxt::optimized_mir`
  [`source`](https://doc.rust-lang.org/beta/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html)
- CTFE 向け MIR:
  `TyCtxt::mir_for_ctfe`
  [`source`](https://doc.rust-lang.org/beta/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html)
- instance 単位の MIR:
  `TyCtxt::instance_mir`
  [`source`](https://doc.rust-lang.org/beta/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html)
- unstable option 側の dump 制御:
  `dump_mir`, `dump_mir_dir`, `dump_mir_graphviz`
  in [`rustc_session::config::UnstableOptions`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/config/struct.UnstableOptions.html)

file artifact を読む代わりに、`TyCtxt` query から直接 MIR body を取るルートがある。
従来通り dump ファイルを使う場合は `UnstableOptions` 側の出力設定が入口になる。

#### self-profile

- unstable option:
  `self_profile`, `self_profile_counter`, `self_profile_events`
  in [`rustc_session::config::UnstableOptions`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/config/struct.UnstableOptions.html)

この機能は rustc 内部 profiler の有効化と出力先設定が中心で、
現在の `typelude-tooling-rustc` がやっていることは生成 artifact の後段収集である。
`rustc_private` 側へ寄せる場合も、入口は `UnstableOptions` の profiler 設定になる。

#### `time-passes`

- unstable option:
  `time_passes`, `time_passes_format`
  in [`rustc_session::config::UnstableOptions`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/config/struct.UnstableOptions.html)
- timing diagnostics API:
  `emit_timing_section_start`, `emit_timing_section_end`
  on [`rustc_errors::DiagCtxtHandle`](https://doc.rust-lang.org/beta/nightly-rustc/rustc_errors/struct.DiagCtxtHandle.html)

`time-passes` 相当を driver 内で拾う場合は、
`UnstableOptions` による compiler timing 出力設定と diagnostics timing API が入口になる。

#### `print-type-sizes`

- unstable option:
  `print_type_sizes`
  in [`rustc_session::config::UnstableOptions`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_session/config/struct.UnstableOptions.html)

これは現在の parser が text 出力を読んでいる機能で、
移行時の rustc 側入口は `UnstableOptions::print_type_sizes` になる。

### 6.4 `typelude-tooling-typelude`

typelude 固有の adapter 層。

主な実体:

- `TypeludeRenderer`
- `TypeludeDiagnosticEnricher`
- `SemanticMapper`
- `TraceGraphBuilder`
- `GraphAnalysis`
- `TypeludeMetricEnricher`

現状評価:

- 方向性は合っている
- まだ typelude 固有 UX の中心と呼ぶには早い
- heuristic な処理が多く、後で solver 構造と tighter に結びつけたい

### 6.5 `typelude-tooling-cli`

現在の user-facing product。

現状評価:

- 実際に触れる入口としては最も進んでいる
- tree / summary / diff / query の UX をここで磨く段階

## 7. いま未成熟な部分

現在まだ弱い領域は次。

- typelude の意味論そのものを直接説明する UX
- VM 専用の可視化
- `why not?` explainer
- graph viewer の洗練
- profiler の product 化
- diagnostics attribute の広域適用
- LSP / MCP
- failing slice minimizer

## 8. 優先順位

現状から自然な優先順は次。

1. 現在の主経路である `solve-tree` / `solve-summary` / `solve-owner` / `solve-diff` を強化する
2. compact formatter と typelude-aware render を拡充する
3. diagnostics enrichment を compiler 側 attribute と後段整形の両方で強化する
4. graph / analyze / profile を「実験機能」から「使える機能」へ引き上げる
5. semantic mapper を proof tree / goal structure とより強く結びつける
6. その後に LSP / MCP へ広げる

## 9. 当面の実務的な次手

短期で効果が高いもの:

- real fixture を増やす
- compact formatter を拡充する
- `solve-owner` の owner 解決精度を上げる
- `solve-diff` の比較粒度を増やす
- capability diagnostics を増やす
- render 対象を `EIf`, `EWhile`, `EGet`, `EMap`, VM 表記まで広げる

中期の課題:

- proof tree を核にした semantic mapping の強化
- graph viewer の意味論寄り表現
- profiler の可視化
- IDE / MCP への接続

## 10. semantic extension 案

個人的な設計案として、typelude 固有の意味論を
`typelude-tooling-typelude` という product 寄りの別レイヤとして持つより、
`typelude-tooling` 本体に「特定プロジェクトの意味論を注入できる extension 機構」
として持つ方が自然ではないか、という案がある。

この案の背景:

- LSP / MCP まで進むと、product surface を project ごとに二重実装したくない
- 型レベル計算のかなりの部分は形式的で、共通基盤側に寄せやすい
- その一方で、project 固有の語彙や表示規則、診断語彙は差し替えたい

この観点では、分けたいのは product ではなく semantics である。

避けたい構図:

- 共通 CLI / LSP / MCP と、typelude 専用 CLI / LSP / MCP が並立する
- product ごとに typelude-specific 束ね直しが必要になる
- graph / diagnostics / render / explain の UX を毎回 project 側で再実装する

代わりに目指す構図:

```text
rustc frontend / analysis engine
  -> core IR / graph / diagnostics / metrics
  -> semantic extension
  -> products (CLI / LSP / MCP)
```

この形では:

- product は 1 系統でよい
- typelude 固有の意味論は extension として注入する
- 将来、他の type-level project でも同じ product surface を共有しやすい

### 10.1 extension として切り出したい責務

semantic extension に寄せたい責務の候補:

- symbol normalization
- type rendering
- capability classification
- diagnostic explanation
- semantic node mapping
- graph enrichment
- metric enrichment
- subject / query の project-specific hint

typelude では現在 `typelude-tooling-typelude` にある機能の多くが、
将来的にはこの extension 境界に収まる想定になる。

### 10.2 crate 境界の再整理イメージ

現状:

- `typelude-tooling-core`
- `typelude-tooling-rustc-private`
- `typelude-tooling-rustc`
- `typelude-tooling-typelude`
- `typelude-tooling-cli`

将来候補:

- `typelude-tooling-core`
- `typelude-tooling-engine`
- `typelude-tooling-product`
- `typelude-tooling-semantic-api`
- `typelude-tooling-semantic-typelude`

ただし、ここで重要なのは crate 名ではなく境界である。

要点:

- engine / product は共通に保つ
- semantics は extension として差し込む
- `typelude-tooling-typelude` は product ではなく semantic extension crate とみなす

### 10.3 この案の位置づけ

これはまだ確定方針ではないが、LSP / MCP を視野に入れたときの
境界設計として有力な候補である。

現時点では:

- 実装の本線は引き続き `rustc_private first`
- 現在の `typelude-tooling-typelude` は実験・先行実装の置き場として有効
- ただし長期的には「typelude-specific product」ではなく
  「semantic extension 実装」として再定義する余地がある
