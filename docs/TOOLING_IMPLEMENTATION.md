# Tooling Implementation Status

現在の tooling 実装の整理メモ。  
設計案ではなく、「今 workspace に何があり、どこまで動くか」をまとめる。

関連資料:

- [RESEARCH_SUMMARY.md](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/docs/RESEARCH_SUMMARY.md)
- [TOOLING_ARCHITECTURE.md](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/docs/TOOLING_ARCHITECTURE.md)
- [RUSTC_TOOLING.md](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/docs/RUSTC_TOOLING.md)

## 1. 方針

現状の本線は `rustc_private first`。

- `RUSTC_LOG` は collector の本線ではない
- trait solving の観測は `rustc_private` driver から直接取る
- CLI は `collect -> solve-tree -> solve-summary` を中心にする
- typelude 固有の意味論写像は主眼ではなく、まず trait solving flow の観測を優先する

## 2. Crate 構成

### 2.1 `typelude-tooling-core`

共通 schema / IR。

主な責務:

- `Trace`, `TraceEvent`, `TraceEventKind`
- `SubjectId`, `GoalId`, `CandidateId`, `DiagId` などの id
- `GoalTree`
- `SolveSummary`
- trace NDJSON の read/write

主なファイル:

- [crates/typelude-tooling-core/src/trace.rs](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/crates/typelude-tooling-core/src/trace.rs)
- [crates/typelude-tooling-core/src/solve.rs](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/crates/typelude-tooling-core/src/solve.rs)

### 2.2 `typelude-tooling-rustc-private`

本線の analysis engine。

主な責務:

- `rustc_driver` への接続
- `TyCtxt` 上での subject 解決
- hook 実行
- owner-based query 実行
- core event schema への NDJSON 出力

主な構成:

- [crates/typelude-tooling-rustc-private/src/lib.rs](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/crates/typelude-tooling-rustc-private/src/lib.rs)
- [crates/typelude-tooling-rustc-private/src/frontends.rs](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/crates/typelude-tooling-rustc-private/src/frontends.rs)
- [crates/typelude-tooling-rustc-private/src/session.rs](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/crates/typelude-tooling-rustc-private/src/session.rs)
- [crates/typelude-tooling-rustc-private/src/hooks/](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/crates/typelude-tooling-rustc-private/src/hooks)
- [crates/typelude-tooling-rustc-private/src/queries/](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/crates/typelude-tooling-rustc-private/src/queries)
- [crates/typelude-tooling-rustc-private/src/subjects/](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/crates/typelude-tooling-rustc-private/src/subjects)
- [crates/typelude-tooling-rustc-private/src/filters/](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/crates/typelude-tooling-rustc-private/src/filters)

### 2.3 `typelude-tooling-cli`

user-facing frontend / orchestrator。

主な責務:

- driver build / 実行
- trace の収集
- solve tree / summary の表示
- owner query の user-facing 入口
- summary diff
- diagnostics / render / profile / graph / analyze

主なファイル:

- [crates/typelude-tooling-cli/src/lib.rs](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/crates/typelude-tooling-cli/src/lib.rs)
- [crates/typelude-tooling-cli/src/solve_view.rs](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/crates/typelude-tooling-cli/src/solve_view.rs)

### 2.4 `typelude-tooling-rustc`

artifact/parser 専用。

主な責務:

- rustc JSON diagnostics
- MIR artifacts
- self-profile
- time-passes
- type sizes

### 2.5 `typelude-tooling-typelude`

typelude 固有の adapter 層。

現状の主な責務:

- render
- diagnostics enrichment
- graph/analyze の一部
- trace metrics enrichment

ただし、現在の主眼は trait solving flow の観測であり、意味論写像は主経路ではない。

## 3. rustc-private 側の現在構造

## 3.1 hooks

増えていく実装群は `hooks/` に切り出している。

現状の hook:

- `TraitSolveHook`
- `ItemStructureHook`
- `DiagnosticsHook`

`TraitSolveHook` は owner を sweep し、explicit predicate subject を列挙して solve する。  
proof tree の実処理本体は query と共有している。

## 3.2 subjects

現在の `ResolvedSubject` は次。

- `Item`
- `Impl`
- `AssocItem`
- `ExplicitPredicate`

subject は `label`, `key`, `metadata`, `parent` を持ち、event schema 上の `SubjectDiscovered` に落とす。

## 3.3 queries

focused 実行のための受け皿を先に作ってある。

現状の query:

- `SolveExplicitPredicateQuery`
- `ResolveOwnerQuery`

`ResolveOwnerQuery` は owner path substring を受け取り、その owner の explicit predicates だけを solve する。

## 3.4 filters

collector 側ではなく engine 側の汎用 filter。

現状:

- `FocusFilter`
- `SubjectFilter`

## 4. Core event schema

現在の主な event:

- `RunStarted`
- `RunFinished`
- `SubjectDiscovered`
- `GoalDiscovered`
- `GoalEntered`
- `GoalExited`
- `CandidateDiscovered`
- `CandidateTried`
- `CandidateResult`
- `DiagnosticEmitted`
- `RelationDeclared`
- `Info`
- `ErrorRaised`

`GoalTree` はこの event 群から再構築する。  
旧 `RUSTC_LOG` 的な depth 復元は本線ではない。

## 5. 現在使える CLI

主なコマンド:

- `collect`
- `trace`
- `solve-tree`
- `solve-summary`
- `solve-owner`
- `solve-diff`
- `diag`
- `render`
- `profile`
- `doctor`
- `graph`
- `analyze`

trait-flow の正規経路は次。

1. `collect`
2. `solve-tree`
3. `solve-summary`

focused 実行の入口は `solve-owner`。

## 6. trait-flow で実装済みの機能

### 6.1 collect

`collect` は `cargo` を `RUSTC_WRAPPER` 付きで実行し、driver が出す NDJSON trace を保存する。

対応:

- `--package`
- `--manifest-path`
- `--toolchain`
- `--hook`
- `--subject-filter`
- `--rebuild-driver`

### 6.2 solve-tree

trace から `subject -> root goals -> nested goals -> candidates` を復元して表示する。

対応:

- `--output text|json`
- `--result`
- `--candidate-kind`
- `--max-depth`
- `--subject`

text 出力は compact formatter を通す。  
json は raw schema のまま。

### 6.3 solve-summary

trace から summary を出す。

指標:

- `subjects`
- `root_goals`
- `goals`
- `candidates`
- `max_goal_depth`
- `avg_candidates_per_goal`
- `result.ok`
- `result.no_solution`
- `result.ambiguous`
- `result.unsupported`
- `top_predicates`
- `top_candidate_kinds`

こちらも `solve-tree` と同じ filter を受ける。

### 6.4 compact formatter

text 表示専用。

現状の対象:

- `RigidAlias { ... }` → `RigidAlias`
- `Root { ... }` → `Root`
- `TraitCandidate { source: ParamEnv(..), ... }` → `TraitCandidate::ParamEnv`
- `Binder { value: TraitPredicate(...), bound_vars: [] }` の簡約
- `AliasRelate(...)`, `NormalizesTo(...)` の head 抽出

### 6.5 solve-owner

最初の user-facing focused entry。

入力:

- `--owner <substring>`
- `--package`
- `--manifest-path`
- `--toolchain`
- `--view tree|summary`
- `--output text|json`

挙動:

- build-wide `collect` を経由しない
- `rustc_private` 側の owner query を直接使う
- その場で trace を組み、tree または summary を返す

### 6.6 solve-diff

summary 同士の比較。

入力:

- `--left`
- `--right`
- `--output text|json`

現状の入力形式:

- trace NDJSON
- summary JSON
- summary text

比較対象:

- scalar 指標
- `top_predicates`
- `top_candidate_kinds`

## 7. typelude-vm 実例

small real-subject の smoke 対象として次を置いている。

- `RunWriter`
- `RunState`
- `OpPush`
- `OpIf`

現状は repo に重い real fixture を固定せず、
[crates/typelude-tooling-cli/tests/real_subjects.rs](/Users/saqula/Documents/02_codes/github.com/aluqas/typelude/crates/typelude-tooling-cli/tests/real_subjects.rs)
で ignored smoke test として保持している。

## 8. テストの現状

通っているもの:

- CLI unit tests
- `solve_view` unit tests
- `golden.rs`
- `rustc-private` unit tests
- ignored real-subject smoke test

実行コマンド:

```bash
env -u RUSTC_WRAPPER SCCACHE_DISABLE=1 cargo +nightly test \
  -p typelude-tooling-cli \
  -p typelude-tooling-rustc-private
```

## 9. 現在まだ主経路でないもの

- typelude 意味論への写像
- semantic graph
- LSP / MCP
- owner より細かい predicate-index query
- summary diff の構造差分強化
- checked-in real NDJSON fixture

## 10. いま自然な次手

優先度が高いもの:

1. `RunWriter` の real fixture 固定
2. `RunState`, `OpPush`, `OpIf` の fixture 追加
3. compact formatter の拡充
4. `solve-owner` の owner 解決精度向上
5. `solve-diff` の比較粒度追加

この段階では、まず trait solving flow を観測・比較できることを優先し、typelude-specific な意味論可視化は後段に回すのがよい。
