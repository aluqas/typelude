//! Church Numerals and Arithmetic
//!
//! Church encoding of natural numbers and arithmetic operations:
//! - `Zero`: λf x. x
//! - `Succ<N>`: λn f x. f (n f x)

use std::marker::PhantomData;

use super::bool::True;
use super::pair::{Pair, SndEval};
use crate::kernel::traits::Apply;
use crate::lambda::Lambda;

// =========================================================================
// Church Numerals
// =========================================================================

/// Zero: λf x. x
pub struct Zero;

impl Lambda for Zero {
    type Output = Zero;
}

/// Succ: λn f x. f (n f x)
pub struct Succ<N>(PhantomData<N>);

impl<N> Lambda for Succ<N> {
    type Output = Succ<N>;
}

// Partial Application States
pub struct Zero1<F>(PhantomData<F>);
pub struct Succ1<N, F>(PhantomData<(N, F)>);

impl<F> Apply<F> for Zero {
    type Output = Zero1<F>;
}
impl<F, X> Apply<X> for Zero1<F> {
    type Output = X;
}

impl<N, F> Apply<F> for Succ<N> {
    type Output = Succ1<N, F>;
}

impl<N, F, X> Apply<X> for Succ1<N, F>
where
    N: Apply<F>,
    <N as Apply<F>>::Output: Apply<X>,
    F: Apply<<<N as Apply<F>>::Output as Apply<X>>::Output>,
{
    type Output = <F as Apply<<<N as Apply<F>>::Output as Apply<X>>::Output>>::Output;
}

/// SuccGen: Generates Succ<N> from N
pub struct SuccGen;

impl<N> Apply<N> for SuccGen {
    type Output = Succ<N>;
}

// =========================================================================
// Arithmetic Operations
// =========================================================================

// --- Add ---
pub type PureAdd<M, N> = <<M as Apply<SuccGen>>::Output as Apply<N>>::Output;

pub struct Add<M, N>(PhantomData<(M, N)>);

impl<M, N> Lambda for Add<M, N>
where
    M: Apply<SuccGen>,
    <M as Apply<SuccGen>>::Output: Apply<N>,
{
    type Output = PureAdd<M, N>;
}

// --- Mul ---
pub type PureMul<M, N> = <<M as Apply<AddPart<N>>>::Output as Apply<Zero>>::Output;

pub struct Mul<M, N>(PhantomData<(M, N)>);

impl<M, N> Lambda for Mul<M, N>
where
    M: Apply<AddPart<N>>,
    <M as Apply<AddPart<N>>>::Output: Apply<Zero>,
{
    type Output = PureMul<M, N>;
}

pub struct AddPart<N>(PhantomData<N>);

impl<N, X> Apply<X> for AddPart<N>
where
    N: Apply<SuccGen>,
    <N as Apply<SuccGen>>::Output: Apply<X>,
{
    type Output = PureAdd<N, X>;
}

// --- Exp ---
pub type PureExp<M, N> = <N as Apply<M>>::Output;

pub struct Exp<M, N>(PhantomData<(M, N)>);

impl<M, N> Lambda for Exp<M, N>
where
    N: Apply<M>,
{
    type Output = PureExp<M, N>;
}

// =========================================================================
// Predecessor and Subtraction
// =========================================================================

// --- Pred ---
pub type PurePred<N> =
    super::pair::PureFst<<<N as Apply<PredStep>>::Output as Apply<Pair<Zero, Zero>>>::Output>;

pub struct Pred<N>(PhantomData<N>);

impl<N> Lambda for Pred<N>
where
    N: Apply<PredStep>,
    <N as Apply<PredStep>>::Output: Apply<Pair<Zero, Zero>>,
    <<N as Apply<PredStep>>::Output as Apply<Pair<Zero, Zero>>>::Output: Apply<True>,
{
    type Output = PurePred<N>;
}

pub struct PredStep;

impl<P> Apply<P> for PredStep
where
    P: Apply<super::bool::False>,               // Snd
    <P as Apply<super::bool::False>>::Output: Apply<SuccGen>, // Succ(Snd)
{
    type Output = Pair<SndEval<P>, Succ<SndEval<P>>>;
}

// --- Sub ---
pub type PureSub<M, N> = <<N as Apply<PredGen>>::Output as Apply<M>>::Output;

pub struct Sub<M, N>(PhantomData<(M, N)>);

impl<M, N> Lambda for Sub<M, N>
where
    N: Apply<PredGen>,
    <N as Apply<PredGen>>::Output: Apply<M>,
{
    type Output = PureSub<M, N>;
}

pub struct PredGen;

impl<N> Apply<N> for PredGen
where
    N: Apply<PredStep>,
    <N as Apply<PredStep>>::Output: Apply<Pair<Zero, Zero>>,
    <<N as Apply<PredStep>>::Output as Apply<Pair<Zero, Zero>>>::Output: Apply<True>,
{
    type Output = PurePred<N>;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::eval::Evaluate;

    type App<F, A> = <F as Apply<A>>::Output;

    // Numbers
    type One = Succ<Zero>;
    type Two = Succ<One>;

    #[test]
    fn test_church_numerals_basic() {
        struct F;
        struct X;
        struct F1<T>(std::marker::PhantomData<T>);
        impl<T> Apply<T> for F {
            type Output = F1<T>;
        }

        type ResZero = App<App<Zero, F>, X>;
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

        type Sum = Evaluate<Add<One, One>>;
        type ResSum = App<App<Sum, F>, X>;
        assert_type_eq_all!(ResSum, F1<F1<X>>);
    }
}
