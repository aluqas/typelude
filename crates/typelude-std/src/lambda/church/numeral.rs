//! Church Numerals and Arithmetic
//!
//! Church encoding of natural numbers and arithmetic operations.

use std::marker::PhantomData;

use typelude_core::{Eval, Evaluate};

use crate::{
    impl_eval_for_lambda, impl_eval_for_lambda_generic,
    lambda::{
        LApp,
        traits::{LNat, LTerm, Lambda},
    },
};
/// Zero: λf x. x
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LZero;
impl LTerm for LZero {}
impl LNat for LZero {}

impl Lambda for LZero {
    type Output = LZero;
}
impl_eval_for_lambda!(LZero);

/// Succ: λn f x. f (n f x)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LSucc<N>(PhantomData<N>);
impl<N> LTerm for LSucc<N> {}
impl<N> LNat for LSucc<N> {}

impl<N> Lambda for LSucc<N> {
    type Output = LSucc<N>;
}
impl_eval_for_lambda_generic!(LSucc, [N]);

// --- Partial Application States (Value Types) ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LZero1<F>(PhantomData<F>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LSucc1<N, F>(PhantomData<(N, F)>);

impl<F> Lambda for LZero1<F> {
    type Output = LZero1<F>;
}
impl_eval_for_lambda_generic!(LZero1, [F]);

impl<N, F> Lambda for LSucc1<N, F> {
    type Output = LSucc1<N, F>;
}
impl_eval_for_lambda_generic!(LSucc1, [N, F]);
// --- Zero ---
// Zero F -> Zero1<F>
impl<F> Lambda for LApp<LZero, F> {
    type Output = LZero1<F>;
}

// Zero1<F> X -> X
impl<F, X> Lambda for LApp<LZero1<F>, X>
where
    X: Eval,
{
    type Output = Evaluate<X>;
}

// --- Succ ---
// Succ<N> F -> Succ1<N, F>
impl<N, F> Lambda for LApp<LSucc<N>, F> {
    type Output = LSucc1<N, F>;
}

// Succ1<N, F> X -> F (N F X)
impl<N, F, X> Lambda for LApp<LSucc1<N, F>, X>
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

impl Lambda for LSuccGen {
    type Output = LSuccGen;
}
impl_eval_for_lambda!(LSuccGen);

// SuccGen N -> Succ<N>
impl<N> Lambda for LApp<LSuccGen, N> {
    type Output = LSucc<N>;
}
// --- Add: λm n f x. m f (n f x) ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LAdd;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LAdd1<M>(PhantomData<M>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LAdd2<M, N>(PhantomData<(M, N)>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LAdd3<M, N, F>(PhantomData<(M, N, F)>);

impl Lambda for LAdd {
    type Output = LAdd;
}
impl_eval_for_lambda!(LAdd);

impl<M> Lambda for LAdd1<M> {
    type Output = LAdd1<M>;
}
impl_eval_for_lambda_generic!(LAdd1, [M]);

impl<M, N> Lambda for LAdd2<M, N> {
    type Output = LAdd2<M, N>;
}
impl_eval_for_lambda_generic!(LAdd2, [M, N]);

impl<M, N, F> Lambda for LAdd3<M, N, F> {
    type Output = LAdd3<M, N, F>;
}
impl_eval_for_lambda_generic!(LAdd3, [M, N, F]);

// Add M -> Add1<M>
impl<M> Lambda for LApp<LAdd, M> {
    type Output = LAdd1<M>;
}

// Add1<M> N -> Add2<M, N>
impl<M, N> Lambda for LApp<LAdd1<M>, N> {
    type Output = LAdd2<M, N>;
}

// Add2<M, N> F -> Add3<M, N, F>
impl<M, N, F> Lambda for LApp<LAdd2<M, N>, F> {
    type Output = LAdd3<M, N, F>;
}

// Add3<M, N, F> X -> M F (N F X)
impl<M, N, F, X> Lambda for LApp<LAdd3<M, N, F>, X>
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

impl Lambda for LMul {
    type Output = LMul;
}
impl_eval_for_lambda!(LMul);

impl<M> Lambda for LMul1<M> {
    type Output = LMul1<M>;
}
impl_eval_for_lambda_generic!(LMul1, [M]);

impl<M, N> Lambda for LMul2<M, N> {
    type Output = LMul2<M, N>;
}
impl_eval_for_lambda_generic!(LMul2, [M, N]);

impl<M, N, F> Lambda for LMul3<M, N, F> {
    type Output = LMul3<M, N, F>;
}
impl_eval_for_lambda_generic!(LMul3, [M, N, F]);

// Mul M -> Mul1<M>
impl<M> Lambda for LApp<LMul, M> {
    type Output = LMul1<M>;
}

// Mul1<M> N -> Mul2<M, N>
impl<M, N> Lambda for LApp<LMul1<M>, N> {
    type Output = LMul2<M, N>;
}

// Mul2<M, N> F -> Mul3<M, N, F>
impl<M, N, F> Lambda for LApp<LMul2<M, N>, F> {
    type Output = LMul3<M, N, F>;
}

// Mul3<M, N, F> X -> M (N F) X
impl<M, N, F, X> Lambda for LApp<LMul3<M, N, F>, X>
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

impl Lambda for LExp {
    type Output = LExp;
}
impl_eval_for_lambda!(LExp);

impl<M> Lambda for LExp1<M> {
    type Output = LExp1<M>;
}
impl_eval_for_lambda_generic!(LExp1, [M]);

impl<M, N> Lambda for LExp2<M, N> {
    type Output = LExp2<M, N>;
}
impl_eval_for_lambda_generic!(LExp2, [M, N]);

impl<M, N, F> Lambda for LExp3<M, N, F> {
    type Output = LExp3<M, N, F>;
}
impl_eval_for_lambda_generic!(LExp3, [M, N, F]);

// Exp M -> Exp1<M>
impl<M> Lambda for LApp<LExp, M> {
    type Output = LExp1<M>;
}

// Exp1<M> N -> Exp2<M, N>
impl<M, N> Lambda for LApp<LExp1<M>, N> {
    type Output = LExp2<M, N>;
}

// Exp2<M, N> F -> Exp3<M, N, F>
impl<M, N, F> Lambda for LApp<LExp2<M, N>, F> {
    type Output = LExp3<M, N, F>;
}

// Exp3<M, N, F> X -> ((N M) F) X
impl<M, N, F, X> Lambda for LApp<LExp3<M, N, F>, X>
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LPred;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LPredStep;

impl Lambda for LPred {
    type Output = LPred;
}
impl_eval_for_lambda!(LPred);

impl Lambda for LPredStep {
    type Output = LPredStep;
}
impl_eval_for_lambda!(LPredStep);

// Pred N -> Fst (N PredStep (Pair Zero Zero))
impl<N> Lambda for LApp<LPred, N>
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
impl<P> Lambda for LApp<LPredStep, P>
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

impl Lambda for LSub {
    type Output = LSub;
}
impl_eval_for_lambda!(LSub);

impl<M> Lambda for LSub1<M> {
    type Output = LSub1<M>;
}
impl_eval_for_lambda_generic!(LSub1, [M]);

// Sub M -> Sub1<M>
impl<M> Lambda for LApp<LSub, M> {
    type Output = LSub1<M>;
}

// Sub1<M> N -> (N Pred) M
impl<M, N> Lambda for LApp<LSub1<M>, N>
where
    // (N Pred)
    LApp<N, LPred>: Eval,
    // ((N Pred) M)
    LApp<Evaluate<LApp<N, LPred>>, M>: Eval,
{
    type Output = Evaluate<LApp<Evaluate<LApp<N, LPred>>, M>>;
}
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
    type Four = LSucc<Three>;
    type Five = LSucc<Four>;
    type Ten = LSucc<LSucc<LSucc<LSucc<LSucc<Five>>>>>;

    // Helper to apply number N to F and X
    // N F X
    type ToType<N, F, X> = App<App<N, F>, X>;

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

    #[test]
    fn test_church_numerals_basic() {
        // Zero F X -> X
        type ResZero = ToType<LZero, F, X>;
        assert_type_eq_all!(ResZero, X);

        // One F X -> F X -> F1<X>
        type ResOne = ToType<One, F, X>;
        assert_type_eq_all!(ResOne, F1<X>);

        // SuccGen
        type TwoComputed = App<LSuccGen, One>;
        assert_type_eq_all!(TwoComputed, Two);
    }

    #[test]
    fn test_church_arithmetic_small() {
        // Add 1 + 2 = 3
        type Sum = App<App<LAdd, One>, Two>;
        type SumRes = ToType<Sum, F, X>;
        type ThreeRes = ToType<Three, F, X>;
        assert_type_eq_all!(SumRes, ThreeRes);

        // Mul 2 * 2 = 4
        type Prod = App<App<LMul, Two>, Two>;
        type ProdRes = ToType<Prod, F, X>;
        type FourRes = ToType<Four, F, X>;
        assert_type_eq_all!(ProdRes, FourRes);

        // Exp 2 ^ 2 = 4
        type Pow = App<App<LExp, Two>, Two>;
        type PowRes = ToType<Pow, F, X>;
        assert_type_eq_all!(PowRes, FourRes);
    }

    #[test]
    fn ignore_test_church_arithmetic_large() {
        // Add 5 + 5 = 10
        type Sum = App<App<LAdd, Five>, Five>;
        type SumRes = ToType<Sum, F, X>;
        type TenRes = ToType<Ten, F, X>;
        assert_type_eq_all!(SumRes, TenRes);

        // Mul 2 * 5 = 10
        type Prod = App<App<LMul, Two>, Five>;
        type ProdRes = ToType<Prod, F, X>;
        assert_type_eq_all!(ProdRes, TenRes);

        // Exp 2 ^ 3 = 8
        type Eight = LSucc<LSucc<LSucc<Five>>>;
        type Pow = App<App<LExp, Two>, Three>;
        type PowRes = ToType<Pow, F, X>;
        type EightRes = ToType<Eight, F, X>;
        assert_type_eq_all!(PowRes, EightRes);
    }

    #[test]
    fn test_church_pred_sub() {
        // Pred 5 = 4
        type Pred5 = App<LPred, Five>;
        type Pred5Res = ToType<Pred5, F, X>;
        type FourRes = ToType<Four, F, X>;
        assert_type_eq_all!(Pred5Res, FourRes);

        // Sub 5 - 2 = 3
        type Diff = App<App<LSub, Five>, Two>;
        type DiffRes = ToType<Diff, F, X>;
        type ThreeRes = ToType<Three, F, X>;
        assert_type_eq_all!(DiffRes, ThreeRes);

        // Sub 10 - 5 = 5
        type Diff2 = App<App<LSub, Ten>, Five>;
        type Diff2Res = ToType<Diff2, F, X>;
        type FiveRes = ToType<Five, F, X>;
        assert_type_eq_all!(Diff2Res, FiveRes);
    }
}
