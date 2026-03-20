# Rustの型システムで「計算」と「制御」を：BoolからVMのステートマシンまで

前作はこちら
<https://zenn.dev/oumi0804/articles/450d54afafa303>

突然ですがみなさん、TypeScriptの型システムはDOOMを動かすことができます。
（WASM相当のスタックマシンを型レベルで実装し、DOOMのWASM版を動かしている）

<https://www.youtube.com/watch?v=0mCsluv5FXA>

ところで、**Rustの型システムはチューリング完全です。**

<https://sdleffler.github.io/RustTypeSystemTuringComplete/>

さて、Rustの型システムでもDOOMを動かしましょう。

:::message alert
ネタバレ: DOOMは動かせません
:::
:::message
この記事はRustの型システムで基本的な計算と制御を行うところまでを解説します。
:::

## 1. `impl` による分岐と再帰：基本のキ

Rustの型システム（トレイト境界）におけるプログラミングの最小単位は、`impl` による「特異化（分岐）」と、関連型（Associated Types）による「結果の返却」です。

### `impl` で型レベルの分岐ができる

「もし型がAならこの処理、Bならこの処理」という分岐は、異なる型に対するトレイトの実装で表現できます。

```rust
trait IsZero {
    type Output;
}

impl IsZero for U0 {
    type Output = True;
}

impl<U, B> IsZero for UInt<U, B> {
    type Output = False;
}
```

### 関連型とトレイト連鎖による再帰

「自身の次のステップ」を関連型に持たせ、それを再帰的に呼び出すことで、型レベルのループや再帰を実装できます。

```rust
trait Len {
    type Output;
}

impl Len for Nil {
    type Output = U0;
}

impl<H, T: Len> Len for Array<H, T> {
    type Output = Add1<T::Output>; // トレイト連鎖による再帰
}
```

## 2. 実装例：Bool演算と `If`

まずは基本となる `Bool` と、それを使った条件分岐 `If` を見てみましょう。

```rust
pub trait Bool {
    type Not;
    type And<Rhs: Bool>;
}

impl Bool for True {
    type Not = False;
    type And<Rhs: Bool> = Rhs;
}

impl Bool for False {
    type Not = True;
    type And<Rhs: Bool> = False;
}
```

これを使えば、`If` 式も「条件型」によるディスパッチで実装できます。

```rust
pub trait IfHelper<Then, Else> {
    type Output;
}

impl<Then, Else> IfHelper<Then, Else> for True {
    type Output = Then;
}

impl<Then, Else> IfHelper<Then, Else> for False {
    type Output = Else;
}

// ユーザー向けエイリアス
type If<Cond, Then, Else> = <Cond as IfHelper<Then, Else>>::Output;
```

## 3. 【転換点】Nand2Tetrisの夢と「トレイト解決の壁」

ここで一つ、私の個人的な「挫折」のエピソードを紹介します。
当初、私は **Nand2Tetris** のように、最小の `Nand` ゲートからすべての論理演算を構築しようとしました。

```rust
trait Nand<Lhs: Bool, Rhs: Bool> { type Output; }
impl Nand<True, True> for () { type Output = False; }
// ...他の3パターン

trait Not<T: Bool> { type Output; }
impl<T: Bool> Not<T> for ()
where (): Nand<T, T> {
    type Output = <() as Nand<T, T>>::Output;
}

trait And<Lhs: Bool, Rhs: Bool> { type Output; }
impl<Lhs: Bool, Rhs: Bool> And<Lhs, Rhs> for ()
where
    (): Nand<Lhs, Rhs>,
    <() as Nand<Lhs, Rhs>>::Output: Bool, // ← これが必要
    (): Not<<() as Nand<Lhs, Rhs>>::Output>,
{
    type Output = <() as Not<<() as Nand<Lhs, Rhs>>::Output>>::Output;
}
```

一見綺麗ですが、実際に `And` や `Or` を組み合わせていくと、Rustコンパイラから「`<() as Nand<Lhs, Rhs>>::Output` が `Bool` であることを証明せよ」といった要求が次々と飛んできます。

これがいわゆる **「制約爆発 (Constraint Explosion)」** です。
演算を一つ増やすごとに `where` 句が指数関数的に増え、最終的に人間（とコンパイラ）の理解を超えてしまいます。

「トレイト解決が無理だ……」と悟った私がたどり着いたのが、計算を「遅延」させる **`Eval` パターン** でした。

```rust
/// 「計算可能なもの」を表す統一インターフェース
pub trait Eval {
    type Output;
}

/// 計算の適用（関数呼び出し相当）をラップする
pub struct EApp<F, A>(PhantomData<(F, A)>);

impl<F, A> Eval for EApp<F, A>
where
    F: TyFn<A> // ラップされた関数
{
    type Output = F::Output;
}
```

計算を直接行うのではなく、「計算を表現する型」を構築し、最後に `Evaluate<T>` で一気に解決することで、中間状態の凄まじいトレイト境界を隠蔽できます。

## 4. 高度な計算：Array と Typenum

`Eval` パターンを基盤にすれば、より高度な操作が可能になります。

### Array (Cons List)

型レベルのリストは `Array<Head, Tail>` の再帰構造で定義します。これに対して `EMap`, `EFilter`, `EFold` といった高階関数を実装できます。

```rust
// 型レベルMapの実装イメージ
impl<Op, H, T> Eval for EMap<Op, Array<H, T>>
where
    Op: TyFn<H>,
    T: Eval, // 遅延評価
{
    type Output = Array<Evaluate<EApp<Op, H>>, Evaluate<EMap<Op, T>>>;
}
```

### Typenum との統合

数値計算をゼロから実装するのは大変なので、既存の `typenum` クレートをラップして `Eval` エコシステムに組み込みます。これにより `EAdd<U1, U2>` のような自然な記述が可能になります。

## 5. 制御の極致：`While` とステートマシン

最後に、型レベルで「状態」を更新し続けるループと、それを用いたVMの実装です。

### `EWhile` の魔法

`While` は「条件が `False` になるまで `Step` 関数を適用し続ける」再帰トレイトで表現されます。

```rust
trait WhileHelper<Pred, Step, State> {
    type Output;
}

// 終了条件
impl<P, S, State> WhileHelper<P, S, State> for False {
    type Output = State;
}

// 継続条件
impl<P, S, State> WhileHelper<P, S, State> for True
where
    NextState = Evaluate<EApp<S, State>>,
    NextCond = Evaluate<EApp<P, NextState>>,
    NextCond: WhileHelper<P, S, NextState>
{
    type Output = <NextCond as WhileHelper<P, S, NextState>>::Output;
}
```

### 簡易ステートマシン（VM）

この `While` を使って、VMの「状態（Stack, Program, Memory）」を更新していくことで、型システム上でのプログラム実行が実現します。

```rust
struct VmState<Stack, Program, Memory>;

trait StepInstr<State> {
    type Output; // Next State
}

// 命令を一つ実行して、次の VmState を返す
impl<S, P, M> StepInstr<VmState<S, P, M>> for MyOpCode { ... }
```

## おわりに

Rustの型システムでのプログラミングは、パズルのような楽しさ（と絶望）があります。
実用性は……正直薄いですが、言語の限界に挑むのはエンジニアの性というものでしょう。

次回はいよいよ、この基盤の上で実際に動く「何か」を作っていきたいと思います。
（DOOMは動かなくても、簡単な数式や論理演算なら型チェック時に「実行」される快感を味わえます）
