# CLAUDE.md — typelude

Rustの型システムで型レベル計算を行うフレームワーク。型をファーストクラス値として扱い、コンパイル時に任意の計算（ループ・再帰・スタックマシン実行）を実現する。

## コマンド

```bash
# 全クレートのビルド・テスト
cargo test --workspace

# 特定クレート
cargo test -p typelude-std
cargo test -p typelude-vm
cargo test -p typelude-macros

# ドキュメント生成
cargo doc --workspace --no-deps --open

# nightly feature 有効化
cargo test --workspace --features nightly
```

## クレート構成

```text
typelude/              # 公開APIファサード。他3クレートの再エクスポート
typelude-std/          # 型レベルプリミティブ・標準ライブラリ
  core/                # Eval, TyFn, ELit, EApp (評価の基盤)
  model/               # データ型定義 (True/False, Array, Nil)
  std/                 # 演算・コレクション・制御フロー (EIf, EWhile, EGet...)
typelude-vm/           # 型レベルスタックマシン
  opcode/              # 命令定義 (OpPush, OpAdd, OpIf, OpWhile...)
  vm/semantics/        # 純粋な小ステップ実行意味論
  vm/runtime/          # エフェクト付き実行ループ (モナド変換子スタック)
  core/                # モナドトレイト (StateT, WriterT, EitherT, SuspendT)
typelude-macros/       # プロシージャルマクロ
  program!             # スタックマシンDSL → 型式への展開
  ty_fn!               # TyFn + Eval の実装生成
```

## 設計の核心: Evalパターン

型レベル計算の統一インターフェース。

```rust
pub trait Eval { type Output; }
pub type Evaluate<T> = <T as Eval>::Output;
```

**型エイリアスではなくEvalパターンを使う理由:**

- 型エイリアスは「透明」— ジェネリクスと組み合わせると制約が呼び出し側に漏れる（制約爆発）
- `impl Eval for EFoo<A, B>` の中に制約を封じ込めることで、外からは `EFoo<A, B>: Eval` の一言で済む
- 型エイリアスでは再帰型が書けない（`type Fib<N> = Add<Fib<Pred<N>>, ...>` はコンパイルエラー）

## 命名規則

| パターン | 意味 | 例 |
|---|---|---|
| `E` prefix | 評価可能な式の型 (Eval型) | `EIf`, `EAdd`, `EGet`, `EWhile` |
| `Op` prefix | VMのオペコード | `OpPush`, `OpAdd`, `OpWhile` |
| `XxxHelper` | dispatch用の内部ヘルパートレイト | `EIfHelper`, `EWhileHelper`, `GetHelper` |
| 素の名前 | データ型・ケイパビリティトレイト | `True`, `False`, `Array`, `Nil`, `Bool`, `IsList` |

## 重要な設計パターン

### Helper トレイトパターン

`True`/`False` のような型でdispatchするとき、overlapping implを避けるために中間ヘルパーを挟む。

```rust
pub trait EIfHelper<Then, Else> { type Output; }
impl<T: Eval, E: Eval> EIfHelper<T, E> for True  { type Output = Evaluate<T>; }
impl<T: Eval, E: Eval> EIfHelper<T, E> for False { type Output = Evaluate<E>; }

impl<Cond, Then, Else> Eval for EIf<Cond, Then, Else>
where
    Cond: Eval,
    Evaluate<Cond>: EIfHelper<Then, Else>,
{ ... }
```

### 再帰はrecursive trait boundで表現する

```rust
impl<Pred, Step, State> EWhileHelper<...> for True
where
    EWhile<Pred, Step, Evaluate<EApp<Step, State>>>: Eval, // ← 再帰
{ ... }
```

### PhantomDataで型情報を保持（ランタイムコストゼロ）

```rust
pub struct EIf<Cond, Then, Else>(PhantomData<(Cond, Then, Else)>);
```

## Nightly Features

`feature = "nightly"` フラグで有効化：

| Feature | 用途 |
|---|---|
| `specialization` | より柔軟なimpl特殊化 |
| `generic_const_exprs` | const genericsの式評価 |

## recursion_limit

`typelude-std`, `typelude-vm` は `#![recursion_limit = "65536"]` を設定している。型レベルの再帰計算（特にEWhile・Fibonacci等）はRustcのデフォルト限界を超えるため。

## 外部クレートへの依存

| クレート | 用途 |
|---|---|
| `typenum` | 型レベル自然数 (`U0`, `U1`, ...) |
| `tstr` | 型レベル文字列 |
| `static_assertions` | テストでの `assert_type_eq_all!` |
| `paste` | マクロ内のトークン結合 |

## テストの書き方

型レベルの等価性は `static_assertions::assert_type_eq_all!` で検証する。

```rust
use static_assertions::assert_type_eq_all;
assert_type_eq_all!(Evaluate<EAdd<ELit<U3>, ELit<U5>>>, U8);
assert_type_eq_all!(Evaluate<EIf<ELit<True>, ELit<u32>, ELit<String>>>, u32);
```

## docs/ の構成

```text
docs/
  rfcs/              # 設計提案 (0004: Collections, 0005: DX, 0006: Primitives)
  article_draft.md   # Zenn記事の叩き台
  journal/           # 開発ジャーナル
```
