# TypeScript Types-Only WASM Runtime Research

`MichiganTypeScript/typescript-types-only-wasm-runtime` の調査メモ。
`typelude-wasm` を設計する際の参照用に、実装範囲とパイプラインを整理する。

対象:

- 型システムだけで動く WASM runtime の構造
- 描画や文字列出力を含む output pipeline
- 巨大な型プログラムを実際に回す evaluation tooling
- `typelude` へ移植する場合の示唆

## 1. 要約

このプロジェクトは「TypeScript の型システムだけで WebAssembly を実行する」実験ではあるが、
単なる toy ではなく、次の 3 層を明確に分けている。

1. Rust 製 frontend
2. TypeScript 型レベル WASM runtime
3. 実行後の output extraction / rendering と evaluation tooling

特に重要なのは、Doom の描画ロジックが generic runtime の一部ではなく、
最終 `ProgramState` の `memory` を解釈する application-specific renderer だという点である。

また、repo 内で言う "GC" は WASM ヒープの GC ではない。
型レベル `memory` 表現が肥大化しすぎないように `L1Cache` を main memory へ sweep する
compile-time optimization である。

## 2. エンドツーエンドのパイプライン

全体の流れは次の通り。

```text
C / WAT / WASM
  -> Rust parser / lowering
  -> generated TypeScript type definitions
  -> entry<Args, DebugMode, StopAt>
  -> bootstrap<>
  -> ProgramState
  -> executeInstruction<>
  -> final ProgramState / raw results
  -> output extraction / rendering
```

### 2.1 frontend

Rust 側の `src/main.rs` が `wast` crate を使って `.wat` を parse し、
TypeScript の型定義に lowering する。

生成される TS module には概ね次が入る。

- `funcs`: 各関数の型表現
- `globals`: 初期グローバル値
- `memory`: 初期 linear memory
- `memorySize`: 初期メモリサイズ
- `indirect`: `call_indirect` 用テーブル
- `entry<...>`: 実行エントリポイント

関数は概ね `params`, `resultTypes`, `locals`, `instructions` を持つ。
instruction は object literal 的な discriminated union へ落とされる。

### 2.2 bootstrap

`entry<...>` は `bootstrap<>` を呼び、実行時の初期状態を組み立てる。

bootstrap の責務:

- 外部引数を WASM value 表現へ変換する
- entry function の引数を locals へ束縛する
- 初期 `ProgramState` を作る
- entry function の instruction stream 末尾に synthetic end instruction を差し込む
- `executeInstruction<>` に制御を渡す

### 2.3 runtime loop

`executeInstruction<>` は 1 命令ずつ `ProgramState` を変換する。

停止条件:

- `instructions` が空になる
- synthetic `Halt` に到達する
- `count == stopAt` になる

終了時は `State.Result.finish<>` が走り、
必要なら final GC を行ったうえで stack から戻り値を抽出する。

## 3. Runtime の中核構造

runtime の中心は `ProgramState`。
DeepWiki 上の整理では概ね次を持つ。

- `count`
- `stack`
- `activeFuncId`
- `activeStackDepth`
- `activeLocals`
- `instructions`
- `activeBranches`
- `L1Cache`
- `memory`
- `executionContexts`
- `funcs`
- `garbageCollection`
- `globals`
- `memorySize`
- `indirect`
- `results`
- `callHistory`

`selectInstruction<>` が命令種別に応じて handler を振り分ける。
命令カテゴリは概ね次のように分かれている。

- const
- variable
- arithmetic
- bitwise
- comparison
- conversion
- memory
- control flow
- synthetic

## 4. 制御フローと call stack

制御フローの要点は、WASM の block/loop/branch を
instruction queue の書き換えと synthetic instruction で表現していることにある。

### 4.1 executionContexts

関数呼び出しの call stack は `executionContexts` で管理される。
各 context は概ね次を保持する。

- caller locals
- caller function id
- caller branch table
- caller stack depth
- caller continuation

`call` 時は現在の continuation を push し、
`return` または `EndFunction` 時に pop して caller 側へ復帰する。

### 4.2 activeBranches

`block`, `loop`, `br`, `br_if`, `br_table` は `activeBranches` で扱う。

- `block`: branch target 用 continuation を保存し、末尾に `EndBlock` を挿入
- `loop`: loop 先頭へ戻る continuation を保存し、末尾に `EndLoop` を挿入
- `br`: target label に対応する instruction list へ差し替え
- `br_if`: stack top の条件で分岐する
- `br_table`: index に応じて target continuation を選ぶ

### 4.3 synthetic instructions

内部補助として次の synthetic instruction を使う。

- `synth.end_block`
- `synth.end_func`
- `synth.end_loop`
- `synth.halt`

これは WebAssembly 標準の opcode ではなく、
型レベル実行系を閉じた形で回すための runtime internal である。

## 5. Memory と "GC"

この repo の memory system は 2 層構造。

- `L1Cache`: 最近の write を保持する一時領域
- `memory`: 永続的な main memory

基本方針:

- store はまず `L1Cache` に書く
- load はまず `L1Cache` を見て、なければ `memory` を見る
- 一定周期で `L1Cache` を `memory` に merge する

### 5.1 GC の意味

ここでいう garbage collection は、
WASM プログラム上の object graph を辿る heap GC ではない。

そうではなく、

- 型レベル `Record<Address, Byte>` が膨れ続ける
- TypeScript compiler が巨大な type を抱えて破綻しやすくなる

という問題を避けるための compile-time cleanup である。

DeepWiki 上の説明では `SweepL1Every = 1024`。
また、終了時にも forced collect が走る。

この設計は TypeScript compiler の制約への適応として読むべきであり、
WASM runtime の意味論上必須の機能とは限らない。

## 6. 出力と描画

generic runtime は「画面を描く」のではなく、
最終 `ProgramState` と stack/memory を返すだけである。
output extraction はその後段で行われる。

### 6.1 数値結果

最も generic なケース。

- 関数の戻り値が stack 上の scalar value
- `State.Result.finish<>` が number / bigint へ戻す

### 6.2 文字列結果

`ReadStringFromMemory` が使われる。

前提:

- 戻り値は string 自体ではなく pointer
- pointer は final state の `stack[0]`
- memory 上の文字列は null terminated
- 各 byte は ASCII として読める

つまり、C 由来の「char* を返す」系の出力に向いた generic reader である。

## 追記: `typelude-wasm` frontend parity メモ

`typelude-wasm` の `twat!` は parser-only 方針を維持しつつ、runtime が既に持っている
整数系 opcode と memory/control の大半に追従し始めている。

現時点で frontend parity に入った代表例:

- `br_table`
- `nop`, `unreachable`
- `i32` の compare / bitwise / shift / `mul/div/rem` / sign-extension / `wrap_i64`
- `i32.load8_s/load16_s/load16_u/store16`
- `i64.clz/ctz/popcnt/rotl/rotr/extend_i32_*`
- `i64.load8/16/32_*`, `i64.store8/16/32`
- `f32.reinterpret_i32`, `f64.reinterpret_i64`, `i64.reinterpret_f64`

未対応の主眼は引き続き float arithmetic / compare、multi-value、一般 reference types、
passive/declarative segment、その他 runtime 未実装領域である。

### 6.3 Doom の描画

Doom は generic output reader ではなく、専用 renderer を使う。

DeepWiki の整理では、
`packages/playground/final-doom-pun-intended/data/process-frame.ts`
の `MeetYourDoom` が次の処理を行う。

- base address として `state['stack'][0]` を使う
- そこから frame buffer を読む
- 320 byte 幅の scanline を 200 行ぶん処理する
- 各 byte を `DoomPaletteToAscii` で ASCII に写像する
- 最終的に ASCII frame を得る

重要なのは、これは WASM runtime の命令実行ロジックではなく、
final `ProgramState['memory']` を読む post-processing layer だということである。

### 6.4 Conway

Conway 系は Doom より generic reader 寄りで、
WASM プログラム側が最終文字列を memory に書き、
pointer を返して `ReadStringFromMemory` で読む流れが中心らしい。

したがって、repo 全体で見ると output extraction のパターンは次の 2 つに整理できる。

1. generic scalar/string reader を使う
2. アプリ専用 renderer で final memory を解釈する

汎用の高レベル reader は少なく、
実質的には `ReadStringFromMemory` が唯一の generic memory reader と見てよい。

## 7. Evaluation Playground

この repo は「型レベル VM 本体」とは別に、
巨大な type computation を現実に回すための tooling を持っている。

中心ディレクトリ:

- `packages/playground/evaluate/config.ts`
- `packages/playground/evaluate/run.ts`
- `packages/playground/evaluate/ts.ts`
- `packages/playground/evaluate/utils.ts`
- `packages/playground/evaluate/stats.ts`
- `packages/playground/evaluate/spawn.ts`

### 7.1 増分実行

1 回の型評価で最後まで走り切ろうとはせず、
`NextResult` を一定命令数ぶんだけ進めて checkpoint 化する。

概念的には次のループになっている。

```text
start.ts
  -> evaluate NextResult
  -> typeToString で state を文字列化
  -> 次の result-XXXXXXXX.ts を生成
  -> 再度 evaluate
```

この方式で TypeScript compiler のメモリ圧力と call stack 問題を避けている。

### 7.2 生成される artifact

DeepWiki 上の整理では概ね次のファイルが出る。

- `start.ts`
- `results/result-00000000.ts` のような増分 checkpoint
- `stats/stats-00000000.json`
- `stats/program-stats.json`
- `stats/program-stats.csv`
- `results/error.ts`

長時間実行時は `--resume` で最新 checkpoint から再開できる。
さらに必要なら別 process へ切り替えて continuation を継ぐ。

### 7.3 finalization

最後の state に対しては別の final TS file を組み立て、
用途に応じて別の type alias を評価する。

- raw result を見る
- `ReadStringFromMemory` を適用する
- Doom なら `MeetYourDoom` のような専用 renderer を適用する

つまり finalization は「VM の終了」ではなく、
「終了 state をどう読むか」の層である。

## 8. TypeScript compiler patch

この repo は vanilla TypeScript だけでは成立しにくく、
`patches/typescript@5.6.3.patch` による compiler patch を前提としている。

DeepWiki 上では次の調整が説明されていた。

- type instantiation depth 制限の大幅緩和
- tuple size 制限の拡張
- type comparison depth 制限の緩和
- type error 表示 truncation の上限拡大
- instantiation count の収集用 logging

これは runtime 自体の意味論というより、
「TypeScript compiler を evaluator として使うための実行基盤」である。

## 9. `typelude-wasm` への示唆

`typelude` 側でこの設計を参照するなら、
そのまま移植するのではなく、次のように層を切り分けた方がよい。

### 9.1 まず必要な核

最小核は次で十分。

- `ProgramState` 相当の machine state
- instruction queue
- stack
- locals
- frames
- branch labels / continuations
- `executeInstruction` 相当の small-step 実行

opcode は最初から全部やる必要はなく、
次の subset でよい。

- `i32.const`
- `local.get`
- `local.set`
- `i32.add`
- `i32.sub`
- `i32.eqz`
- `block`
- `loop`
- `br_if`
- `call`
- `return`

### 9.2 後段に回してよいもの

以下は第 2 段階以降でよい。

- `call_indirect`
- `br_table`
- `memory.grow`
- `memory.size`
- full load/store family
- 64bit arithmetic の網羅
- bespoke renderer

### 9.3 GC は最初から不要

TypeScript 側の `L1Cache + GC` は
TypeScript compiler の type explosion 対策の色が強い。

Rust の型レベル実装では、まず素直な memory 表現から始め、
trait solving や recursion limit の都合で必要になってから
正規化・圧縮・sweep を考える方が自然である。

### 9.4 renderer を runtime に混ぜない

特に重要なのはここである。

- generic runtime は WASM の意味論だけを持つ
- 描画や画面化は final state の reader として別層に置く

この分離を守ると、
Conway のような string-returning program と
Doom のような framebuffer-returning program を
同じ core runtime 上で扱える。

## 10. 今回の調査で見えた全体像

この repo を一言で言うと、
「型レベル WebAssembly runtime」そのものより、
「型レベル WebAssembly runtime を成立させる周辺基盤込みの system」である。

構造としては次の 4 層に分けて理解するのがよい。

1. Rust frontend による WASM -> TS types 変換
2. `ProgramState` と `executeInstruction<>` による generic runtime
3. `ReadStringFromMemory` や `MeetYourDoom` による output extraction
4. 増分実行、checkpoint、resume、stats、patched TS compiler からなる evaluation environment

`typelude-wasm` を作る際も、
いきなり full feature を目指すよりこの層構造を守る方がよい。

## 11. `typelude-wasm` の今後の対応ロードマップ

現在の `typelude-wasm` は、
runtime 側では `i32 + locals + direct call + structured control flow + linear memory + globals + memory.grow + active data init`
あたりまで先行している一方、
`wasm_wat!` frontend はそこまで追従していない。

したがって、次の順で進めるのが自然である。

### 11.1 Phase 1: 軽量な frontend 追従

まずは runtime に既にある機能を
`wasm_wat!` から end-to-end で使えるようにする。

対象:

- `memory.grow` の WAT lowering
- nonzero `offset` を持つ `i32.load/store/load8_u/store8` の lowering
- `global` section の lowering
- active `data` section の lowering
- `start` section の lowering

この phase の主眼は frontend 側の gap を埋めることであり、
新しい machine semantics を大きく増やすことではない。

完了条件:

- `memory.grow` を含む WAT が `wasm_wat!` 経由で動く
- `offset != 0` の load/store を lower できる
- global/data/start を含む module を lower できる
- 非対応な data/init/start 形は compile error で固定する

### 11.2 Phase 2: tables と `call_indirect`

次に入れるべきは tables と indirect call。
これは frontend だけの話ではなく、
table 初期化、slot lookup、type check を runtime 側で実証する必要がある。

対象:

- table store の handwritten runtime test
- active elem segment
- `OpCallIndirect<TypeIdx, TableIdx>`
- `call_indirect` の WAT lowering
- table export の IR 保持

完了条件:

- active elem segment で function index が table に載る
- `call_indirect` が signature 一致時だけ動く
- null slot / table OOB / type mismatch は compile error のまま固定する
- indirect call 付き WAT を lower して runtime/oracle で一致する

### 11.3 Phase 3: imports

imports は module instantiation 全体に触るため、
tables より先に入れるより、table/call_indirect の整理後に入れる方がよい。

対象:

- `import` section の WAT lowering
- typed host env への binding
- imported function / global / memory / table の instantiation
- data/global init で imported immutable global を使えるようにする

完了条件:

- unresolved import は compile error にする
- host function call が runtime/oracle で一致する
- imported immutable global を offset/init expr で使える
- imported memory/table の最小ケースが通る

### 11.4 Phase 4: export API の本格化

現在の実行面は function index 寄りなので、
module 機能を本物の WASM module に寄せるには name-first の export API が必要になる。

対象:

- `InvokeExport` を function export で安定化
- global export / table export の解決 API
- wrong-kind export の compile-time rejection

完了条件:

- export 名から function/global/table を型レベルで解決できる
- function 以外を `InvokeExport` に渡した場合は compile error にする
- public integration test が index だけでなく name-first でも通る

### 11.5 Phase 5: `i64`

整数系の次段としては `i64` が自然。
`i32` の拡張として扱いやすく、module 機能とも独立して進めやすい。

対象:

- `WasmI64`
- `i64.const`
- `local/global/load/store` の最小セット
- `add/sub/mul/eqz/eq/ne` から順次拡張

完了条件:

- i64 locals/calls/memory round-trip
- WAT lowering が i64 subset に追従する
- `wasmi` oracle と一致する

### 11.6 Phase 6: `f32`, `f64`

浮動小数は最後に回すのがよい。
これは単に型を増やす話ではなく、
NaN、signed zero、bitpattern fidelity をどう持つかの問題になる。

対象:

- `WasmF32`, `WasmF64`
- `const/load/store`
- `eq/ne/lt/gt/...` の spec 準拠

完了条件:

- bitpattern としての round-trip が成立する
- NaN / signed zero の挙動を固定したテストが通る
- `wasmi` oracle と最小 subset で一致する

### 11.7 対応順の結論

実装順としては次が妥当である。

1. `memory.grow` の WAT lowering
2. nonzero offset の load/store lowering
3. `global` / `data` / `start` section の WAT lowering
4. table / elem / `call_indirect`
5. imports
6. table/global export API
7. `i64`
8. `f32` / `f64`

この順序を取る理由は単純で、
既に runtime にある機能をまず frontend から使えるようにし、
その後に module 構造そのものを厚くし、
最後に新しい value kind を増やす方が手戻りが少ないからである。

## Sources

- GitHub: <https://github.com/MichiganTypeScript/typescript-types-only-wasm-runtime>
- DeepWiki runtime architecture:
  <https://deepwiki.com/search/explain-the-runtime-architectu_eef21547-2ac5-4c74-ac0a-0bc8b90eea6b>
- DeepWiki GC clarification:
  <https://deepwiki.com/search/clarify-the-meaning-of-garbage_3b852309-d679-4572-9453-fc553bfcd7bb>
- DeepWiki control flow / branches:
  <https://deepwiki.com/search/explain-how-blockloopbranchbri_ce57007a-3836-4f96-9435-92ebd5d7d1e2>
- DeepWiki frontend lowering:
  <https://deepwiki.com/search/how-does-the-rust-compiler-fro_c38c1c48-9f32-4497-96a6-a80e189b459e>
- DeepWiki compiler patches / tooling:
  <https://deepwiki.com/search/what-typescript-compiler-patch_ba5b39e7-4ebd-4f95-bb14-405a41782b14>
- DeepWiki end-to-end pipeline:
  <https://deepwiki.com/search/describe-the-endtoend-executio_fb3c16a7-b046-44a0-a351-2107306067de>
- DeepWiki Doom pipeline:
  <https://deepwiki.com/search/explain-the-doom-application-p_ae1c93f2-c40d-4fe1-912c-250f95b97a5e>
- DeepWiki Doom frame extraction:
  <https://deepwiki.com/search/in-the-doom-pipeline-explain-e_b3d650f7-8474-46ed-84f7-35d294054ca9>
- DeepWiki Conway pipeline:
  <https://deepwiki.com/search/explain-the-conways-game-of-li_01b34506-eb87-42eb-b2eb-d1d27a91aee8>
- DeepWiki evaluation playground:
  <https://deepwiki.com/search/explain-the-evaluation-playgro_f1e0828e-5a39-4d8f-81a2-e00441290152>
- DeepWiki finalization pipeline:
  <https://deepwiki.com/search/explain-the-finalization-pipel_261d7b8c-ded6-4023-a8d5-4ce1c4b4eca6>
- DeepWiki disk artifacts / checkpoints:
  <https://deepwiki.com/search/what-are-the-concrete-artifact_2141008f-761f-4c52-b3c4-b68bc5d2e85a>
- DeepWiki `ReadStringFromMemory`:
  <https://deepwiki.com/search/explain-how-readstringfrommemo_9befd2d6-063f-4f88-920b-71b5b7d6b2e1>
- DeepWiki output readers:
  <https://deepwiki.com/search/are-there-generic-reusable-out_43487d82-09c3-4105-a71a-f07b0fc612d1>
