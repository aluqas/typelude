//! Church Numerals and Arithmetic
//!
//! Church encoding of natural numbers and arithmetic operations:
//! - `Zero`: λf x. x
//! - `Succ<N>`: λn f x. f (n f x)
//!
//! # Evaluation Strategy
//!
//! This module uses **direct `Eval` implementation** for `LApp` nodes,
//! avoiding the `Lambda` blanket impl which causes eager trait resolution.
//! Value types implement both `Lambda` and `Eval`. Application nodes
//! implement only `Eval` with explicit where-clauses.

use std::marker::PhantomData;

use crate::{
    eval::{Eval, Evaluate},
    lambda::{
        LApp,
        traits::{LNat, LTerm},
    },
};

// =========================================================================
// Church Numerals (Value Types)
// =========================================================================

/// Zero: λf x. x
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LZero;
impl LTerm for LZero {}
impl LNat for LZero {}

impl Eval for LZero {
    type Output = LZero;
}

/// Succ: λn f x. f (n f x)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LSucc<N>(PhantomData<N>);
impl<N> LTerm for LSucc<N> {}
impl<N> LNat for LSucc<N> {}

impl<N> Eval for LSucc<N> {
    type Output = LSucc<N>;
}

// --- Partial Application States (Value Types) ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LZero1<F>(PhantomData<F>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LSucc1<N, F>(PhantomData<(N, F)>);

impl<F> Eval for LZero1<F> {
    type Output = LZero1<F>;
}
impl<N, F> Eval for LSucc1<N, F> {
    type Output = LSucc1<N, F>;
}

// =========================================================================
// Eval implementations for LApp (Direct, no Lambda blanket)
// =========================================================================

// --- Zero ---
// Zero F -> Zero1<F>
impl<F> Eval for LApp<LZero, F> {
    type Output = LZero1<F>;
}

// Zero1<F> X -> X
impl<F, X> Eval for LApp<LZero1<F>, X>
where
    X: Eval,
{
    type Output = Evaluate<X>;
}

// --- Succ ---
// Succ<N> F -> Succ1<N, F>
impl<N, F> Eval for LApp<LSucc<N>, F> {
    type Output = LSucc1<N, F>;
}

// Succ1<N, F> X -> F (N F X)
impl<N, F, X> Eval for LApp<LSucc1<N, F>, X>
where
    // (N F)
    LApp<N, F>: Eval,
    // ((N F) X)
    LApp<Evaluate<LApp<N, F>>, X>: Eval,
    // (F ((N F) X))
    LApp<F, Evaluate<LApp<Evaluate<LApp<N, F>>, X>>>: Eval,
{
    type Output = Evaluate<LApp<F, Evaluate<LApp<Evaluate<LApp<N, F>>, X>>>>;
}

/// SuccGen: Generates Succ<N> from N
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LSuccGen;

impl Eval for LSuccGen {
    type Output = LSuccGen;
}

// SuccGen N -> Succ<N>
impl<N> Eval for LApp<LSuccGen, N> {
    type Output = LSucc<N>;
}

// =========================================================================
// Arithmetic Operations
// =========================================================================

// --- Add: λm n f x. m f (n f x) ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LAdd;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LAdd1<M>(PhantomData<M>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LAdd2<M, N>(PhantomData<(M, N)>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LAdd3<M, N, F>(PhantomData<(M, N, F)>);

impl Eval for LAdd {
    type Output = LAdd;
}
impl<M> Eval for LAdd1<M> {
    type Output = LAdd1<M>;
}
impl<M, N> Eval for LAdd2<M, N> {
    type Output = LAdd2<M, N>;
}
impl<M, N, F> Eval for LAdd3<M, N, F> {
    type Output = LAdd3<M, N, F>;
}

// Add M -> Add1<M>
impl<M> Eval for LApp<LAdd, M> {
    type Output = LAdd1<M>;
}

// Add1<M> N -> Add2<M, N>
impl<M, N> Eval for LApp<LAdd1<M>, N> {
    type Output = LAdd2<M, N>;
}

// Add2<M, N> F -> Add3<M, N, F>
impl<M, N, F> Eval for LApp<LAdd2<M, N>, F> {
    type Output = LAdd3<M, N, F>;
}

// Add3<M, N, F> X -> M F (N F X)
impl<M, N, F, X> Eval for LApp<LAdd3<M, N, F>, X>
where
    // (N F)
    LApp<N, F>: Eval,
    // ((N F) X)
    LApp<Evaluate<LApp<N, F>>, X>: Eval,
    // (M F)
    LApp<M, F>: Eval,
    // ((M F) ((N F) X))
    LApp<Evaluate<LApp<M, F>>, Evaluate<LApp<Evaluate<LApp<N, F>>, X>>>: Eval,
{
    type Output = Evaluate<LApp<Evaluate<LApp<M, F>>, Evaluate<LApp<Evaluate<LApp<N, F>>, X>>>>;
}

// --- Mul: λm n f x. m (n f) x ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LMul;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LMul1<M>(PhantomData<M>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LMul2<M, N>(PhantomData<(M, N)>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LMul3<M, N, F>(PhantomData<(M, N, F)>);

impl Eval for LMul {
    type Output = LMul;
}
impl<M> Eval for LMul1<M> {
    type Output = LMul1<M>;
}
impl<M, N> Eval for LMul2<M, N> {
    type Output = LMul2<M, N>;
}
impl<M, N, F> Eval for LMul3<M, N, F> {
    type Output = LMul3<M, N, F>;
}

// Mul M -> Mul1<M>
impl<M> Eval for LApp<LMul, M> {
    type Output = LMul1<M>;
}

// Mul1<M> N -> Mul2<M, N>
impl<M, N> Eval for LApp<LMul1<M>, N> {
    type Output = LMul2<M, N>;
}

// Mul2<M, N> F -> Mul3<M, N, F>
impl<M, N, F> Eval for LApp<LMul2<M, N>, F> {
    type Output = LMul3<M, N, F>;
}

// Mul3<M, N, F> X -> M (N F) X
impl<M, N, F, X> Eval for LApp<LMul3<M, N, F>, X>
where
    // (N F)
    LApp<N, F>: Eval,
    // (M (N F))
    LApp<M, Evaluate<LApp<N, F>>>: Eval,
    // ((M (N F)) X)
    LApp<Evaluate<LApp<M, Evaluate<LApp<N, F>>>>, X>: Eval,
{
    type Output = Evaluate<LApp<Evaluate<LApp<M, Evaluate<LApp<N, F>>>>, X>>;
}

// --- Exp: λm n. n m ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LExp;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LExp1<M>(PhantomData<M>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LExp2<M, N>(PhantomData<(M, N)>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LExp3<M, N, F>(PhantomData<(M, N, F)>);

impl Eval for LExp {
    type Output = LExp;
}
impl<M> Eval for LExp1<M> {
    type Output = LExp1<M>;
}
impl<M, N> Eval for LExp2<M, N> {
    type Output = LExp2<M, N>;
}
impl<M, N, F> Eval for LExp3<M, N, F> {
    type Output = LExp3<M, N, F>;
}

// Exp M -> Exp1<M>
impl<M> Eval for LApp<LExp, M> {
    type Output = LExp1<M>;
}

// Exp1<M> N -> Exp2<M, N>
impl<M, N> Eval for LApp<LExp1<M>, N> {
    type Output = LExp2<M, N>;
}

// Exp2<M, N> F -> Exp3<M, N, F>
impl<M, N, F> Eval for LApp<LExp2<M, N>, F> {
    type Output = LExp3<M, N, F>;
}

// Exp3<M, N, F> X -> ((N M) F) X
impl<M, N, F, X> Eval for LApp<LExp3<M, N, F>, X>
where
    // (N M)
    LApp<N, M>: Eval,
    // ((N M) F)
    LApp<Evaluate<LApp<N, M>>, F>: Eval,
    // (((N M) F) X)
    LApp<Evaluate<LApp<Evaluate<LApp<N, M>>, F>>, X>: Eval,
{
    type Output = Evaluate<LApp<Evaluate<LApp<Evaluate<LApp<N, M>>, F>>, X>>;
}

// =========================================================================
// Predecessor and Subtraction
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LPred;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LPredStep;

impl Eval for LPred {
    type Output = LPred;
}
impl Eval for LPredStep {
    type Output = LPredStep;
}

// Pred N -> Fst (N PredStep (Pair Zero Zero))
impl<N> Eval for LApp<LPred, N>
where
    // (N PredStep)
    LApp<N, LPredStep>: Eval,
    // ((N PredStep) (Pair Zero Zero))
    LApp<Evaluate<LApp<N, LPredStep>>, super::pair::LPair2<LZero, LZero>>: Eval,
    // (Fst ...)
    LApp<
        super::pair::LFst,
        Evaluate<LApp<Evaluate<LApp<N, LPredStep>>, super::pair::LPair2<LZero, LZero>>>,
    >: Eval,
{
    type Output = Evaluate<
        LApp<
            super::pair::LFst,
            Evaluate<LApp<Evaluate<LApp<N, LPredStep>>, super::pair::LPair2<LZero, LZero>>>,
        >,
    >;
}

// PredStep P -> Pair (Snd P) (Succ (Snd P))
impl<P> Eval for LApp<LPredStep, P>
where
    // (Snd P)
    LApp<super::pair::LSnd, P>: Eval,
    // (SuccGen (Snd P))
    LApp<LSuccGen, Evaluate<LApp<super::pair::LSnd, P>>>: Eval,
{
    type Output = super::pair::LPair2<
        Evaluate<LApp<super::pair::LSnd, P>>,
        Evaluate<LApp<LSuccGen, Evaluate<LApp<super::pair::LSnd, P>>>>,
    >;
}

// --- Sub: λm n. n pred m ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LSub;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LSub1<M>(PhantomData<M>);

impl Eval for LSub {
    type Output = LSub;
}
impl<M> Eval for LSub1<M> {
    type Output = LSub1<M>;
}

// Sub M -> Sub1<M>
impl<M> Eval for LApp<LSub, M> {
    type Output = LSub1<M>;
}

// Sub1<M> N -> (N Pred) M
impl<M, N> Eval for LApp<LSub1<M>, N>
where
    // (N Pred)
    LApp<N, LPred>: Eval,
    // ((N Pred) M)
    LApp<Evaluate<LApp<N, LPred>>, M>: Eval,
{
    type Output = Evaluate<LApp<Evaluate<LApp<N, LPred>>, M>>;
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;

    // Helper alias
    type App<F, A> = Evaluate<LApp<F, A>>;

    // Numbers
    type One = LSucc<LZero>;
    type Two = LSucc<One>;
    type Three = LSucc<Two>;

    #[test]
    fn test_church_numerals_basic() {
        #[derive(Clone)]
        struct F;
        impl Eval for F {
            type Output = F;
        }

        struct F1<T>(std::marker::PhantomData<T>);
        impl<T> Eval for F1<T> {
            type Output = F1<T>;
        }

        #[derive(Clone)]
        struct X;
        impl Eval for X {
            type Output = X;
        }

        // Define F X -> F1<Eval(X)>
        impl<X: Eval> Eval for LApp<F, X> {
            type Output = F1<Evaluate<X>>;
        }
        impl<X, Y: Eval> Eval for LApp<F1<X>, Y> {
            type Output = F1<Evaluate<Y>>;
        }

        // Zero F X -> X
        type ResZero = App<App<LZero, F>, X>;
        assert_type_eq_all!(ResZero, X);

        // One F X -> F X -> F1<X>
        type ResOne = App<App<One, F>, X>;
        assert_type_eq_all!(ResOne, F1<X>);

        // SuccGen
        type TwoComputed = App<LSuccGen, One>;
        assert_type_eq_all!(TwoComputed, Two);
    }

    #[test]
    fn test_church_arithmetic() {
        #[derive(Clone)]
        struct F;
        impl Eval for F {
            type Output = F;
        }

        struct F1<T>(std::marker::PhantomData<T>);
        impl<T> Eval for F1<T> {
            type Output = F1<T>;
        }

        #[derive(Clone)]
        struct X;
        impl Eval for X {
            type Output = X;
        }

        impl<X: Eval> Eval for LApp<F, X> {
            type Output = F1<Evaluate<X>>;
        }
        impl<X, Y: Eval> Eval for LApp<F1<X>, Y> {
            type Output = F1<Evaluate<Y>>;
        }

        // Add
        // 1 + 2 = 3
        type Sum = App<App<LAdd, One>, Two>;
        // Verify Sum F X == Three F X
        type SumRes = App<App<Sum, F>, X>;
        type ThreeRes = App<App<Three, F>, X>;
        assert_type_eq_all!(SumRes, ThreeRes);

        // Mul
        // 1 * 2 = 2
        type Prod = App<App<LMul, One>, Two>;
        type ProdRes = App<App<Prod, F>, X>;
        type TwoRes = App<App<Two, F>, X>;
        assert_type_eq_all!(ProdRes, TwoRes);

        // Exp
        // 2 ^ 1 = 2
        type Pow1 = App<App<LExp, Two>, One>;
        type Pow1Res = App<App<Pow1, F>, X>;
        assert_type_eq_all!(Pow1Res, TwoRes);
    }

    #[test]
    fn test_church_pred_sub() {
        #[derive(Clone)]
        struct F;
        impl Eval for F {
            type Output = F;
        }

        struct F1<T>(std::marker::PhantomData<T>);
        impl<T> Eval for F1<T> {
            type Output = F1<T>;
        }

        #[derive(Clone)]
        struct X;
        impl Eval for X {
            type Output = X;
        }

        impl<X: Eval> Eval for LApp<F, X> {
            type Output = F1<Evaluate<X>>;
        }
        impl<X, Y: Eval> Eval for LApp<F1<X>, Y> {
            type Output = F1<Evaluate<Y>>;
        }

        // Pred 1 = 0
        type Pred1 = App<LPred, One>;
        type Pred1Res = App<App<Pred1, F>, X>;
        type ZeroRes = App<App<LZero, F>, X>;
        assert_type_eq_all!(Pred1Res, ZeroRes);

        // Pred 2 = 1
        type Pred2 = App<LPred, Two>;
        type Pred2Res = App<App<Pred2, F>, X>;
        type OneRes = App<App<One, F>, X>;
        assert_type_eq_all!(Pred2Res, OneRes);

        // Sub 3 - 2 = 1
        type Diff = App<App<LSub, Three>, Two>;
        type DiffRes = App<App<Diff, F>, X>;
        assert_type_eq_all!(DiffRes, OneRes);

        // Sub 2 - 1 = 1
        type Diff2 = App<App<LSub, Two>, One>;
        type Diff2Res = App<App<Diff2, F>, X>;
        assert_type_eq_all!(Diff2Res, OneRes);
    }
}
