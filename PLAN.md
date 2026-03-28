## Tooling Foundation Crates Plan

### Summary
この計画では、tooling 基盤をまず同一 workspace 内の `crates/` 配下に追加し、既存の `typelude-std` / `typelude-vm` / `typelude-macros` には最小限の依存だけを持たせる。`typelude-core` は今回は復活させず、将来そこへ寄せられるように依存方向だけ整える。最初に切る crate は 4 つに限定する。

- `typelude-tooling-core`
- `typelude-tooling-rustc`
- `typelude-tooling-typelude`
- `typelude-tooling-cli`

この 4 crate で、`collector -> IR -> typelude adapter -> product` の骨格を先に固定する。`rustc_private`、LSP、MCP は v1 の crate には含めない。

### Crate Layout And Interfaces
#### `typelude-tooling-core`
責務は共通 IR と共有型のみ。外部依存は標準ライブラリ中心に抑え、`rustc_private` や既存 typelude crate に依存しない。

公開する最低限のモジュール:
- `ids`: `TraceId`, `EventId`, `NodeId`, `GoalId`, `SpanId`, `TypeId`
- `span`: source span と origin 情報
- `trace`: `Trace`, `TraceEvent`, `TraceEventKind`
- `graph`: `Graph`, `GraphNode`, `GraphEdge`
- `diagnostic`: `DiagnosticRecord`, `DiagnosticKind`, `CapabilityFailure`
- `metrics`: `MetricRecord`, `MetricKind`
- `render`: `RenderMode`, `RenderedText`
- `error`: tooling 用の共通 error 型

`TraceEventKind` は v1 で次を固定する。
- `GoalStarted`
- `GoalFinished`
- `CandidateTried`
- `CandidateChosen`
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

#### `typelude-tooling-rustc`
責務は rustc 出力の collector。`typelude-tooling-core` のみに依存し、`typelude-tooling-typelude` には依存しない。v1 は `rustc_private` を使わず、安定した入力だけを扱う。

公開する collector:
- `log`: `RustcLogCollector`, `RustcLogConfig`
- `diagnostics`: `RustcDiagnosticsCollector`, `RustcDiagnosticsConfig`
- `mir`: `MirArtifactCollector`, `MirArtifactConfig`
- `profile`: `SelfProfileCollector`, `SelfProfileConfig`
- `passes`: `TimePassesCollector`
- `types`: `TypeSizesCollector`

v1 で扱う入力:
- `RUSTC_LOG`
- `--error-format=json --json=...`
- `-Z dump-mir=all`
- `-Z time-passes`
- `-Z print-type-sizes`
- `-Z self-profile`

v1 では `dump-dep-graph` と `solve::inspect` は interface だけ予約し、実装対象から外す。

#### `typelude-tooling-typelude`
責務は typelude-specific な解釈と render。`typelude-tooling-core` に依存し、既存の `typelude-std` と `typelude-vm` を読む。`typelude-tooling-rustc` には依存しない。

公開する機能:
- `mapper`: `SemanticMapper`, `SemanticNode`, `SemanticNodeKind`
- `diagnostics`: `TypeludeDiagnosticEnricher`
- `render`: `TypeludeRenderer`
- `metrics`: `TypeludeMetricEnricher`
- `caps`: capability 名の正規化と分類
- `naming`: helper trait や internal symbol の圧縮表示

`SemanticNodeKind` は v1 で次を固定する。
- `Eval`
- `Apply`
- `If`
- `While`
- `Map`
- `Get`
- `PrimitiveOp`
- `HelperDispatch`
- `VmOp`
- `VmStep`

`TypeludeRenderer` の v1 出力対象:
- `Array<...>` の list 記法化
- typenum の自然数化
- helper trait 名の圧縮
- nested `EApp` の簡約表示
- capability 名の表示整形

#### `typelude-tooling-cli`
責務は人間向け product。`typelude-tooling-core`、`typelude-tooling-rustc`、`typelude-tooling-typelude` に依存する。既存 crate から直接 rustc を叩く責務を持たない。

v1 のサブコマンド:
- `trace`: `RUSTC_LOG` 収集と整形表示
- `diag`: rustc JSON diagnostics の typelude-aware 整形
- `render`: 型表記の簡約表示
- `profile`: `time-passes` / `print-type-sizes` / `self-profile` の要約
- `doctor`: 必要な env / toolchain / flags を検査

v1 の出力モード:
- `text`
- `json`

### Implementation Plan
#### Phase 1: Workspace Skeleton
- workspace `Cargo.toml` に 4 crate を member 追加する。
- crate 名と依存方向を固定する。
- `typelude-tooling-core` 以外は `core` を必須依存にする。
- `typelude-tooling-rustc` は `typelude` 系 crate に依存しない。
- `typelude-tooling-typelude` は `rustc` collector crate に依存しない。
- `typelude-tooling-cli` のみが両者を束ねる。

#### Phase 2: Core IR Freeze
- `ids`, `span`, `trace`, `diagnostic`, `metrics`, `render` の最小 public API を実装する。
- v1 では generic すぎる抽象化を避け、enum と struct を明示的に定義する。
- `serde` は `text/json` 出力のために `core` 側へ入れる。
- IR は append-only を前提にし、v1 の削除互換は考えない。

#### Phase 3: Rustc Collectors
- `RUSTC_LOG=rustc_trait_selection=info` を基準に collector を作る。
- `RUSTC_LOG_FORMAT_JSON=1` を標準入力形式にし、plain text log parser は補助扱いにする。
- diagnostics collector は rustc JSON diagnostic を `DiagnosticRecord` に落とす。
- MIR / time-passes / print-type-sizes / self-profile は artifact collector として分離する。
- collector は raw 情報を失わずに `core` IR に変換する。typelude-specific な解釈は入れない。

#### Phase 4: Typelude Adapter
- helper trait 群を semantic node に束ねる mapper を実装する。
- `Eval`, `EApp`, `EIf`, `EWhile`, `EGet`, `EMap`, VM trace を v1 の主対象にする。
- renderer は display 向けの簡約表示を先に実装し、graph export は後回しにする。
- diagnostic enricher は capability failure の説明だけに絞る。`why/why-not` の完全版は v2 に回す。
- metric enricher は `step_count`, `obligation_count`, `recursion_depth`, `type_size`, `re_eval_count` を v1 指標にする。

#### Phase 5: CLI Product
- `trace` は collector + mapper + renderer を束ね、collapsed trace を text/json で出す。
- `diag` は rustc JSON diagnostics を typelude-aware text/json に再構成する。
- `profile` は `time-passes`、`print-type-sizes`、`self-profile` を単一レポートにまとめる。
- `doctor` は nightly availability、required env vars、supported flags を確認する。

### Test Plan
- `typelude-tooling-core`
  - IR roundtrip tests
  - JSON serialization snapshot tests
  - stable id generation tests
- `typelude-tooling-rustc`
  - fixture-based parser tests for `RUSTC_LOG` JSON
  - fixture-based parser tests for rustc JSON diagnostics
  - sample `time-passes` / `print-type-sizes` / `self-profile` artifact parsing tests
- `typelude-tooling-typelude`
  - semantic mapping tests for `EIf`, `EWhile`, `EGet`, `EMap`
  - renderer snapshot tests for `Array<...>`, typenum, nested `EApp`
  - capability diagnostics enrichment tests
- `typelude-tooling-cli`
  - golden tests for `trace`, `diag`, `profile` subcommands
  - smoke tests against a minimal typelude sample crate

Acceptance criteria:
- `RUSTC_LOG` JSON から `TraceEvent` を生成できる
- rustc diagnostics JSON から typelude-aware diagnostics を出せる
- `Array<...>` と typenum の簡約表示ができる
- `trace` と `diag` の CLI 出力が golden で固定される

### Assumptions And Defaults
- 新規基盤 crate は同一 workspace の `crates/` 配下に置く。
- コメントアウトされている `typelude-core` は今回は復活させない。
- v1 は `rustc_private` 非依存で始める。
- `solve::inspect` backend は将来追加するが、今回は interface 予約のみとする。
- 最初の本当の共通基盤は `trace IR`、`diagnostic IR`、`semantic mapper` の 3 つとする。
- collector は dumb、adapter は typelude-specific、CLI は product 専任、という責務分離を守る。
