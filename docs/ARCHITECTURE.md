# Architecture

typeludeのクレート構成と内部設計の詳細。

## クレート依存関係

```
typelude          (公開ファサード)
  ├── typelude-std
  ├── typelude-vm
  │     └── typelude-std
  └── typelude-macros
        └── typelude-std
```

## typelude-std

型レベルプログラミングの基盤となるプリミティブと標準ライブラリ。

### モジュール構成

```
core/
  eval.rs      # Eval トレイトと Evaluate 型エイリアス
  fn.rs        # TyFn<Arg> トレイト
  expr.rs      # ELit<T>, EApp<F, A>
  col.rs       # ENil, ECons, EList（式のリスト）

model/
  prim/
    bool.rs    # True, False, IsBool
    int.rs     # 整数値の型
    str.rs     # 文字列値の型
    option.rs  # SomeVal, NoneVal
  col/
    array.rs   # Array<Head, Tail>, Nil, IsList

std/
  prim/
    bool.rs    # Bool トレイト（Not/And/Or/Xor/Nand...の関連型）
    int.rs
    str.rs
    option.rs
  ops/
    logic.rs   # ENot, EAnd, EOr, EXor, ENand, ENor, EXnor
    arith.rs   # EAdd, ESub, EMul, EDiv, ERem, EPow
    cmp.rs     # EEq, ENeq, ELt, EGt, ELe, EGe
  col/
    array.rs   # EGet, ESet, ELen, EHead, ETail, EMap, EFilter, EFold...
    map.rs
    tree_array.rs
    tree_map.rs
  control.rs   # EIf, EWhile
  reify.rs     # Reify トレイト（型レベル → ランタイム値）
  interop.rs   # IntoCoreBool, IntoCoreNat（外部型の正規化）
  traits.rs    # Nat, Int, Bool, TAdd, TMul...
  debug/
    trace.rs   # Trace トレイト（デバッグ用）
```

### 設計の6層モデル

| 層 | 役割 | 例 |
| --- | --- | --- |
| 1. Values | データ型定義 | `True`, `False`, `Array<H,T>`, `Nil` |
| 2. Capabilities | 能力を表すトレイト | `IsBool`, `IsList`, `Bool`, `Len`, `Get` |
| 3. Backends | dispatch用ヘルパー | `EIfHelper`, `GetHelper`, `MapHelper` |
| 4. Evaluator | Evalの実装 | `impl Eval for EIf<...>` |
| 5. Macros | ボイラープレート削減 | `define_arith_op!`, `ty_fn!` |
| 6. Aliases | ユーザー向け公開型 | `EAdd`, `EOr`, `EGet`, `EWhile` |

## typelude-vm

型レベルスタックマシンの実装。意味論とエフェクトを分離した設計。

### モジュール構成

```
opcode/
  stack.rs     # OpPush, OpPop, OpDup, OpSwap, OpDrop
  numeric.rs   # OpAdd, OpSub, OpLt, OpGt, OpEq, OpNot...
  control.rs   # OpIf, OpWhile, OpCall, OpReturn
  local.rs     # OpLet, OpGetLocal, OpSetLocal, OpDropLocal
  memory.rs    # OpLoad, OpStore
  host.rs      # OpHostCall

vm/
  semantics/   # 純粋な小ステップ意味論（エフェクトなし）
    step.rs    # 1命令の実行
    state.rs   # VmState（スタック・ローカル・メモリ・フレーム）
    instr/     # 各命令カテゴリの意味論
    helpers/   # 値取り出し・条件判定・インデックス操作

  runtime/     # エフェクト付き実行ループ
    run.rs     # ERunVm（メインエントリポイント）
    effects/   # スタック・状態・トレース・例外・I/O
    outcome.rs # Done / Raised / Suspended

  protocol/    # VMインターフェース契約
    request.rs # HostRequest, HostSignature
    trap.rs    # 例外・トラップ
    trace_event.rs # 実行トレース

core/          # モナド変換子（エフェクト合成）
  id.rs        # Identity モナド
  state_t.rs   # StateT（状態管理）
  writer_t.rs  # WriterT（トレース蓄積）
  either_t.rs  # EitherT（エラーハンドリング）
  suspend_t.rs # SuspendT（サスペンション/レジューム）
  traits.rs    # Monad, MonadState, MonadError, MonadWriter, MonadSuspend
```

### 純粋意味論とエフェクトの分離

```
semantics/（純粋）        runtime/（エフェクト付き）
  ↓ step: State → Result    ↓ run: Program → Outcome
  決定的・副作用なし         トレース・例外・ホスト呼び出し
  型レベルの関数             モナド変換子スタックで合成
```

エフェクトスタック:

```
SuspendT<            ← ホスト呼び出しで中断/再開
  EitherT<Trap,      ← トラップ・例外
    WriterT<Trace,   ← 実行トレースの蓄積
      StateT<State,  ← VMの状態
        Id>>>>       ← 恒等モナド（基底）
```

### オペコードの実行フロー

1. `ERunVm<Program, State>` が評価される
2. `vm/runtime/run.rs` がプログラムを1命令ずつ処理
3. 各命令は `vm/semantics/step.rs` で純粋に評価（`StepResult` を返す）
4. `StepResult` に応じてエフェクト層が状態・トレース・中断を処理
5. `Done`, `Raised`, `Suspended` のいずれかで終了

### ホスト呼び出しとサスペンション

`OpHostCall` は VMを中断し `HostRequest` を外部に返す。
ホストからのレスポンスを受け取ってVMを再開できる（レジューム）。

```rust
// 中断
type Suspended = Evaluate<ERunVm<ProgramWithHostCall, InitState>>;
// → Suspended = SuspendedVm<Request, RemainingProgram, State>

// 再開（ホストがレスポンスを型として渡す）
type Resumed = Evaluate<EResumeVm<Suspended, HostResponse>>;
```

## typelude-macros

プロシージャルマクロの実装。

### 主要マクロ

| マクロ | 役割 |
| --- | --- |
| `program!` | スタックマシンDSL → 型式への展開 |
| `ty_fn!` | `TyFn` + `Eval` の実装を同時生成 |
| `impl_eval!` | `Eval` 実装の生成 |
| `bound!` | trait bounds の簡潔な記述 |

### program! マクロのDSL

```rust
type Prog = program! {
    (push U10)       // OpPush<ELit<U10>>
    (let)            // OpLet<...>
    (get_local U0)   // OpGetLocal<U0>
    (push U1)
    (add)            // OpAdd
    (while Cond Body) // OpWhile<Cond, Body>
};
```

`~T` 記法はマクロ内で「Tを評価した型」を意味する（`Evaluate<T>` の糖衣構文）。

## typelude（ファサード）

他3クレートの再エクスポートのみ。外部向け公開APIの窓口。

```rust
pub use typelude_std::{Eval, Evaluate, std, tyarray};
pub use typelude_macros::program;
pub use typelude_vm as vm;
pub use typenum;
```

ユーザーは `typelude` だけ依存すれば他のクレートを個別にimportする必要がない。
