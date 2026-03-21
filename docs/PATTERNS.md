# Design Patterns

typeludeにおける型レベルプログラミングのデザインパターン集。

## 1. Evalパターン

### 問題

型エイリアスによる合成はジェネリクスと組み合わせると制約が呼び出し側に漏れる（制約の爆発）。
また型エイリアスでは再帰型が書けない。

```rust
// NG: 型エイリアス合成の限界
type Foo<A: Bool, B: Bool> = And<Not<A>, Or<A, B>>;
// Foo<A, B> を使う側に Not<A>: Bool, Or<A,B>: Bool... が全部必要になる

// NG: 再帰型エイリアスは禁止
type Fib<N> = Add<Fib<Pred<N>>, Fib<Pred<Pred<N>>>>; // コンパイルエラー
```

### 解決

計算を「式の型」として表現し、評価を `Eval` トレイトの実装として記述する。

```rust
pub trait Eval { type Output; }
pub type Evaluate<T> = <T as Eval>::Output;

pub struct EFoo<A, B>(PhantomData<(A, B)>);

impl<A, B> Eval for EFoo<A, B>
where
    A: Eval,
    B: Eval,
    // 制約はここに封じ込める
    Evaluate<A>: Bool,
    Evaluate<B>: Bool,
{
    type Output = <Evaluate<A> as Bool>::And<Evaluate<B>>;
}

// 呼び出し側は EFoo<A, B>: Eval だけ
fn bar<A, B>() where EFoo<A, B>: Eval { ... }
```

### いつ型エイリアスを使ってよいか

具体型だけを扱う場合（ジェネリクスなし）は型エイリアスで十分。

```rust
// OK: 入力が具体型
type MyBool = And<Not<True>, Or<False, True>>;

// NG: ジェネリクスが入ったらEvalに切り替える
type Foo<A: Bool> = Not<A>; // Aがジェネリクスなのでいずれ爆発する
```

---

## 2. Helperトレイトパターン

### 問題

`True`/`False` のような型でdispatchする際、直接 `Eval` の実装を書こうとすると
overlapping implicationsの問題が起きる。

```rust
// NG: overlapping impl になりうる
impl<Cond: Eval, Then, Else> Eval for EIf<Cond, Then, Else>
where Evaluate<Cond> = True { ... }   // これはRustでは書けない
```

### 解決

中間のHelperトレイトを挟み、型によるdispatchをそこで行う。

```rust
// Helperトレイトで True/False のdispatchを担当
pub trait EIfHelper<Then, Else> {
    type Output;
}

impl<Then: Eval, Else: Eval> EIfHelper<Then, Else> for True {
    type Output = Evaluate<Then>;
}

impl<Then: Eval, Else: Eval> EIfHelper<Then, Else> for False {
    type Output = Evaluate<Else>;
}

// EIf の Eval 実装はHelperに委譲するだけ
impl<Cond, Then, Else> Eval for EIf<Cond, Then, Else>
where
    Cond: Eval,
    Then: Eval,
    Else: Eval,
    Evaluate<Cond>: EIfHelper<Then, Else>,
{
    type Output = <Evaluate<Cond> as EIfHelper<Then, Else>>::Output;
}
```

### 命名規則

Helperトレイトは `XxxHelper` という名前で、対応するE型と同じファイルに置く。
マクロ `helper_if!`, `helper_list!`, `helper_bit!` で定型的なHelperを自動生成できる。

---

## 3. 再帰的trait boundパターン

### 問題

型レベルの再帰計算（リストの走査、ループ）を表現したい。

### 解決

`impl` のwhere句に自身を含む型を要求することで再帰を表現する。

```rust
// EGet の再帰実装
trait GetHelper<Idx> { type Output; }

// ベースケース
impl<Head, Tail: IsList> GetHelper<U0> for Array<Head, Tail> {
    type Output = Head;
}

// 再帰ケース: GetHelper<N> が GetHelper<N-1> を要求する
impl<Head, Tail: IsList, N> GetHelper<UInt<N, B1>> for Array<Head, Tail>
where
    Tail: GetHelper<N>, // ← これが型レベルの再帰
{
    type Output = <Tail as GetHelper<N>>::Output;
}
```

EWhileも同様の構造：

```rust
impl<Pred, Step, State> EWhileHelper<Pred, Step, State> for True
where
    EApp<Step, State>: Eval,
    EWhile<Pred, Step, Evaluate<EApp<Step, State>>>: Eval, // ← 再帰
{
    type Output = Evaluate<EWhile<Pred, Step, Evaluate<EApp<Step, State>>>>;
}
```

### 注意

再帰の深さは `recursion_limit` に制限される。
深い再帰が必要な場合は `#![recursion_limit = "65536"]` を設定する。

---

## 4. PhantomDataによるゼロコスト型パラメータ

### 問題

型パラメータを持つが、ランタイムにはデータを持たない式の型を定義したい。

### 解決

`PhantomData` で型情報を保持する。コンパイル時にすべて消去される。

```rust
pub struct EIf<Cond, Then, Else>(PhantomData<(Cond, Then, Else)>);
pub struct EApp<Ef, Ea>(PhantomData<(Ef, Ea)>);
pub struct Array<Head, Tail: IsList>(PhantomData<(Head, Tail)>);
```

---

## 5. マクロによるボイラープレート削減

算術・論理演算のような定型的なE型の定義には専用マクロを使う。

```rust
// NG: 毎回手書きするのは冗長
pub struct EAdd<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);
impl<Lhs, Rhs> Eval for EAdd<Lhs, Rhs> where ... { ... }
impl<Lhs, Rhs> TyFn<(Lhs, Rhs)> for EAdd<Lhs, Rhs> where ... { ... }

// OK: マクロで生成
define_arith_op!(Add, TAdd, "Type-level addition");
// → EAdd が生成される

define_logic_op!(And, Bool, And, "Type-level AND");
// → EAnd が生成される
```

`ty_fn!` マクロは `TyFn` + `Eval` の実装を同時に生成する：

```rust
ty_fn! {
    pub struct EMyOp<A, B>
    where A, B, ~A: MyTrait<~B>
    {
        type Output = <~A as MyTrait<~B>>::Output;
    }
}
```

`~A` は「Aを評価した型」（`Evaluate<A>`）を意味するDSL記法。
