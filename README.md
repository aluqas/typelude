# typelude

Rustの型システムで型レベル計算を行うフレームワーク。型をファーストクラス値として扱い、コンパイル時に任意の計算（条件分岐・再帰・ループ・スタックマシン実行）を実現する。

```rust
// Fibonacci(10) をコンパイル時に計算
type Result = Evaluate<ERunVm<FibProgram10, InitVmState>>;
// Result = U55 がコンパイル時に検証される
```

## クレート構成

| クレート          | 役割                                 |
| ----------------- | ------------------------------------ |
| `typelude`        | 公開APIファサード・再エクスポート    |
| `typelude-std`    | 型レベルプリミティブ・標準ライブラリ |
| `typelude-vm`     | 型レベルスタックマシン               |
| `typelude-macros` | DSL用プロシージャルマクロ            |

## 使い方

### 基本: 型の評価

```rust
use typelude::{Eval, Evaluate, std::prim::bool::{True, False}};
use typenum::{U3, U5};

// 型レベルの加算
type Sum = Evaluate<EAdd<ELit<U3>, ELit<U5>>>; // = U8

// 型レベルの条件分岐
type R = Evaluate<EIf<ELit<True>, ELit<u32>, ELit<String>>>; // = u32
```

### Bool演算

```rust
use typelude::std::prim::bool::{True, False};
use typelude::std::ops::logic::{ENot, EAnd, EOr};

type NotTrue   = Evaluate<ENot<ELit<True>>>;           // = False
type TrueOrFalse = Evaluate<EOr<ELit<True>, ELit<False>>>; // = True
```

### 型レベルArray

```rust
use typelude::{tyarray, std::col::array::{EGet, ESet, ELen}};
use typenum::{U0, U1, U2};

type Arr = tyarray![U10, U20, U30];
type Len = Evaluate<ELen<Arr>>;          // = U3
type Elem = Evaluate<EGet<Arr, U1>>;     // = U20
type Updated = Evaluate<ESet<Arr, U1, U99>>;
```

### スタックマシン

```rust
use typelude::{vm, program};

type Prog = program! {
    (push U10)
    (push U20)
    (add)         // スタックトップ: U30
};
```

## 設計の核心: Evalパターン

```rust
pub trait Eval { type Output; }
pub type Evaluate<T> = <T as Eval>::Output;
```

型レベル計算を「式の型」として表現し、`Eval` トレイトの実装として記述する。これにより制約の爆発を防ぎ、再帰的な型レベル計算を実現する。詳細は `CLAUDE.md` を参照。

## ビルド

```bash
cargo test --workspace

# nightly features (specialization, generic_const_exprs)
cargo test --workspace --features nightly
```

## ドキュメント

- [`CLAUDE.md`](./CLAUDE.md) — 設計の詳細・命名規則・開発ガイド
- [`docs/rfcs/`](./docs/rfcs/) — 設計提案
- [`crates/typelude-vm/docs/vm-semantics.md`](./crates/typelude-vm/docs/vm-semantics.md) — VMの意味論
