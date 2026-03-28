# Tooling Architecture

typelude 向けの debug / diagnostics / profiling / graph tooling を、共通基盤から段階的に積み上げるための大まかな設計メモ。

## 1. 基本方針

単発ツールを個別に作るのではなく、次の4層で積み上げる。

1. `collectors`
2. `core model`
3. `typelude adapter`
4. `products`

全体の流れは次の通り。

```text
rustc_private / rustc artifacts
        ↓
    collectors
        ↓
    core model
        ↓
 typelude adapter
        ↓
 products (CLI / tests / LSP / MCP / viewers)
```

狙い:

- 先に `rustc_private` collector を本線として固定する
- `RUSTC_LOG` は互換 backend に落とす
- typelude-specific な処理を低レベル収集から分離する

## 2. Collectors

collector は「取るだけ」に徹する。
この層では typelude の意味論をできるだけ持ち込まない。

候補:

- `rustc_private` proof-tree collector
- diagnostics JSON collector
- MIR collector
- self-profile collector
- source collector
- legacy `RUSTC_LOG` collector

入力源:

- legacy `RUSTC_LOG`
- `--error-format=json` / `--json`
- `-Z dump-mir`
- `-Z self-profile`
- `rustc_trait_selection::solve::inspect`
- source code / spans

責務:

- raw 出力を読む
- source span や target 名を拾う
- 最小限の parse をする
- core model に流し込める形へ変換する

## 3. Core Model

共通基盤。最重要。
product や typelude adapter は collector に直接依存しない。

少なくとも次の IR を持つ。

### 3.1 trace IR

時系列イベントの表現。

候補イベント:

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

### 3.2 graph IR

構造の表現。

候補ノード / edge:

- `GoalNode`
- `CandidateNode`
- `ExprNode`
- `ObligationEdge`
- `ExpansionEdge`
- `SemanticMapEdge`

### 3.3 diagnostic IR

表示前の診断表現。

候補:

- `capability_required`
- `capability_missing`
- `actual_type`
- `expected_form`
- `origin_span`
- `obligation_chain`
- `suggestion`

### 3.4 metric IR

計測値の表現。

候補:

- `step_count`
- `obligation_count`
- `candidate_count`
- `branch_count`
- `recursion_depth`
- `goal_depth_max`
- `type_size`
- `type_growth`
- `re_eval_count`
- `normalization_count`
- `cache_hit_rate`
- `wall_time`

### 3.5 shared ids

最初に安定化したい要素:

- `TraceId`
- `NodeId`
- `GoalId`
- `SpanId`
- `TypeId`

## 4. Typelude Adapter

低レベルの compiler 事実を、typelude の概念へ写像する層。

この層が typelude-specific UX の中心になる。

### 4.1 semantic mapper

役割:

- helper trait 群を `EIf`, `EWhile`, `EGet`, `EMap` などに束ねる
- alias expansion を primitive composition として再解釈する
- obligation / candidate を semantic node に対応付ける

### 4.2 pretty / render

役割:

- `Array<...>` の list 記法化
- typenum の自然数化
- nested `EApp` の簡約表示
- helper trait 名の圧縮
- VM state の専用表示

### 4.3 diagnostic enricher

役割:

- trait error を capability failure に持ち上げる
- helper trait noise を隠す
- `why failed` を生成する

### 4.4 metric enricher

役割:

- semantic step count
- loop iteration count
- lookup depth
- re-evaluation hot spot

を semantic node 単位で付与する。

## 5. Products

上の共通基盤から生える利用面。

候補:

- CLI trace viewer
- diagnostics formatter
- pretty printer
- graph viewer
- profiler
- test kit
- LSP
- MCP backend

設計原則:

- product は collector に直接依存しない
- `core model` と `typelude adapter` のみに依存する

## 6. 推奨する積み上げ順

短期価値と長期移行性を両立する順番。

1. `diagnostic IR` + typelude-aware diagnostics
2. pretty / render
3. `rustc_private` collector を core event schema に揃える
4. `semantic mapper`
5. CLI collect/trace
6. `metric enricher`
7. profiler
8. graph viewer
9. legacy `RUSTC_LOG` compatibility path
10. LSP / MCP

この順の利点:

- 最初に UX が改善する
- `RUSTC_LOG` を暫定 backend にできる
- 後から `solve::inspect` に差し替えても上位層を壊しにくい

## 7. crate 分割案

大まかな切り方:

- `typelude-tooling-core`
  - trace IR, graph IR, diagnostic IR, shared ids
- `typelude-tooling-rustc`
  - diagnostics JSON, MIR, self-profile など artifact parser
- `typelude-tooling-typelude`
  - semantic mapper, pretty/render, diagnostic enricher, metric enricher
- `typelude-tooling-cli`
  - trace/profiler/diagnostics の CLI
- `typelude-tooling-lsp`
  - editor integration
- `typelude-tooling-mcp`
  - agent / conversational interface

nightly / unstable 依存を隔離したい場合:

- `typelude-tooling-rustc-private`
  - `rustc_driver`
  - `rustc_interface`
  - `solve::inspect`
  - hook registry
  - raw event emission

## 8. 境界の切り方

長期的に崩れにくくするための原則。

- collector は dumb に保つ
- typelude-specific な解釈は adapter に閉じ込める
- 出力文字列より先に構造化 schema を定義する
- nightly / `rustc_private` 依存は専用 crate に隔離する
- product は IR にだけ依存させる

## 9. 最初に固定したいインターフェース

本当に先に決める価値が高いのは次。

- trace event schema
- diagnostic IR schema
- semantic mapper interface
- type rendering interface
- metric schema

これらが先にあると、
`RUSTC_LOG` backend で始めて後で `solve::inspect` backend に移る場合も、
上位の formatter / viewer / explainer を保ちやすい。

## 10. まとめ

この tooling では、最初に作るべき本当の共通基盤は次の3つ。

- `trace IR`
- `diagnostic IR`
- `semantic mapper interface`

これを中心に据えると、
`rustc_private` collector を本線に保ったまま、typelude adapter と products を独立に育てられる。
