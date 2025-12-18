//! Church Numerals and Arithmetic
//!
//! Church encoding of natural numbers and arithmetic operations:
//! - `Zero`: λf x. x
//! - `Succ<N>`: λn f x. f (n f x)

use std::marker::PhantomData;

use super::{
    bool::LTrue,
    pair::{LPair, LSndEval},
};
use crate::{kernel::traits::Apply, lambda::Lambda};

// =========================================================================
// Church Numerals
// =========================================================================

/// Zero: λf x. x
pub struct LZero;

impl Lambda for LZero {
    type Output = LZero;
}

/// Succ: λn f x. f (n f x)
pub struct LSucc<N>(PhantomData<N>);

impl<N> Lambda for LSucc<N> {
    type Output = LSucc<N>;
}

// Partial Application States
pub struct LZero1<F>(PhantomData<F>);
pub struct LSucc1<N, F>(PhantomData<(N, F)>);

impl<F> Apply<F> for LZero {
    type Output = LZero1<F>;
}
impl<F, X> Apply<X> for LZero1<F> {
    type Output = X;
}

impl<N, F> Apply<F> for LSucc<N> {
    type Output = LSucc1<N, F>;
}

impl<N, F, X> Apply<X> for LSucc1<N, F>
where
    N: Apply<F>,
    <N as Apply<F>>::Output: Apply<X>,
    F: Apply<<<N as Apply<F>>::Output as Apply<X>>::Output>,
{
    type Output = <F as Apply<<<N as Apply<F>>::Output as Apply<X>>::Output>>::Output;
}

/// SuccGen: Generates Succ<N> from N
pub struct LSuccGen;

impl<N> Apply<N> for LSuccGen {
    type Output = LSucc<N>;
}

// =========================================================================
// Arithmetic Operations
// =========================================================================

// --- Add ---
pub type LPureAdd<M, N> = <<M as Apply<LSuccGen>>::Output as Apply<N>>::Output;

pub struct LAdd<M, N>(PhantomData<(M, N)>);

impl<M, N> Lambda for LAdd<M, N>
where
    M: Apply<LSuccGen>,
    <M as Apply<LSuccGen>>::Output: Apply<N>,
{
    type Output = LPureAdd<M, N>;
}

// --- Mul ---
pub type LPureMul<M, N> = <<M as Apply<LAddPart<N>>>::Output as Apply<LZero>>::Output;

pub struct LMul<M, N>(PhantomData<(M, N)>);

impl<M, N> Lambda for LMul<M, N>
where
    M: Apply<LAddPart<N>>,
    <M as Apply<LAddPart<N>>>::Output: Apply<LZero>,
{
    type Output = LPureMul<M, N>;
}

pub struct LAddPart<N>(PhantomData<N>);

impl<N, X> Apply<X> for LAddPart<N>
where
    N: Apply<LSuccGen>,
    <N as Apply<LSuccGen>>::Output: Apply<X>,
{
    type Output = LPureAdd<N, X>;
}

// --- Exp ---
pub type LPureExp<M, N> = <N as Apply<M>>::Output;

pub struct LExp<M, N>(PhantomData<(M, N)>);

impl<M, N> Lambda for LExp<M, N>
where
    N: Apply<M>,
{
    type Output = LPureExp<M, N>;
}

// =========================================================================
// Predecessor and Subtraction
// =========================================================================

// --- Pred ---
pub type LPurePred<N> = super::pair::LPureFst<
    <<N as Apply<LPredStep>>::Output as Apply<LPair<LZero, LZero>>>::Output,
>;

pub struct LPred<N>(PhantomData<N>);

impl<N> Lambda for LPred<N>
where
    N: Apply<LPredStep>,
    <N as Apply<LPredStep>>::Output: Apply<LPair<LZero, LZero>>,
    <<N as Apply<LPredStep>>::Output as Apply<LPair<LZero, LZero>>>::Output: Apply<LTrue>,
{
    type Output = LPurePred<N>;
}

pub struct LPredStep;

impl<P> Apply<P> for LPredStep
where
    P: Apply<super::bool::LFalse>,                               // Snd
    <P as Apply<super::bool::LFalse>>::Output: Apply<LSuccGen>, // Succ(Snd)
{
    type Output = LPair<LSndEval<P>, LSucc<LSndEval<P>>>;
}

// --- Sub ---
pub type LPureSub<M, N> = <<N as Apply<LPredGen>>::Output as Apply<M>>::Output;

pub struct LSub<M, N>(PhantomData<(M, N)>);

impl<M, N> Lambda for LSub<M, N>
where
    N: Apply<LPredGen>,
    <N as Apply<LPredGen>>::Output: Apply<M>,
{
    type Output = LPureSub<M, N>;
}

pub struct LPredGen;

impl<N> Apply<N> for LPredGen
where
    N: Apply<LPredStep>,
    <N as Apply<LPredStep>>::Output: Apply<LPair<LZero, LZero>>,
    <<N as Apply<LPredStep>>::Output as Apply<LPair<LZero, LZero>>>::Output: Apply<LTrue>,
{
    type Output = LPurePred<N>;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::eval::Evaluate;

    type App<F, A> = <F as Apply<A>>::Output;

    // Numbers
    type One = LSucc<LZero>;
    type Two = LSucc<One>;

    #[test]
    fn test_church_numerals_basic() {
        struct F;
        struct X;
        struct F1<T>(std::marker::PhantomData<T>);
        impl<T> Apply<T> for F {
            type Output = F1<T>;
        }

        type ResZero = App<App<LZero, F>, X>;
        assert_type_eq_all!(ResZero, X);
    }

    #[test]
    fn test_church_add() {
        struct F;
        struct X;
        struct F1<T>(std::marker::PhantomData<T>);
        impl<T> Apply<T> for F {
            type Output = F1<T>;
        }

        type Sum = Evaluate<LAdd<One, One>>;
        type ResSum = App<App<Sum, F>, X>;
        assert_type_eq_all!(ResSum, F1<F1<X>>);
    }
}
