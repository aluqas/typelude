use std::marker::PhantomData;

use super::{Apply, Lambda};

// =========================================================================
// Church Booleans
// =========================================================================

/// Church True: \t f. t
pub struct True;
/// Church False: \t f. f
pub struct False;

impl Lambda for True {
    type Output = True;
}
impl Lambda for False {
    type Output = False;
}

// Partial Application States
pub struct True1<T>(PhantomData<T>);
pub struct False1<T>(PhantomData<T>);

// --- True Implementation ---
// \t f. t
impl<T> Apply<T> for True {
    type Output = True1<T>;
}
impl<T, F> Apply<F> for True1<T> {
    type Output = T;
}

// --- False Implementation ---
// \t f. f
impl<T> Apply<T> for False {
    type Output = False1<T>;
}
impl<T, F> Apply<F> for False1<T> {
    type Output = F;
}

// --- If ---
/// Pure If Alias: ((P T) E)
pub type PureIf<P, T, E> = <<P as Apply<T>>::Output as Apply<E>>::Output;

/// Church If Struct
pub struct If<P, T, E>(PhantomData<(P, T, E)>);

impl<P, T, E> Lambda for If<P, T, E>
where
    P: Apply<T>,
    <P as Apply<T>>::Output: Apply<E>,
{
    type Output = PureIf<P, T, E>;
}

// =========================================================================
// Church Numerals
// =========================================================================

/// Zero: \f x. x
pub struct Zero;
impl Lambda for Zero {
    type Output = Zero;
}

/// Succ: \n f x. f (n f x)
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
    PureFst<<<N as Apply<PredStep>>::Output as Apply<Pair<Zero, Zero>>>::Output>;

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
    P: Apply<False>,                             // Snd
    <P as Apply<False>>::Output: Apply<SuccGen>, // Succ(Snd)
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
    // Here we use Pred structure logic, but we return a value (PurePred<N>)
    // PredGen is used inside PureSub.
    // PurePred<N> is a type alias, so we must satisfy its bounds.
    // But PurePred is complex alias.
    // Let's rely on Eval bounds for now? No, Apply is pure.
    N: Apply<PredStep>,
    <N as Apply<PredStep>>::Output: Apply<Pair<Zero, Zero>>,
    <<N as Apply<PredStep>>::Output as Apply<Pair<Zero, Zero>>>::Output: Apply<True>,
{
    type Output = PurePred<N>;
}

// =========================================================================
// Church Pairs
// =========================================================================

/// Pair: \x y. \f. f x y
pub struct Pair<X, Y>(PhantomData<(X, Y)>);
impl<X, Y> Lambda for Pair<X, Y> {
    type Output = Pair<X, Y>;
}

/// Pure Fst/Snd Aliases
pub type PureFst<P> = <P as Apply<True>>::Output;
pub type PureSnd<P> = <P as Apply<False>>::Output;

/// Fst Struct
pub struct Fst<P>(PhantomData<P>);
impl<P> Lambda for Fst<P>
where
    P: Apply<True>,
{
    type Output = PureFst<P>;
}

/// Snd Struct
pub struct Snd<P>(PhantomData<P>);
impl<P> Lambda for Snd<P>
where
    P: Apply<False>,
{
    type Output = PureSnd<P>;
}

// Helpers for internal use
type SndEval<P> = <P as Apply<False>>::Output;

// Pair<X, Y> f -> f X Y
impl<X, Y, F> Apply<F> for Pair<X, Y>
where
    F: Apply<X>,
    <F as Apply<X>>::Output: Apply<Y>,
{
    type Output = <<F as Apply<X>>::Output as Apply<Y>>::Output;
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
    fn test_church_bools_basic() {
        struct A;
        struct B;
        // Using Structs
        type TrueRes = Evaluate<If<True, A, B>>;
        type FalseRes = Evaluate<If<False, A, B>>;
        assert_type_eq_all!(TrueRes, A);
        assert_type_eq_all!(FalseRes, B);
    }

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
