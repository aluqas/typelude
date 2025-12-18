use std::marker::PhantomData;

use super::Apply;

// =========================================================================
// Church Booleans
// =========================================================================

/// Church True: \t f. t
pub struct True;

/// Church False: \t f. f
pub struct False;

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

/// Church If: \p t e. p t e
/// Essentially just applying p to t and e.
/// Type Alias for convenience: If<P, T, E> -> ((P T) E)
pub type If<P, T, E> = <<P as Apply<T>>::Output as Apply<E>>::Output;

// =========================================================================
// Church Numerals
// =========================================================================

/// Zero: \f x. x
pub struct Zero;

/// Succ: \n f x. f (n f x)
pub struct Succ<N>(PhantomData<N>);

// Partial Application States
pub struct Zero1<F>(PhantomData<F>);
pub struct Succ1<N, F>(PhantomData<(N, F)>);
pub struct Succ2<N, F, X>(PhantomData<(N, F, X)>);

// --- Zero Implementation ---
// \f x. x (Same structure as False)
impl<F> Apply<F> for Zero {
    type Output = Zero1<F>;
}
impl<F, X> Apply<X> for Zero1<F> {
    type Output = X;
}

// --- Succ Implementation ---
// Succ n f x -> f (n f x)
impl<N, F> Apply<F> for Succ<N> {
    type Output = Succ1<N, F>;
}

impl<N, F, X> Apply<X> for Succ1<N, F>
where
    // Logic: Calculate (n f x), then apply f to it.
    N: Apply<F>,
    <N as Apply<F>>::Output: Apply<X>,
    F: Apply<<<N as Apply<F>>::Output as Apply<X>>::Output>,
{
    type Output = <F as Apply<<<N as Apply<F>>::Output as Apply<X>>::Output>>::Output;
}

// =========================================================================
// Arithmetic Operations
// =========================================================================

/// Add: \m n. m Succ n
/// Adds m to n by applying Succ m times to n.
/// Result = m Succ n
pub type Add<M, N> = <<M as Apply<SuccGen>>::Output as Apply<N>>::Output;

// Helper struct solely to be passed as the 'f' (function) to the Numeral M.
// When a Numeral M is applied to SuccGen, it returns a function that applies SuccGen M times.
pub struct SuccGen; // Represents the "Succ" function conceptually passed to numerals

// SuccGen x -> Succ<x>
// This allows "m Succ n" to construct "Succ<Succ<...n...>>"
impl<N> Apply<N> for SuccGen {
    type Output = Succ<N>;
}

// =========================================================================
// Church Pairs
// =========================================================================

/// Pair: \x y. \f. f x y
pub struct Pair<X, Y>(PhantomData<(X, Y)>);

/// Fst: \p. p True
pub type Fst<P> = <P as Apply<True>>::Output;

/// Snd: \p. p False
pub type Snd<P> = <P as Apply<False>>::Output;

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

    type App<F, A> = <F as Apply<A>>::Output;

    #[test]
    fn test_church_bools_basic() {
        struct A;
        struct B;
        type TrueRes = If<True, A, B>;
        type FalseRes = If<False, A, B>;
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

        type One = Succ<Zero>;
        type ResOne = App<App<One, F>, X>;
        assert_type_eq_all!(ResOne, F1<X>);

        type Two = Succ<One>;
        type ResTwo = App<App<Two, F>, X>;
        assert_type_eq_all!(ResTwo, F1<F1<X>>);
    }

    #[test]
    fn test_church_add() {
        struct F;
        struct X;
        struct F1<T>(std::marker::PhantomData<T>);
        impl<T> Apply<T> for F {
            type Output = F1<T>;
        }

        type One = Succ<Zero>;
        type Sum = Add<One, One>;
        type ResSum = App<App<Sum, F>, X>;
        assert_type_eq_all!(ResSum, F1<F1<X>>);
    }
}
