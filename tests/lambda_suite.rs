#![recursion_limit = "512"]

use std::marker::PhantomData;

use static_assertions::assert_type_eq_all;
use typelude::{
    eval::Evaluate,
    lambda::{
        Apply,
        church::{
            LAdd, LExp, LFalse, LFst, LIf, LMul, LPair, LPred, LPureAdd, LSnd, LSub, LSucc,
            LSuccGen, LTrue, LZero,
        },
        fix::LFix,
        list::{LCons, LFoldr, LHeadOr, LIsEmpty, LNil, LTailOr},
        ski::{I, K, S},
    },
};

type App<F, A> = <F as Apply<A>>::Output;

// =========================================================================
// Helper Types
// =========================================================================

type One = LSucc<LZero>;
type Two = LSucc<One>;
type Three = LSucc<Two>;
type Four = LSucc<Three>;
type Five = LSucc<Four>;
type Six = LSucc<Five>;
type Eight = LSucc<LSucc<Six>>;

// Helper to normalize Church numerals (Function form) to Structural form (Succ<...>)
// N -> N SuccGen Zero
type Normalize<N> = <<N as Apply<LSuccGen>>::Output as Apply<LZero>>::Output;

// =========================================================================
// 1. Basic Combinators (SKI)
// =========================================================================

#[test]
fn test_ski_logic() {
    // I x = x
    type ResI = <I as Apply<LZero>>::Output;
    assert_type_eq_all!(ResI, LZero);

    // K x y = x
    type ResK = <<K as Apply<LZero>>::Output as Apply<One>>::Output;
    assert_type_eq_all!(ResK, LZero);

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
    type RawRes = Evaluate<App<App<LAdd, Two>, Three>>;
    type Res = Normalize<RawRes>;
    assert_type_eq_all!(Res, Five);
}

#[test]
fn test_church_arithmetic_mul() {
    // 2 * 3 = 6
    // 2 * 3 = 6
    type RawRes = Evaluate<App<App<LMul, Two>, Three>>;
    type Res = Normalize<RawRes>;
    assert_type_eq_all!(Res, Six);

    // 3 * 2 = 6
    type RawRes2 = Evaluate<App<App<LMul, Three>, Two>>;
    type Res2 = Normalize<RawRes2>;
    assert_type_eq_all!(Res2, Six);

    // 0 * 5 = 0
    type RawRes3 = Evaluate<App<App<LMul, LZero>, Five>>;
    type Res3 = Normalize<RawRes3>;
    assert_type_eq_all!(Res3, LZero);
}

#[test]
fn test_church_arithmetic_exp() {
    // 2 ^ 3 = 8
    // Exp returns a function, we must normalize it to verify structure
    type RawRes = Evaluate<LExp<Two, Three>>;
    type Res = Normalize<RawRes>;
    assert_type_eq_all!(Res, Eight);

    // 3 ^ 2 = 9
    type Nine = LSucc<Eight>;
    type RawRes2 = Evaluate<LExp<Three, Two>>;
    type Res2 = Normalize<RawRes2>;
    assert_type_eq_all!(Res2, Nine);

    // x ^ 0 = 1
    type RawRes3 = Evaluate<LExp<Five, LZero>>;
    type Res3 = Normalize<RawRes3>;
    assert_type_eq_all!(Res3, One);
}

#[test]
fn test_church_arithmetic_pred() {
    // Pred 1 = 0
    type Res1 = Evaluate<LPred<One>>;
    assert_type_eq_all!(Res1, LZero);

    // Pred 3 = 2
    type Res2 = Evaluate<LPred<Three>>;
    assert_type_eq_all!(Res2, Two);

    // Pred 0 = 0 (Saturation)
    type Res3 = Evaluate<LPred<LZero>>;
    assert_type_eq_all!(Res3, LZero);
}

#[test]
fn test_church_arithmetic_sub() {
    // 3 - 2 = 1
    type Res1 = Evaluate<LSub<Three, Two>>;
    assert_type_eq_all!(Res1, One);

    // 2 - 3 = 0 (Saturation)
    type Res2 = Evaluate<LSub<Two, Three>>;
    assert_type_eq_all!(Res2, LZero);

    // 5 - 0 = 5
    type Res3 = Evaluate<LSub<Five, LZero>>;
    assert_type_eq_all!(Res3, Five);
}

// =========================================================================
// 3. Church Booleans & Pairs
// =========================================================================

#[test]
fn test_church_pairs() {
    type P = LPair<One, Two>;
    type F = Evaluate<LFst<P>>;
    type S = Evaluate<LSnd<P>>;

    assert_type_eq_all!(F, One);
    assert_type_eq_all!(S, Two);
}

#[test]
fn test_church_bools() {
    // If True One Two -> One
    type Res1 = Evaluate<LIf<LTrue, One, Two>>;
    assert_type_eq_all!(Res1, One);

    // If False One Two -> Two
    type Res2 = Evaluate<LIf<LFalse, One, Two>>;
    assert_type_eq_all!(Res2, Two);
}

// =========================================================================
// 4. List Operations
// =========================================================================

#[test]
fn test_list_accessors() {
    struct DefaultVal;
    type L = LCons<One, LCons<Two, LNil>>;

    // Head
    type H = Evaluate<LHeadOr<L, DefaultVal>>;
    assert_type_eq_all!(H, One);

    // Tail
    type T = Evaluate<LTailOr<L, DefaultVal>>;
    // T should be Cons<Two, Nil>
    // Let's check Head of T
    type HT = Evaluate<LHeadOr<T, DefaultVal>>;
    assert_type_eq_all!(HT, Two);

    // IsEmpty
    assert_type_eq_all!(Evaluate<LIsEmpty<L>>, LFalse);
    assert_type_eq_all!(Evaluate<LIsEmpty<LNil>>, LTrue);

    // Empty Access
    type EmptyH = Evaluate<LHeadOr<LNil, DefaultVal>>;
    assert_type_eq_all!(EmptyH, DefaultVal);
}

#[test]
fn test_list_fold_sum() {
    // Sum [1, 2, 3] = 6
    type L = LCons<One, LCons<Two, LCons<Three, LNil>>>;

    // OpSum: \x acc. PureAdd<x, acc>
    struct OpSum;
    impl<X> Apply<X> for OpSum {
        type Output = OpSum1<X>;
    }
    struct OpSum1<X>(PhantomData<X>);

    impl<X, Acc> Apply<Acc> for OpSum1<X>
    where
        X: Apply<LSuccGen>,
        <X as Apply<LSuccGen>>::Output: Apply<Acc>,
    {
        // Return PureAdd directly, so ECall gets the result value, not the Add expression.
        type Output = LPureAdd<X, Acc>;
    }

    type RawRes = Evaluate<LFoldr<OpSum, LZero, L>>;
    type Res = Normalize<RawRes>;
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

    impl<R> Apply<R> for Unroll {
        type Output = Unroll1<R>;
    }

    // Base case: RecFunc Z -> Done
    impl<R> Apply<Z> for Unroll1<R> {
        type Output = Done;
    }

    // Recursive case: RecFunc (S P) -> RecFunc P
    impl<R, P> Apply<S<P>> for Unroll1<R>
    where
        R: Apply<P>,
    {
        type Output = <R as Apply<P>>::Output;
    }

    type RecFunc = LFix<Unroll>;

    assert_type_eq_all!(<RecFunc as Apply<Z>>::Output, Done);
    assert_type_eq_all!(<RecFunc as Apply<S<Z>>>::Output, Done);
    assert_type_eq_all!(<RecFunc as Apply<S<S<Z>>>>::Output, Done);
}
