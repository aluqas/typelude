前作はこちら
<https://zenn.dev/oumi0804/articles/450d54afafa303>

突然ですがみなさん、TypeScriptの型システムはDOOMを動かすことができます。
（WASM相当のスタックマシンを型レベルで実装し、DOOMのWASM版を動かしている）

<https://www.youtube.com/watch?v=0mCsluv5FXA>

ところで、**Rustの型システムはチューリング完全です。**

<https://sdleffler.github.io/RustTypeSystemTuringComplete/>

さて、Rustの型システムでもDOOMを動かしましょう。

:::message alert
****ネタバレ：DOOMは動かせません****
:::

:::message
この記事はRustの型システムで基本的な計算と制御を行うところまでを扱います。
「型レベルスタックマシン」を実装し、コンパイル時にフィボナッチ数列を計算するところがゴールです。
:::

## Rustのコンパイラレベルのプログラミングについて

Rustでコンパイル時計算といえば `const fn` が思い浮かぶかもしれません。

```rust
const fn fibonacci(n: u64) -> u64 {
    if n <= 1 { n } else { fibonacci(n - 1) + fibonacci(n - 2) }
}

const FIB_10: u64 = fibonacci(10); // コンパイル時に計算される
```

const fnなどのコンパイル時計算は、低レベルの最適化でのわかりやすい例として、
マジックナンバー的なものがあります。暗号アルゴリズムなどではある理想的な定数をハードコードすることが往々にしてありますが、しかし意図をコードとして残しきれないのは残念です。

マジックナンバーや表などをconst fnで意味論から記述し、実行時にはオーバーヘッドがない。これはとても理想的です。

ところで、この記事ではまったく別のアプローチを取ります。
**型そのものを使って計算したいですね**。TypeScriptができるんだからやりましょう。

```rust
// こういうことをしたい
type Fib10 = /* 型レベルの計算 */;
// Fib10 = U55 がコンパイル時に検証される
```

### const fn との違い

`const fn` は「値の計算」です。型検査とは独立していて、コンパイラが定数畳み込みを行うだけです。

型レベルプログラミングは**型の世界で完結する計算**です。この違いはCurry-Howard対応に由来します。

> 型は命題であり、その型の実装（impl）は証明である。

型レベルの計算とは、型という命題を変換する推論のことです。この性質によって、「コンパイルが通る」こと自体が**計算結果の正しさの証明**になります。

:::message
**Curry-Howard対応（概略）**

| プログラミング        | 論理       |
| --------------------- | ---------- |
| 型                    | 命題       |
| 値・関数              | 証明       |
| `impl Trait for Type` | 推論規則   |
| 型レベルの計算        | 証明の変換 |

Rustのtrait解決は本質的に**論理プログラミングシステム**として動作します。traitのimplはHorn節であり、型検査はProlog的な証明探索です[^crichton]。
:::

[^crichton]: Will Crichton, "Type-level Programming in Rust" <https://willcrichton.net/notes/type-level-programming/>

## 1. implによる型レベル分岐・場合分け

型レベルプログラミングの最も基本的な道具は `impl` による**型の場合分け**です。

まず、型レベルのBooleanを定義しましょう。

```rust
// 型レベルの値: True と False は「値」ではなく「型」
pub struct True;
pub struct False;

// マーカートレイト: 「これはBoolean型である」という証明
pub trait IsBool {}

impl IsBool for True {}
impl IsBool for False {}
```

`True` と `False` は普通の構造体ですが、インスタンスを作ることはほとんどありません。これらは**型そのものが値**の役割を果たします。値の世界では `true`/`false` が存在し、型の世界では `True`/`False` が存在します。

次に、この2つの型を使って分岐を実現します。

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

type If<Cond, Then, Else> = <Cond as IfHelper<Then, Else>>::Output;
```

```rust
type R1 = If<True, u32, String>;  // = u32
type R2 = If<False, u32, String>; // = String
```

**なぜこれが「分岐」として機能するのか。** `True` と `False` は別々の型であり、Rustのトレイト解決はどのimplを使うかを**型によって一意に決定**します。`True ≠ False` が型レベルで保証されているため、分岐が決定的になります。

これはラムダ計算でのChurch Boolean (`λt. λf. t` が True、`λt. λf. f` が False) と意味論的に同型です。型システムの文脈では「型による場合分け（type case analysis）」と呼ばれます。

## 関連型で「出力」を持つ型レベル関数へ

implによる分岐を一般化しましょう。関連型（Associated Types）を使うと、**型レベルの関数**が表現できます。

```rust
pub trait Bool: Sized {
    type Not: Bool;
    type And<Rhs: Bool>: Bool;
    type Or<Rhs: Bool>: Bool;
    type Xor<Rhs: Bool>: Bool;
    type Nand<Rhs: Bool>: Bool;
}
```

`Bool` トレイトの関連型 `Not` は「Notを適用した結果の型」です。型の世界での**関数の戻り値**に相当します。

```rust
impl Bool for True {
    type Not = False;
    type And<Rhs: Bool> = Rhs;       // True AND Rhs = Rhs
    type Or<Rhs: Bool> = True;        // True OR Rhs = True
    type Xor<Rhs: Bool> = Rhs::Not;  // True XOR Rhs = NOT Rhs
    type Nand<Rhs: Bool> = Rhs::Not; // True NAND Rhs = NOT Rhs
}

impl Bool for False {
    type Not = True;
    type And<Rhs: Bool> = False;     // False AND Rhs = False
    type Or<Rhs: Bool> = Rhs;        // False OR Rhs = Rhs
    type Xor<Rhs: Bool> = Rhs;       // False XOR Rhs = Rhs
    type Nand<Rhs: Bool> = True;     // False NAND Rhs = True
}
```

型エイリアスで使いやすくすると：

```rust
type Not<T: Bool> = <T as Bool>::Not;
type And<Lhs: Bool, Rhs: Bool> = <Lhs as Bool>::And<Rhs>;
type Or<Lhs: Bool, Rhs: Bool> = <Lhs as Bool>::Or<Rhs>;
```

複雑な論理式も記述できます：

```rust
// Not(A) AND (B OR C) ? D : B
type ExampleFn<A: Bool, B: Bool, C: Bool, D> =
    If<And<Not<A>, Or<B, C>>, D, B>;

// A=False, B=True, C=False, D=u64 → And(True, True) → u64
type Result = ExampleFn<False, True, False, u64>; // = u64
```

型レベルの「半加算器」も書けます：

```rust
// 1ビット加算: (Sum, Carry)
type HalfAdder<A: Bool, B: Bool> = (Xor<A, B>, And<A, B>);
type Sum = HalfAdder<True, True>; // = (False, True) = 2
```

## 壁：ジェネリクスとの組み合わせで制約が爆発する

型エイリアスによる合成はシンプルですが、ジェネリクスと組み合わせると問題が発生します。

```rust
fn check<A: Bool, B: Bool, C: Bool>()
    -> If<And<Not<A>, Or<B, C>>, u64, i64>
{
    // コンパイラが追加で要求するもの:
    // Not<A>: Bool         ← 関数シグネチャに書いてない
    // Or<B, C>: Bool       ← 関数シグネチャに書いてない
    // And<...>: Bool       ← 関数シグネチャに書いてない
    todo!()
}
```

この関数を別の関数から呼ぶと、その関数にも同じ制約が必要になります。合成するたびに制約が積み重なっていきます。これが**制約の爆発**です。

型エイリアスは「透明」です。コンパイラは型エイリアスを展開するので、内側で使われているすべての制約が外に漏れます。

もう一つの問題は、**型エイリアスでは再帰が書けない**ことです：

```rust
// コンパイルエラー: 再帰的な型エイリアスは禁止
type Fib<N: Nat> = Add<Fib<Pred<N>>, Fib<Pred<Pred<N>>>>;
```

型エイリアス合成は**具体型だけを扱う場合**に限り有効です。ジェネリクスが入った瞬間に制約が呼び出し側全体に伝播し、「部分的にEvalを使う」という中間的な選択肢は一番損なアプローチになります。必然的に全体をEvalに統一する方向に収束します。

## 解決策：Evalパターン

これらの問題を解決するのが **Evalパターン** です。

```rust
pub trait Eval {
    type Output;
}

pub type Evaluate<T> = <T as Eval>::Output;
```

たった2行ですが、この単純な抽象が強力な性質をもたらします。

**核心的なアイデア**: 計算を「式の型」として表現し、計算の実行を `Eval` トレイトの実装として記述する。

型エイリアスが「透明」（制約が漏れる）なのに対し、Evalパターンは「不透明」です。`impl Eval for EFoo<A, B>` の中に制約を封じ込めることで、**外からは `EFoo<A, B>: Eval` の一言で済む**ようになります。

合わせて、型レベル関数を一級市民として扱うための `TyFn` トレイトを定義します：

```rust
pub trait TyFn<Arg> {
    type Output;
}
```

そして基礎的な式の型として `ELit`（値の持ち上げ）と `EApp`（関数適用）を用意します：

```rust
use std::marker::PhantomData;

// 値Tをそのまま評価すると T が返る (identity)
pub struct ELit<T>(PhantomData<T>);

impl<T> Eval for ELit<T> {
    type Output = T;
}

// 関数Efを引数Eaに適用する
pub struct EApp<Ef, Ea>(PhantomData<(Ef, Ea)>);

impl<Ef, Ea> Eval for EApp<Ef, Ea>
where
    Ea: Eval,
    Ef: TyFn<Ea::Output>,
{
    type Output = <Ef as TyFn<Ea::Output>>::Output;
}
```

`PhantomData` によってこれらの型は**ランタイムコストがゼロ**です。コンパイル時にすべて消去されます。

:::message
**理論的な背景：System Fω との対応**

Evalパターンは型理論の階層と対応しています。

| Evalの要素             | 理論的対応                             |
| ---------------------- | -------------------------------------- |
| `Eval` トレイト        | 型レベル関数の適用規則                 |
| impl内への制約封じ込め | **System Fω** における型演算子の抽象化 |
| `ELit`                 | η変換（値の持ち上げ）                  |
| `EApp`                 | β簡約（関数適用）                      |
| `TyFn<Arg>`            | 高階型（型 → 型の関数）                |

Richard EisenbergはHaskellの文脈で「型族（type families）はSystem Fωの表現力を持つ」ことを形式化しています[^eisenberg]。RustのEvalパターンはこの表現力をtraitシステムの上に構築したものと見なせます。
:::

[^eisenberg]: Richard A. Eisenberg, "Dependent Types in Haskell: Theory and Practice" (2016) <https://arxiv.org/abs/1610.07978>

## Bool演算をEvalパターンで実装する

型エイリアスで実装したBool演算をEvalパターンで書き直します。

```rust
// NOT
pub struct ENot<T>(PhantomData<T>);

impl<T> Eval for ENot<T>
where
    T: Eval,
    T::Output: Bool,
{
    type Output = <T::Output as Bool>::Not;
}

// AND
pub struct EAnd<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for EAnd<Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Lhs::Output: Bool,
    Rhs::Output: Bool,
    <Lhs::Output as Bool>::And<Rhs::Output>: Sized,
{
    type Output = <Lhs::Output as Bool>::And<Rhs::Output>;
}
```

条件分岐 `EIf` はヘルパートレイトパターンで実装します：

```rust
pub struct EIf<Cond, Then, Else>(PhantomData<(Cond, Then, Else)>);

pub trait EIfHelper<Then, Else> {
    type Output;
}

impl<Then: Eval, Else: Eval> EIfHelper<Then, Else> for True {
    type Output = Evaluate<Then>;
}

impl<Then: Eval, Else: Eval> EIfHelper<Then, Else> for False {
    type Output = Evaluate<Else>;
}

impl<Cond, Then, Else> Eval for EIf<Cond, Then, Else>
where
    Cond: Eval,
    Then: Eval,
    Else: Eval,
    Cond::Output: EIfHelper<Then, Else>,
{
    type Output = <Cond::Output as EIfHelper<Then, Else>>::Output;
}
```

使ってみましょう：

```rust
use static_assertions::assert_type_eq_all;

assert_type_eq_all!(
    Evaluate<EIf<ELit<True>, ELit<u32>, ELit<String>>>,
    u32
);

assert_type_eq_all!(
    Evaluate<EIf<ENot<ELit<True>>, ELit<u32>, ELit<String>>>,
    String
);
```

先ほど爆発した例をEvalで書き直すと：

```rust
type ExampleFn<A, B, C, D> = EIf<EAnd<ENot<A>, EOr<B, C>>, D, B>;

// 呼び出し側に要求されるのはこれだけ
fn check<A, B, C, D>() where ExampleFn<A, B, C, D>: Eval { ... }
```

制約が `ExampleFn<...>: Eval` の一言に収まっています。

## コレクション：型レベルArrayの実装

Bool演算ができたので、次はデータ構造です。

```rust
// Cons List 構造
pub trait IsList {}

pub struct Nil;
impl IsList for Nil {}

pub struct Array<Head, Tail: IsList>(PhantomData<(Head, Tail)>);
impl<Head, Tail: IsList> IsList for Array<Head, Tail> {}
```

`Array<Head, Tail>` は連結リストのcons cellです。マクロを使えば読みやすく書けます：

```rust
type MyArr = tyarray![U1, U2, U3, U4, U5];
// = Array<U1, Array<U2, Array<U3, Array<U4, Array<U5, Nil>>>>>
```

基本操作をEvalパターンで実装します：

```rust
type MyArr = tyarray![U10, U20, U30];

assert_type_eq_all!(Evaluate<ELen<MyArr>>, U3);
assert_type_eq_all!(Evaluate<EGet<MyArr, U0>>, U10);
assert_type_eq_all!(Evaluate<EGet<MyArr, U2>>, U30);

// ESetは新しい配列を返す（不変更新）
type Updated = Evaluate<ESet<MyArr, U1, U99>>;
assert_type_eq_all!(Evaluate<EGet<Updated, U1>>, U99);
```

`EGet` の実装は**再帰的なtrait bound**を使います：

```rust
trait GetHelper<Idx> {
    type Output;
}

// ベースケース: インデックスが0なら先頭を返す
impl<Head, Tail: IsList> GetHelper<U0> for Array<Head, Tail> {
    type Output = Head;
}

// 再帰ケース: インデックスをデクリメントして末尾へ
impl<Head, Tail: IsList, N: Unsigned> GetHelper<UInt<N, B1>> for Array<Head, Tail>
where
    Tail: GetHelper<N>, // ← 再帰
{
    type Output = <Tail as GetHelper<N>>::Output;
}
```

これが型レベルの再帰です。`GetHelper<N>` が `GetHelper<N-1>` を要求し、最終的にベースケース `GetHelper<U0>` に到達します。

:::message
**`typenum` について**

数値計算には `typenum` クレートの型レベル自然数を使います。Peano数を二進数で効率化した表現で、型レベルで加算・減算・比較が定義されています。

```rust
use typenum::{U0, U1, U5, U10};
type Sum = typenum::Sum<U3, U5>; // = U8 (コンパイル時)
```

:::

## 繰り返し：EWhileで型レベルのループを実現

いよいよループです。

```rust
/// EWhile<Pred, Step, State>
/// - Pred: State → Bool  (ループ継続条件)
/// - Step: State → State (ループ本体)
/// - State: 現在の状態
pub struct EWhile<Pred, Step, State>(PhantomData<(Pred, Step, State)>);
```

実装のポイントは**再帰的なtrait bound**です：

```rust
pub trait EWhileHelper<Pred, Step, State> {
    type Output;
}

// 条件がFalseならループ終了
impl<Pred, Step, State> EWhileHelper<Pred, Step, State> for False {
    type Output = State;
}

// 条件がTrueならステップを実行して再帰
impl<Pred, Step, State> EWhileHelper<Pred, Step, State> for True
where
    EApp<Step, State>: Eval,
    EWhile<Pred, Step, Evaluate<EApp<Step, State>>>: Eval, // ← 再帰！
{
    type Output =
        Evaluate<EWhile<Pred, Step, Evaluate<EApp<Step, State>>>>;
}

impl<Pred, Step, State> Eval for EWhile<Pred, Step, State>
where
    EApp<Pred, State>: Eval,
    Evaluate<EApp<Pred, State>>: EWhileHelper<Pred, Step, State>,
{
    type Output =
        <Evaluate<EApp<Pred, State>> as EWhileHelper<Pred, Step, State>>::Output;
}
```

疑似コードで表すと：

```
EWhile<Pred, Step, S> =
    if Pred(S) = True  → EWhile<Pred, Step, Step(S)>  // 再帰
    if Pred(S) = False → S                              // 終了
```

:::message
**理論的な背景：なぜこれでチューリング完全になるのか**

`EWhile` の実装は型レベルの**不動点演算子**（μ演算子）です。

```
while(pred, step, s) = μF. λs. if pred(s) then F(step(s)) else s
```

型システムの表現力の階層：

```
System T   原始再帰のみ       → 強正規化（必ず停止）
    ↓
System Fω  高階型演算子       → まだ正規化可能
    ↓
Fω + μ     不動点を追加       → チューリング完全（停止保証なし）
```

RustのtraitシステムはFωに相当し、recursive trait boundsがμ演算子を与えます。これでチューリング完全になります[^oppi]。`recursion_limit = "65536"` は理論上の無限再帰を実用上の有限に制限しているだけです。

Haskellの文脈では「型クラス + UndecidableInstances = チューリング完全」として知られており[^jones]、RustのEvalパターンはその構造的等価物です。
:::

[^oppi]: oppi.li, "Turing Complete Type Systems" <https://oppi.li/posts/turing_complete_type_systems/>
[^jones]: Mark P. Jones, "Type Classes with Functional Dependencies", ESOP 2000

## 全部合わせて：型レベルスタックマシン

Bool・Array・Whileが揃いました。これらを組み合わせると**型レベルのスタックマシン**が実装できます。

VMの状態は型として表現されます：

```rust
// VmState<Stack, Locals, Memory>
// Stack, Locals, Memory はすべて型レベルのArray
type InitState = VmState<Nil, Nil, Nil>;
```

オペコードもすべて型です：

```rust
type Push42   = OpPush<U42>;
type Add      = OpAdd;
type LetVar   = OpLet<VarA>;
type GetVar   = OpGetLocal<VarA>;
type Loop     = OpWhile<CondProgram, BodyProgram>;
```

実際にフィボナッチ数列を計算するプログラムを見てみましょう。

```rust
// Fibonacci(10) を計算するプログラム
// ローカル変数: idx=0→n, idx=1→a, idx=2→b
// while n > 0: (a, b) = (b, a+b), n -= 1

// ループ条件: n > 0
type FibCond<N> = program! {
    (push N)
    (get_local U0)  // n をスタックに積む
    (lt)            // N < n ? → True/False
};

// ループ本体
type FibBody = program! {
    (get_local U1)  // a
    (get_local U2)  // b
    (add)           // a + b
    (get_local U2)  // b
    (set_local U1)  // a = b
    (set_local U2)  // b = a + b
    (get_local U0)  // n
    (push U1)
    (sub)
    (set_local U0)  // n -= 1
};

type FibProgram10 = program! {
    (push U10) (let)  // n = 10
    (push U0)  (let)  // a = 0
    (push U1)  (let)  // b = 1
    (while FibCond<U10> FibBody)
    (get_local U2)    // return b
};

// コンパイラが型を解決する時点で Fib(10) = 55 が計算される
assert_type_eq_all!(
    Evaluate<ETop<Stack<Evaluate<ERunVm<FibProgram10, InitVmState>>>>>,
    U55
);
```

コンパイルが通ること自体が「Fibonacci(10) = 55」の証明になっています。

## まとめ：何を実現したか

この記事で積み上げてきたもの：

| 機能           | 型レベルでの表現            | 理論的対応             |
| -------------- | --------------------------- | ---------------------- |
| 条件分岐       | distinct impl dispatch      | 型による場合分け       |
| 型レベル関数   | `trait Bool { type Not; }`  | 関連型 = 関数の出力    |
| 制約の封じ込め | Evalパターン                | System Fω の型演算子   |
| 再帰           | recursive trait bounds      | μ演算子（不動点）      |
| ループ         | `EWhile<Pred, Step, State>` | μ + 条件分岐           |
| コレクション   | `Array<Head, Tail>`         | 帰納的データ構造       |
| スタックマシン | `ERunVm<Program, State>`    | 上記すべての組み合わせ |

計算の十分条件となる3要素：

1. **型による場合分け（条件分岐）** — distinct implのdispatch
2. **recursive trait bounds（再帰）** — μ演算子として機能
3. **関連型（状態）** — 型引数による状態の伝播

この3つが揃ったとき、Rustの型システムはチューリング完全な計算機として機能します。

DOOMは動かせませんでしたが、コンパイル時にFibonacci(10) = 55 を証明できました。

次回は... 本当にDOOMを目指してみましょうか。

---

*この記事で実装したtypeludeはGitHubで公開しています。*
