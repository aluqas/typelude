//! Fibonacci test using Lambda/Eval pattern

use typelude::{
    eval::Evaluate,
    lambda::{
        Apply, Lambda,
        church::{LFalse, LPair, LSucc, LSuccGen, LTrue, LZero},
    },
};

// =========================================================================
// Fibonacci Calculation (Iterative Pair Approach)
// =========================================================================

/// Fib<N>: Computes the Nth Fibonacci number using Church numerals.
///
/// Algorithm:
/// - Start: (0, 1)
/// - Step: (a, b) -> (b, a+b)
/// - Result: Fst of final pair = F(n)
pub struct Fib<N>(std::marker::PhantomData<N>);

impl<N> Lambda for Fib<N>
where
    N: Apply<FibStep>,
    <N as Apply<FibStep>>::Output: Apply<LPair<LZero, LSucc<LZero>>>,
    <<N as Apply<FibStep>>::Output as Apply<LPair<LZero, LSucc<LZero>>>>::Output: Apply<LTrue>,
{
    type Output =
        <<<N as Apply<FibStep>>::Output as Apply<LPair<LZero, LSucc<LZero>>>>::Output as Apply<
            LTrue,
        >>::Output;
}

pub struct FibStep;

impl<P> Apply<P> for FibStep
where
    P: Apply<LTrue> + Apply<LFalse>,
    <P as Apply<LFalse>>::Output: Apply<LSuccGen>,
    <P as Apply<LTrue>>::Output: Apply<LSuccGen>,
    <<P as Apply<LTrue>>::Output as Apply<LSuccGen>>::Output: Apply<<P as Apply<LFalse>>::Output>,
{
    type Output = LPair<
        <P as Apply<LFalse>>::Output, // Snd p
        <<<P as Apply<LTrue>>::Output as Apply<typelude::lambda::church::LSuccGen>>::Output
            as Apply<<P as Apply<LFalse>>::Output>>::Output, // Add (Fst p) (Snd p)
    >;
}

#[test]
fn test_fib_small() {
    use static_assertions::assert_type_eq_all;

    // Numbers
    type One = LSucc<LZero>;
    type Two = LSucc<One>;
    type Three = LSucc<Two>;
    type Four = LSucc<Three>;
    type Five = LSucc<Four>;

    // Fib 0 = 0
    type F0 = Evaluate<Fib<LZero>>;
    assert_type_eq_all!(F0, LZero);

    // Fib 1 = 1
    type F1 = Evaluate<Fib<One>>;
    assert_type_eq_all!(F1, One);

    // For F(2) and beyond, we compare structurally using Apply
    type App<F, A> = <F as Apply<A>>::Output;

    struct TestF;
    struct TestX;
    struct TestF1<T>(std::marker::PhantomData<T>);
    impl<T> Apply<T> for TestF {
        type Output = TestF1<T>;
    }

    // Fib 2 = 1
    type F2 = Evaluate<Fib<Two>>;
    type ResF2 = App<App<F2, TestF>, TestX>;
    type ResOne = App<App<One, TestF>, TestX>;
    assert_type_eq_all!(ResF2, ResOne);

    // Fib 3 = 2
    type F3 = Evaluate<Fib<Three>>;
    type ResF3 = App<App<F3, TestF>, TestX>;
    type ResTwo = App<App<Two, TestF>, TestX>;
    assert_type_eq_all!(ResF3, ResTwo);

    // Fib 4 = 3
    type F4 = Evaluate<Fib<Four>>;
    type ResF4 = App<App<F4, TestF>, TestX>;
    type ResThree = App<App<Three, TestF>, TestX>;
    assert_type_eq_all!(ResF4, ResThree);

    // Fib 5 = 5
    type F5 = Evaluate<Fib<Five>>;
    type ResF5 = App<App<F5, TestF>, TestX>;
    type ResFive = App<App<Five, TestF>, TestX>;
    assert_type_eq_all!(ResF5, ResFive);
}
