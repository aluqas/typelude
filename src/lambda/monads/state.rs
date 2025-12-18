//! State Monad
//!
//! `State<F>` wraps a function `S -> (A, S)` for stateful computations.

use std::marker::PhantomData;

use crate::{
    kernel::traits::Apply,
    lambda::{
        church::{False, Pair, True},
        traits::Bind,
    },
};

/// State Monad Wrapper: State<F>
/// F is a function S -> (A, S)
pub struct State<F>(PhantomData<F>);

// State<F> s -> F s
impl<F, S> Apply<S> for State<F>
where
    F: Apply<S>,
{
    type Output = <F as Apply<S>>::Output;
}

// -------------------------------------------------------------------------
// Bind: m >>= k
// \s. let (a, s') = m s in k a s'
// -------------------------------------------------------------------------

impl<F, K> Bind<K> for State<F> {
    type Output = State<BindState<F, K>>;
}

pub struct BindState<F, K>(PhantomData<(F, K)>);

// Apply BindState to S
impl<F, K, S> Apply<S> for BindState<F, K>
where
    // 1. Run m s -> pair
    F: Apply<S>,
    // The output must be a Pair, meaning it applies True to get Fst, and False to get Snd.
    <F as Apply<S>>::Output: Apply<True> + Apply<False>,
    // 2. Extract a = Fst p
    // K a -> m'
    K: Apply<<<F as Apply<S>>::Output as Apply<True>>::Output>,
    // 3. Extract s' = Snd p
    // m' s' -> Result
    <K as Apply<<<F as Apply<S>>::Output as Apply<True>>::Output>>::Output:
        Apply<<<F as Apply<S>>::Output as Apply<False>>::Output>,
{
    type Output =
        <<K as Apply<<<F as Apply<S>>::Output as Apply<True>>::Output>>::Output as Apply<
            <<F as Apply<S>>::Output as Apply<False>>::Output,
        >>::Output;
}

// -------------------------------------------------------------------------
// Return / Pure: return x = \s. (x, s)
// -------------------------------------------------------------------------

pub struct Return<A>(PhantomData<A>);

impl<A, S> Apply<S> for Return<A> {
    type Output = Pair<A, S>;
}

// Return<A> >>= k  === k A
impl<A, K> Bind<K> for Return<A>
where
    K: Apply<A>,
{
    type Output = <K as Apply<A>>::Output;
}

// -------------------------------------------------------------------------
// Get: get = \s. (s, s)
// -------------------------------------------------------------------------

pub struct Get;

impl<S> Apply<S> for Get {
    type Output = Pair<S, S>;
}

// Get >>= k
// \s. let (x, s) = (s, s) in k x s -> k s s
impl<K> Bind<K> for Get {
    type Output = State<BindGet<K>>;
}

pub struct BindGet<K>(PhantomData<K>);

impl<K, S> Apply<S> for BindGet<K>
where
    K: Apply<S>,
    <K as Apply<S>>::Output: Apply<S>,
{
    type Output = <<K as Apply<S>>::Output as Apply<S>>::Output;
}

// -------------------------------------------------------------------------
// Put: put s = \_old. ((), s)
// -------------------------------------------------------------------------

pub struct Put<NewS>(PhantomData<NewS>);

// Helper Unit type
pub struct Unit;

impl<NewS, OldS> Apply<OldS> for Put<NewS> {
    type Output = Pair<Unit, NewS>;
}

// Put<NewS> >>= k
// \old. let ((), NewS) = ((), NewS) in k () NewS
impl<NewS, K> Bind<K> for Put<NewS> {
    type Output = State<BindPut<NewS, K>>;
}

pub struct BindPut<NewS, K>(PhantomData<(NewS, K)>);

impl<NewS, K, OldS> Apply<OldS> for BindPut<NewS, K>
where
    K: Apply<Unit>,
    <K as Apply<Unit>>::Output: Apply<NewS>,
{
    type Output = <<K as Apply<Unit>>::Output as Apply<NewS>>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::lambda::church::{Succ, Zero};

    #[test]
    fn test_state_monad() {
        // Scenario: Return(Zero) >>= PutSucc >>= DoGet
        struct PutSucc;
        impl<X> Apply<X> for PutSucc {
            type Output = Put<Succ<X>>;
        }

        struct DoGet;
        impl<X> Apply<X> for DoGet {
            type Output = Get;
        }

        type Step1 = Return<Zero>;
        type M1 = Step1;
        type M2 = <M1 as Bind<PutSucc>>::Output;
        type M3 = <M2 as Bind<DoGet>>::Output;

        type FinalResult = <M3 as Apply<Zero>>::Output;
        type Val = <FinalResult as Apply<True>>::Output;
        type St = <FinalResult as Apply<False>>::Output;

        assert_type_eq_all!(Val, Succ<Zero>);
        assert_type_eq_all!(St, Succ<Zero>);
    }
}
