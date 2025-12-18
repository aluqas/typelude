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

/// Mul: \m n. m (Add n) Zero
/// Result = m (Add n) Zero
///
/// Note: \m n f x. m (n f) x is simpler def: Mul m n = m \circ n
/// But `m (Add n) Zero` works too.
/// Let's use `Mul m n = m (Add n) Zero` because Add n is partially applied.
/// Add n x = n + x.
/// m (Add n) Zero -> Apply (Add n) m times to Zero.
/// 0: Zero
/// 1: Add n Zero = n
/// 2: Add n n = 2n
/// ...
/// This requires `Add<N>` to be an `Apply` implementor.
/// `Add<M, N>` is a type alias. We need `AddPart<N>` struct.
pub type Mul<M, N> = <<M as Apply<AddPart<N>>>::Output as Apply<Zero>>::Output;

pub struct AddPart<N>(PhantomData<N>);
// AddPart<N> X -> Add<N, X>
impl<N, X> Apply<X> for AddPart<N>
where
    N: Apply<SuccGen>,
    <N as Apply<SuccGen>>::Output: Apply<X>,
{
    type Output = Add<N, X>;
}

/// Exp: \m n. n m
/// Exponentiation m^n. Note: Order is tricky. Usually Exp m n = n m.
/// 2^3 = 3 2.
/// 2: \f x. f(f x)
/// 3: \f x. f(f(f x))
/// 3 2 -> 2 applied 3 times. 2(2(2(...))) -> 2^3.
/// Correct.
pub type Exp<M, N> = <N as Apply<M>>::Output;

// =========================================================================
// Predecessor and Subtraction
// =========================================================================

/// Pred: \n. Fst (n (\p. Pair (Snd p) (Succ (Snd p))) (Pair Zero Zero))
/// Logic:
/// Start with Pair(0, 0).
/// Step: (a, b) -> (b, b+1).
/// After n steps: (n-1, n).
/// Fst gives n-1. (For 0, it gives 0).
///
/// Needed Step Function: `PredStep`
pub type Pred<N> = Fst<
    <<N as Apply<PredStep>>::Output as Apply<Pair<Zero, Zero>>>::Output
>;

pub struct PredStep;
// PredStep p -> Pair (Snd p) (Succ (Snd p))
impl<P> Apply<P> for PredStep
where
    P: Apply<False>, // Snd
    <P as Apply<False>>::Output: Apply<SuccGen>, // Succ(Snd)
{
    type Output = Pair<Snd<P>, Succ<Snd<P>>>;
}

/// Sub: \m n. n Pred m
/// m - n = Apply Pred n times to m.
pub type Sub<M, N> = <<N as Apply<PredGen>>::Output as Apply<M>>::Output;

pub struct PredGen;
impl<N> Apply<N> for PredGen
where
    N: Apply<PredStep>,
    <N as Apply<PredStep>>::Output: Apply<Pair<Zero, Zero>>,
    <<N as Apply<PredStep>>::Output as Apply<Pair<Zero, Zero>>>::Output: Apply<True>,
{
    type Output = Pred<N>;
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

    // Numbers
    type One = Succ<Zero>;
    type Two = Succ<One>;
    type Three = Succ<Two>;

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

        type ResOne = App<App<One, F>, X>;
        assert_type_eq_all!(ResOne, F1<X>);

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

        type Sum = Add<One, One>;
        type ResSum = App<App<Sum, F>, X>;
        assert_type_eq_all!(ResSum, F1<F1<X>>);
    }

    #[test]
    fn test_church_mul() {
        // 2 * 3 = 6? Too deep?
        // 2 * 1 = 2
        type Prod = Mul<Two, One>;
        // Verify structure: Prod F X == Two F X
        struct F; struct X;
        struct F1<T>(std::marker::PhantomData<T>);
        impl<T> Apply<T> for F { type Output = F1<T>; }

        type ResProd = App<App<Prod, F>, X>;
        type ResTwo = App<App<Two, F>, X>;
        assert_type_eq_all!(ResProd, ResTwo);
    }

    #[test]
    fn test_church_exp() {
        // 2^1 = 2
        type Pow = Exp<Two, One>; // One Two -> Two applied 1 time -> Two.

        struct F; struct X;
        struct F1<T>(std::marker::PhantomData<T>);
        impl<T> Apply<T> for F { type Output = F1<T>; }

        type ResPow = App<App<Pow, F>, X>;
        type ResTwo = App<App<Two, F>, X>;
        assert_type_eq_all!(ResPow, ResTwo);
    }

    #[test]
    fn test_church_pred() {
        // Pred 1 = 0
        type P1 = Pred<One>;
        assert_type_eq_all!(P1, Zero);

        // Pred 2 = 1
        type P2 = Pred<Two>;
        // Check structural equality by applying
        struct F; struct X;
        struct F1<T>(std::marker::PhantomData<T>);
        impl<T> Apply<T> for F { type Output = F1<T>; }

        type ResP2 = App<App<P2, F>, X>;
        type ResOne = App<App<One, F>, X>;
        assert_type_eq_all!(ResP2, ResOne);
    }

    #[test]
    fn test_church_sub() {
        // 2 - 1 = 1
        type Diff = Sub<Two, One>;

        struct F; struct X;
        struct F1<T>(std::marker::PhantomData<T>);
        impl<T> Apply<T> for F { type Output = F1<T>; }

        type ResDiff = App<App<Diff, F>, X>;
        type ResOne = App<App<One, F>, X>;
        assert_type_eq_all!(ResDiff, ResOne);
    }
}
