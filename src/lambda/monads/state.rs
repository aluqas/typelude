//! State Monad
//!
//! `State<F>` wraps a function `S -> (A, S)` for stateful computations.

use std::marker::PhantomData;

use crate::{
    kernel::traits::Apply,
    lambda::{
        church::{LFalse, LPair, LTrue},
        traits::LBind,
    },
};

/// State Monad Wrapper: State<F>
/// F is a function S -> (A, S)
pub struct LState<F>(PhantomData<F>);

// State<F> s -> F s
impl<F, S> Apply<S> for LState<F>
where
    F: Apply<S>,
{
    type Output = <F as Apply<S>>::Output;
}

// -------------------------------------------------------------------------
// Bind: m >>= k
// \s. let (a, s') = m s in k a s'
// -------------------------------------------------------------------------

impl<F, K> LBind<K> for LState<F> {
    type Output = LState<LBindState<F, K>>;
}

pub struct LBindState<F, K>(PhantomData<(F, K)>);

// Apply BindState to S
impl<F, K, S> Apply<S> for LBindState<F, K>
where
    // 1. Run m s -> pair
    F: Apply<S>,
    // The output must be a Pair, meaning it applies True to get Fst, and False to get Snd.
    <F as Apply<S>>::Output: Apply<LTrue> + Apply<LFalse>,
    // 2. Extract a = Fst p
    // K a -> m'
    K: Apply<<<F as Apply<S>>::Output as Apply<LTrue>>::Output>,
    // 3. Extract s' = Snd p
    // m' s' -> Result
    <K as Apply<<<F as Apply<S>>::Output as Apply<LTrue>>::Output>>::Output:
        Apply<<<F as Apply<S>>::Output as Apply<LFalse>>::Output>,
{
    type Output =
        <<K as Apply<<<F as Apply<S>>::Output as Apply<LTrue>>::Output>>::Output as Apply<
            <<F as Apply<S>>::Output as Apply<LFalse>>::Output,
        >>::Output;
}

// -------------------------------------------------------------------------
// Return / Pure: return x = \s. (x, s)
// -------------------------------------------------------------------------

pub struct LReturn<A>(PhantomData<A>);

impl<A, S> Apply<S> for LReturn<A> {
    type Output = LPair<A, S>;
}

// Return<A> >>= k  === k A
impl<A, K> LBind<K> for LReturn<A>
where
    K: Apply<A>,
{
    type Output = <K as Apply<A>>::Output;
}

// -------------------------------------------------------------------------
// Get: get = \s. (s, s)
// -------------------------------------------------------------------------

pub struct LGet;

impl<S> Apply<S> for LGet {
    type Output = LPair<S, S>;
}

// Get >>= k
// \s. let (x, s) = (s, s) in k x s -> k s s
impl<K> LBind<K> for LGet {
    type Output = LState<LBindGet<K>>;
}

pub struct LBindGet<K>(PhantomData<K>);

impl<K, S> Apply<S> for LBindGet<K>
where
    K: Apply<S>,
    <K as Apply<S>>::Output: Apply<S>,
{
    type Output = <<K as Apply<S>>::Output as Apply<S>>::Output;
}

// -------------------------------------------------------------------------
// Put: put s = \_old. ((), s)
// -------------------------------------------------------------------------

pub struct LPut<NewS>(PhantomData<NewS>);

// Helper Unit type
pub struct Unit;

impl<NewS, OldS> Apply<OldS> for LPut<NewS> {
    type Output = LPair<Unit, NewS>;
}

// Put<NewS> >>= k
// \old. let ((), NewS) = ((), NewS) in k () NewS
impl<NewS, K> LBind<K> for LPut<NewS> {
    type Output = LState<LBindPut<NewS, K>>;
}

pub struct LBindPut<NewS, K>(PhantomData<(NewS, K)>);

impl<NewS, K, OldS> Apply<OldS> for LBindPut<NewS, K>
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
    use crate::lambda::church::{LSucc, LZero};

    #[test]
    fn test_state_monad() {
        // Scenario: Return(Zero) >>= PutSucc >>= DoGet
        struct PutSucc;
        impl<X> Apply<X> for PutSucc {
            type Output = LPut<LSucc<X>>;
        }

        struct DoGet;
        impl<X> Apply<X> for DoGet {
            type Output = LGet;
        }

        type Step1 = LReturn<LZero>;
        type M1 = Step1;
        type M2 = <M1 as LBind<PutSucc>>::Output;
        type M3 = <M2 as LBind<DoGet>>::Output;

        type FinalResult = <M3 as Apply<LZero>>::Output;
        type Val = <FinalResult as Apply<LTrue>>::Output;
        type St = <FinalResult as Apply<LFalse>>::Output;

        assert_type_eq_all!(Val, LSucc<LZero>);
        assert_type_eq_all!(St, LSucc<LZero>);
    }
}
