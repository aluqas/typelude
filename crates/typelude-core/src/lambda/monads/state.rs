//! State Monad
//!
//! `State<F>` wraps a function `S -> (A, S)` for stateful computations.

use std::marker::PhantomData;

use crate::{
    eval::{Eval, Evaluate},
    lambda::{LApp, Lambda, church::LPair2, traits::LBind},
};

// Macro to implement Eval for terms (they evaluate to themselves)
macro_rules! impl_eval_term {
    ($($t:ty),*) => {
        $(
            impl Eval for $t {
                type Output = $t;
            }
        )*
    };
}

/// State Monad Wrapper: State<F>
/// F is a function S -> (A, S)
pub struct LState<F>(PhantomData<F>);

impl<F> Lambda for LState<F> {
    type Output = LState<F>;
}
impl<F> Eval for LState<F> { type Output = Self; }

// State<F> s -> F s
impl<F, S> Lambda for LApp<LState<F>, S>
where
    F: Eval,
    S: Eval,
    LApp<F, S>: Lambda,
{
    type Output = <LApp<F, S> as Lambda>::Output;
}

// -------------------------------------------------------------------------
// Bind: m >>= k
// \s. let (a, s') = m s in k a s'
// -------------------------------------------------------------------------

impl<F, K> LBind<K> for LState<F> {
    type Output = LState<LBindState<F, K>>;
}

pub struct LBindState<F, K>(PhantomData<(F, K)>);
impl<F, K> Lambda for LBindState<F, K> {
    type Output = LBindState<F, K>;
}
impl<F, K> Eval for LBindState<F, K> { type Output = Self; }

// BindState<F, K> S -> Result
impl<F, K, S> Lambda for LApp<LBindState<F, K>, S>
where
    F: Eval,
    K: Eval,
    S: Eval,
    // 1. Run m s -> pair
    LApp<F, S>: Lambda,
    // The output must be a Pair2<A, NewS>.
    // To get A and NewS, we can use LFst and LSnd if they work on Pair2.
    // Or we can assume the result is LPair2<A, NewS> and extract via Pattern Matching?
    // Rust traits don't support pattern matching on types easily in where clauses.
    // But we can use LFst/LSnd.
    // LFst<Pair2<A, S>> -> A.
    // We defined LFst/LSnd in pair.rs to work via LApp.

    // Let Pair = (m s)
    // Fst Pair
    LApp<crate::lambda::church::LFst, <LApp<F, S> as Lambda>::Output>: Lambda,
    // Snd Pair
    LApp<crate::lambda::church::LSnd, <LApp<F, S> as Lambda>::Output>: Lambda,
    // k a -> m'
    // Let A = Fst Pair
    LApp<K, <LApp<crate::lambda::church::LFst, <LApp<F, S> as Lambda>::Output> as Lambda>::Output>:
        Lambda,
    // m' s' -> Result
    // Let M' = k a
    // Let S' = Snd Pair
    LApp<
        <LApp<
            K,
            <LApp<crate::lambda::church::LFst, <LApp<F, S> as Lambda>::Output> as Lambda>::Output,
        > as Lambda>::Output,
        <LApp<crate::lambda::church::LSnd, <LApp<F, S> as Lambda>::Output> as Lambda>::Output,
    >: Lambda,
{
    type Output = <LApp<
        <LApp<
            K,
            <LApp<crate::lambda::church::LFst, <LApp<F, S> as Lambda>::Output> as Lambda>::Output,
        > as Lambda>::Output,
        <LApp<crate::lambda::church::LSnd, <LApp<F, S> as Lambda>::Output> as Lambda>::Output,
    > as Lambda>::Output;
}

// -------------------------------------------------------------------------
// Return / Pure: return x = \s. (x, s)
// -------------------------------------------------------------------------

pub struct LReturn<A>(PhantomData<A>);
impl<A> Lambda for LReturn<A> {
    type Output = LReturn<A>;
}
impl<A> Eval for LReturn<A> { type Output = Self; }

// Return<A> S -> (A, S)
impl<A, S> Lambda for LApp<LReturn<A>, S>
where
    A: Eval,
    S: Eval,
{
    // Return a Pair Value directly.
    // LApp<LApp<LPair, A>, S> -> LPair2<A, S>
    // We can just return LPair2<A, S> since A and S are Evaluated.
    // Wait, Evaluate<A> and Evaluate<S>.
    type Output = LPair2<Evaluate<A>, Evaluate<S>>;
}

// Return<A> >>= k  === k A
impl<A, K> LBind<K> for LReturn<A>
where
    K: Eval,
    A: Eval,
    LApp<K, A>: Lambda,
{
    type Output = <LApp<K, A> as Lambda>::Output;
}

// -------------------------------------------------------------------------
// Get: get = \s. (s, s)
// -------------------------------------------------------------------------

pub struct LGet;
impl Lambda for LGet {
    type Output = LGet;
}
impl_eval_term!(LGet);

// Get S -> (S, S)
impl<S> Lambda for LApp<LGet, S>
where
    S: Eval,
{
    type Output = LPair2<Evaluate<S>, Evaluate<S>>;
}

// Get >>= k
// \s. let (x, s) = (s, s) in k x s -> k s s
impl<K> LBind<K> for LGet {
    type Output = LState<LBindGet<K>>;
}

pub struct LBindGet<K>(PhantomData<K>);
impl<K> Lambda for LBindGet<K> {
    type Output = LBindGet<K>;
}
impl<K> Eval for LBindGet<K> { type Output = Self; }

impl<K, S> Lambda for LApp<LBindGet<K>, S>
where
    K: Eval,
    S: Eval + Clone, // Used twice
    // k s -> m'
    LApp<K, S>: Lambda,
    // m' s
    LApp<<LApp<K, S> as Lambda>::Output, S>: Lambda,
{
    type Output = <LApp<<LApp<K, S> as Lambda>::Output, S> as Lambda>::Output;
}

// -------------------------------------------------------------------------
// Put: put s = \_old. ((), s)
// -------------------------------------------------------------------------

pub struct LPut<NewS>(PhantomData<NewS>);
impl<NewS> Lambda for LPut<NewS> {
    type Output = LPut<NewS>;
}
impl<NewS> Eval for LPut<NewS> { type Output = Self; }

// Helper Unit type
pub struct Unit;
impl Lambda for Unit {
    type Output = Unit;
}
impl_eval_term!(Unit);

// Put<NewS> OldS -> ((), NewS)
impl<NewS, OldS> Lambda for LApp<LPut<NewS>, OldS>
where
    NewS: Eval,
    OldS: Eval,
{
    type Output = LPair2<Unit, Evaluate<NewS>>;
}

// Put<NewS> >>= k
// \old. let ((), NewS) = ((), NewS) in k () NewS
impl<NewS, K> LBind<K> for LPut<NewS> {
    type Output = LState<LBindPut<NewS, K>>;
}

pub struct LBindPut<NewS, K>(PhantomData<(NewS, K)>);
impl<NewS, K> Lambda for LBindPut<NewS, K> {
    type Output = LBindPut<NewS, K>;
}
impl<NewS, K> Eval for LBindPut<NewS, K> { type Output = Self; }

impl<NewS, K, OldS> Lambda for LApp<LBindPut<NewS, K>, OldS>
where
    NewS: Eval,
    K: Eval,
    OldS: Eval,
    // k () -> m'
    LApp<K, Unit>: Lambda,
    // m' NewS
    LApp<<LApp<K, Unit> as Lambda>::Output, NewS>: Lambda,
{
    type Output = <LApp<<LApp<K, Unit> as Lambda>::Output, NewS> as Lambda>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::lambda::church::{LSucc, LZero};

    // Helper alias
    type App<F, A> = Evaluate<LApp<F, A>>;

    #[test]
    fn test_state_monad() {
        // Scenario: Return(Zero) >>= PutSucc >>= DoGet

        // PutSucc: \x. Put (Succ x)
        struct PutSucc;
        impl Lambda for PutSucc {
            type Output = PutSucc;
        }
        impl Eval for PutSucc { type Output = Self; }

        impl<X> Lambda for LApp<PutSucc, X>
        where
            X: crate::lambda::traits::LNat + Eval,
        {
            type Output = LPut<LSucc<Evaluate<X>>>;
        }

        // DoGet: \x. Get
        struct DoGet;
        impl Lambda for DoGet {
            type Output = DoGet;
        }
        impl Eval for DoGet { type Output = Self; }

        impl<X> Lambda for LApp<DoGet, X>
        where
            X: Eval,
        {
            type Output = LGet;
        }

        type Step1 = LReturn<LZero>;
        type M1 = Step1;
        // M1 >>= PutSucc
        type M2 = <M1 as LBind<PutSucc>>::Output;
        // M2 >>= DoGet
        type M3 = <M2 as LBind<DoGet>>::Output;

        // Run M3 with LZero state
        type FinalResult = App<M3, LZero>;

        // Result is (Val, St).
        // Val = Fst FinalResult
        type Val = App<crate::lambda::church::LFst, FinalResult>;
        type St = App<crate::lambda::church::LSnd, FinalResult>;

        // Expected: PutSucc(Zero) -> Put(1). State becomes 1. Val is ().
        // Then DoGet -> Get. Returns (1, 1).
        // Wait.
        // Return(0) >>= k. -> k 0.
        // PutSucc 0 -> Put 1.
        // Put 1 >>= k'. -> \old. k' () 1.
        // DoGet () -> Get.
        // \old. Get 1. -> (1, 1).

        // So Val should be 1, St should be 1.

        assert_type_eq_all!(Val, LSucc<LZero>);
        assert_type_eq_all!(St, LSucc<LZero>);
    }
}
