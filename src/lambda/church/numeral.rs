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
use crate::{
    kernel::traits::Apply,
    lambda::{
        Lambda,
        traits::{LNat, LTerm},
    },
};

// =========================================================================
// Church Numerals
// =========================================================================

/// Zero: λf x. x
pub struct LZero;
impl LTerm for LZero {}
impl LNat for LZero {}

impl Lambda for LZero {
    type Output = LZero;
}

/// Succ: λn f x. f (n f x)
pub struct LSucc<N>(PhantomData<N>);
impl<N: LNat> LTerm for LSucc<N> {}
impl<N: LNat> LNat for LSucc<N> {}

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

// --- Add: \m n f x. m f (n f x) ---
// M f (N f x)
// --- Add: \m n f x. m f (n f x) ---
// M f (N f x)
pub type LPureAdd<M, N> = LAdd2<M, N>;

pub struct LAdd;

// Add is a Lambda
impl Lambda for LAdd {
    type Output = LAdd;
}

// Add M N -> Add1<M, N>
impl<M> Apply<M> for LAdd {
    type Output = LAdd1<M>;
}
pub struct LAdd1<M>(PhantomData<M>);
impl<M> Lambda for LAdd1<M> {
    type Output = LAdd1<M>;
}

impl<M, N> Apply<N> for LAdd1<M> {
    type Output = LAdd2<M, N>;
}

// Add2 is the numeral: \f x. m f (n f x)
pub struct LAdd2<M, N>(PhantomData<(M, N)>);
impl<M, N> Lambda for LAdd2<M, N> {
    type Output = LAdd2<M, N>;
}

impl<M, N, F> Apply<F> for LAdd2<M, N> {
    type Output = LAdd3<M, N, F>;
}
pub struct LAdd3<M, N, F>(PhantomData<(M, N, F)>);
impl<M, N, F> Lambda for LAdd3<M, N, F> {
    type Output = LAdd3<M, N, F>;
}

impl<M, N, F, X> Apply<X> for LAdd3<M, N, F>
where
    // n f x
    N: Apply<F>,
    <N as Apply<F>>::Output: Apply<X>,
    // m f (n f x)
    M: Apply<F>,
    <M as Apply<F>>::Output: Apply<<<N as Apply<F>>::Output as Apply<X>>::Output>,
{
    type Output =
        <<M as Apply<F>>::Output as Apply<<<N as Apply<F>>::Output as Apply<X>>::Output>>::Output;
}

// --- Mul: \m n f x. m (n f) x ---
pub type LPureMul<M, N> = LMul2<M, N>;

pub struct LMul;
impl Lambda for LMul {
    type Output = LMul;
}

// Mul M N -> Mul1<M, N>
impl<M> Apply<M> for LMul {
    type Output = LMul1<M>;
}
pub struct LMul1<M>(PhantomData<M>);
impl<M> Lambda for LMul1<M> {
    type Output = LMul1<M>;
}

impl<M, N> Apply<N> for LMul1<M> {
    type Output = LMul2<M, N>;
}

// Mul2 is the numeral: \f x. m (n f) x
pub struct LMul2<M, N>(PhantomData<(M, N)>);
impl<M, N> Lambda for LMul2<M, N> {
    type Output = LMul2<M, N>;
}

impl<M, N, F> Apply<F> for LMul2<M, N>
where
    // n f -> nf (composed function)
    N: Apply<F>,
{
    type Output = LMul3<M, <N as Apply<F>>::Output>;
}
pub struct LMul3<M, NF>(PhantomData<(M, NF)>);
impl<M, NF> Lambda for LMul3<M, NF> {
    type Output = LMul3<M, NF>;
}

impl<M, NF, X> Apply<X> for LMul3<M, NF>
where
    // m (nf) x
    M: Apply<NF>,
    <M as Apply<NF>>::Output: Apply<X>,
{
    type Output = <<M as Apply<NF>>::Output as Apply<X>>::Output;
}
// AddPart removed as it was for the old implementation

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
pub type LPurePred<N> =
    super::pair::LPureFst<<<N as Apply<LPredStep>>::Output as Apply<LPair<LZero, LZero>>>::Output>;

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
    P: Apply<super::bool::LFalse>,                              // Snd
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

        type Sum = Evaluate<App<App<LAdd, One>, One>>;
        type ResSum = App<App<Sum, F>, X>;
        assert_type_eq_all!(ResSum, F1<F1<X>>);
    }
}
