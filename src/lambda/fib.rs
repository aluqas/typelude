use std::marker::PhantomData;

use super::{Apply, church::{Pair, Zero, Succ, Add, Fst, Snd, Zero1, Succ1, Succ2}};

// =========================================================================
// Fibonacci Calculation (Iterative Pair Approach)
// =========================================================================

/// Fib: \n. Fst (n FibStep (Pair Zero One))
///
/// Start: (0, 1)
/// Step: (a, b) -> (b, a+b)
/// End: (F(n), F(n+1))
/// Result: Fst -> F(n)
///
/// Note: Fib(0) -> Fst (0 step (0,1)) -> Fst(0,1) -> 0. Correct.
/// Note: Fib(1) -> Fst (1 step (0,1)) -> Fst(1,1) -> 1. Correct.
/// Note: Fib(2) -> Fst (2 step (0,1)) -> Fst(1,2) -> 1. Correct.
/// Note: Fib(3) -> Fst (3 step (0,1)) -> Fst(2,3) -> 2. Correct.
pub type Fib<N> = Fst<
    <<N as Apply<FibStep>>::Output as Apply<Pair<Zero, Succ<Zero>>>>::Output
>;

/// Step Function: \p. Pair (Snd p) (Add (Fst p) (Snd p))
pub struct FibStep;

impl<P> Apply<P> for FibStep
where
    P: Apply<crate::lambda::church::False>, // Snd
    P: Apply<crate::lambda::church::True>,  // Fst
    <P as Apply<crate::lambda::church::False>>::Output: Apply<crate::lambda::church::SuccGen>, // Add Support
    // Add<Fst, Snd>
    // Fst<P> must be applicable to AddPart<Snd<P>>
    // AddPart<Snd<P>> defined in church.rs? Yes but private?
    // We used Add<M, N> type alias.
    // Add<Fst<P>, Snd<P>> = Fst<P> (SuccGen) Snd<P>
    // Wait, Add definition: <<M as Apply<SuccGen>>::Output as Apply<N>>::Output
    // So:
    Fst<P>: Apply<crate::lambda::church::SuccGen>,
    <Fst<P> as Apply<crate::lambda::church::SuccGen>>::Output: Apply<Snd<P>>,
{
    type Output = Pair<
        Snd<P>,
        Add<Fst<P>, Snd<P>>
    >;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use crate::lambda::church::{Zero, Succ};
    use super::*;

    type App<F, A> = <F as Apply<A>>::Output;

    // Numbers
    type One = Succ<Zero>;
    type Two = Succ<One>;
    type Three = Succ<Two>;
    type Four = Succ<Three>;
    type Five = Succ<Four>;
    type Eight = Add<Four, Four>; // Fib(6) = 8. Need Add to construct expected 8? Or just recursive Succ.

    #[test]
    fn test_fib_small() {
        // Fib 0 = 0
        type F0 = Fib<Zero>;
        assert_type_eq_all!(F0, Zero);

        // Fib 1 = 1
        type ResFib1 = Fib<One>;
        assert_type_eq_all!(ResFib1, One);

        // Fib 2 = 1
        type F2 = Fib<Two>;
        // Use structural check
        struct F; struct X;
        struct TestF1<T>(std::marker::PhantomData<T>);
        impl<T> Apply<T> for F { type Output = TestF1<T>; }

        type ResF2 = App<App<F2, F>, X>;
        type ResOne = App<App<One, F>, X>;
        assert_type_eq_all!(ResF2, ResOne);

        // Fib 3 = 2
        type F3 = Fib<Three>;
        type ResF3 = App<App<F3, F>, X>;
        type ResTwo = App<App<Two, F>, X>;
        assert_type_eq_all!(ResF3, ResTwo);

        // Fib 4 = 3
        type F4 = Fib<Four>;
        type ResF4 = App<App<F4, F>, X>;
        type ResThree = App<App<Three, F>, X>;
        assert_type_eq_all!(ResF4, ResThree);

        // Fib 5 = 5
        type F5 = Fib<Five>;
        type ResF5 = App<App<F5, F>, X>;
        type ResFive = App<App<Five, F>, X>;
        assert_type_eq_all!(ResF5, ResFive);
    }
}
