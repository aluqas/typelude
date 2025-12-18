#![recursion_limit = "512"]

use typelude::eval::{Evaluate, Eval};
use typelude::lambda::Apply;
use typelude::lambda::church::{
    Zero, Succ, Add, Mul, Exp, Sub, Pred, True, False,
    PureAdd, If, Pair, Fst, Snd, SuccGen
};
use typelude::lambda::ski::{S, K, I};
use typelude::lambda::list::{Nil, Cons, Foldr, HeadOr, TailOr, IsEmpty};
use typelude::lambda::fix::Fix;
use static_assertions::assert_type_eq_all;
use std::marker::PhantomData;

// =========================================================================
// Helper Types
// =========================================================================

type One = Succ<Zero>;
type Two = Succ<One>;
type Three = Succ<Two>;
type Four = Succ<Three>;
type Five = Succ<Four>;
type Six = Succ<Five>;
type Eight = Succ<Succ<Six>>;

// Helper to normalize Church numerals (Function form) to Structural form (Succ<...>)
// N -> N SuccGen Zero
type Normalize<N> = <<N as Apply<SuccGen>>::Output as Apply<Zero>>::Output;

// =========================================================================
// 1. Basic Combinators (SKI)
// =========================================================================

#[test]
fn test_ski_logic() {
    // I x = x
    type ResI = <I as Apply<Zero>>::Output;
    assert_type_eq_all!(ResI, Zero);

    // K x y = x
    type ResK = <<K as Apply<Zero>>::Output as Apply<One>>::Output;
    assert_type_eq_all!(ResK, Zero);

    // S K K x = I x = x
    type SKK = <<S as Apply<K>>::Output as Apply<K>>::Output;
    type ResSKK = <SKK as Apply<Five>>::Output;
    assert_type_eq_all!(ResSKK, Five);
}

// =========================================================================
// 2. Church Arithmetic
// =========================================================================

#[test]
fn test_church_arithmetic_add() {
    // 2 + 3 = 5
    type Res = Evaluate<Add<Two, Three>>;
    assert_type_eq_all!(Res, Five);
}

#[test]
fn test_church_arithmetic_mul() {
    // 2 * 3 = 6
    type Res = Evaluate<Mul<Two, Three>>;
    assert_type_eq_all!(Res, Six);

    // 3 * 2 = 6
    type Res2 = Evaluate<Mul<Three, Two>>;
    assert_type_eq_all!(Res2, Six);

    // 0 * 5 = 0
    type Res3 = Evaluate<Mul<Zero, Five>>;
    assert_type_eq_all!(Res3, Zero);
}

#[test]
fn test_church_arithmetic_exp() {
    // 2 ^ 3 = 8
    // Exp returns a function, we must normalize it to verify structure
    type RawRes = Evaluate<Exp<Two, Three>>;
    type Res = Normalize<RawRes>;
    assert_type_eq_all!(Res, Eight);

    // 3 ^ 2 = 9
    type Nine = Succ<Eight>;
    type RawRes2 = Evaluate<Exp<Three, Two>>;
    type Res2 = Normalize<RawRes2>;
    assert_type_eq_all!(Res2, Nine);

    // x ^ 0 = 1
    type RawRes3 = Evaluate<Exp<Five, Zero>>;
    type Res3 = Normalize<RawRes3>;
    assert_type_eq_all!(Res3, One);
}

#[test]
fn test_church_arithmetic_pred() {
    // Pred 1 = 0
    type Res1 = Evaluate<Pred<One>>;
    assert_type_eq_all!(Res1, Zero);

    // Pred 3 = 2
    type Res2 = Evaluate<Pred<Three>>;
    assert_type_eq_all!(Res2, Two);

    // Pred 0 = 0 (Saturation)
    type Res3 = Evaluate<Pred<Zero>>;
    assert_type_eq_all!(Res3, Zero);
}

#[test]
fn test_church_arithmetic_sub() {
    // 3 - 2 = 1
    type Res1 = Evaluate<Sub<Three, Two>>;
    assert_type_eq_all!(Res1, One);

    // 2 - 3 = 0 (Saturation)
    type Res2 = Evaluate<Sub<Two, Three>>;
    assert_type_eq_all!(Res2, Zero);

    // 5 - 0 = 5
    type Res3 = Evaluate<Sub<Five, Zero>>;
    assert_type_eq_all!(Res3, Five);
}

// =========================================================================
// 3. Church Booleans & Pairs
// =========================================================================

#[test]
fn test_church_pairs() {
    type P = Pair<One, Two>;
    type F = Evaluate<Fst<P>>;
    type S = Evaluate<Snd<P>>;

    assert_type_eq_all!(F, One);
    assert_type_eq_all!(S, Two);
}

#[test]
fn test_church_bools() {
    // If True One Two -> One
    type Res1 = Evaluate<If<True, One, Two>>;
    assert_type_eq_all!(Res1, One);

    // If False One Two -> Two
    type Res2 = Evaluate<If<False, One, Two>>;
    assert_type_eq_all!(Res2, Two);
}

// =========================================================================
// 4. List Operations
// =========================================================================

#[test]
fn test_list_accessors() {
    struct DefaultVal;
    type L = Cons<One, Cons<Two, Nil>>;

    // Head
    type H = Evaluate<HeadOr<L, DefaultVal>>;
    assert_type_eq_all!(H, One);

    // Tail
    type T = Evaluate<TailOr<L, DefaultVal>>;
    // T should be Cons<Two, Nil>
    // Let's check Head of T
    type HT = Evaluate<HeadOr<T, DefaultVal>>;
    assert_type_eq_all!(HT, Two);

    // IsEmpty
    assert_type_eq_all!(Evaluate<IsEmpty<L>>, False);
    assert_type_eq_all!(Evaluate<IsEmpty<Nil>>, True);

    // Empty Access
    type EmptyH = Evaluate<HeadOr<Nil, DefaultVal>>;
    assert_type_eq_all!(EmptyH, DefaultVal);
}

#[test]
fn test_list_fold_sum() {
    // Sum [1, 2, 3] = 6
    type L = Cons<One, Cons<Two, Cons<Three, Nil>>>;

    // OpSum: \x acc. PureAdd<x, acc>
    struct OpSum;
    impl<X> Apply<X> for OpSum { type Output = OpSum1<X>; }
    struct OpSum1<X>(PhantomData<X>);

    impl<X, Acc> Apply<Acc> for OpSum1<X>
    where X: Apply<SuccGen>,
          <X as Apply<SuccGen>>::Output: Apply<Acc>
    {
        // Return PureAdd directly, so ECall gets the result value, not the Add expression.
        type Output = PureAdd<X, Acc>;
    }

    type Res = Evaluate<Foldr<OpSum, Zero, L>>;
    assert_type_eq_all!(Res, Six);
}

// =========================================================================
// 5. Fixed-Point Combinator
// =========================================================================

#[test]
fn test_fix_simple_recursion() {
    // Test a simple recursive countdown using structural types (Z, S<N>) to avoid
    // complexity of Church Arithmetic bounds in this specific test.

    struct Z;
    struct S<N>(PhantomData<N>);

    // F(0) = Done
    // F(S n) = F(n)

    struct Unroll;
    struct Unroll1<R>(PhantomData<R>);
    struct Done;

    impl<R> Apply<R> for Unroll { type Output = Unroll1<R>; }

    // Base case: RecFunc Z -> Done
    impl<R> Apply<Z> for Unroll1<R> { type Output = Done; }

    // Recursive case: RecFunc (S P) -> RecFunc P
    impl<R, P> Apply<S<P>> for Unroll1<R>
    where R: Apply<P>
    {
        type Output = <R as Apply<P>>::Output;
    }

    type RecFunc = Fix<Unroll>;

    assert_type_eq_all!(<RecFunc as Apply<Z>>::Output, Done);
    assert_type_eq_all!(<RecFunc as Apply<S<Z>>>::Output, Done);
    assert_type_eq_all!(<RecFunc as Apply<S<S<Z>>>>::Output, Done);
}
