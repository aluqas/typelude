//! State Monad
//!
//! `State<F>` wraps a function `S -> (A, S)` for stateful computations.

use std::marker::PhantomData;

use typelude_std::core::{Apply, Eval, Evaluate};

use crate::lambda::{LApp, church::LPair2, traits::LBind};

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

impl<F> Eval for LState<F> {
    type Output = LState<F>;
}

// State<F> s -> F s
impl<F, S> Eval for LApp<LState<F>, S>
where
    F: Eval,
    S: Eval,
    LApp<F, S>: Eval,
{
    type Output = Evaluate<LApp<F, S>>;
}
impl<F, K> LBind<K> for LState<F> {
    type Output = LState<LBindState<F, K>>;
}

pub struct LBindState<F, K>(PhantomData<(F, K)>);
impl<F, K> Eval for LBindState<F, K> {
    type Output = LBindState<F, K>;
}

// BindState<F, K> S -> Result
impl<F, K, S> Eval for LApp<LBindState<F, K>, S>
where
    F: Eval,
    K: Eval,
    S: Eval,
    // 1. Run m s -> pair
    LApp<F, S>: Eval,
    // The output must be a Pair2<A, NewS>.
    // To get A and NewS, we can use LFst and LSnd if they work on Pair2.
    // Or we can assume the result is LPair2<A, NewS> and extract via Pattern Matching?
    // Rust traits don't support pattern matching on types easily in where clauses.
    // But we can use LFst/LSnd.
    // LFst<Pair2<A, S>> -> A.
    // We defined LFst/LSnd in pair.rs to work via LApp.

    // Let Pair = (m s)
    // Fst Pair
    LApp<crate::lambda::church::LFst, Evaluate<LApp<F, S>>>: Eval,
    // Snd Pair
    LApp<crate::lambda::church::LSnd, Evaluate<LApp<F, S>>>: Eval,
    // k a -> m'
    // Let A = Fst Pair
    LApp<K, Evaluate<LApp<crate::lambda::church::LFst, Evaluate<LApp<F, S>>>>>: Eval,
    // m' s' -> Result
    // Let M' = k a
    // Let S' = Snd Pair
    LApp<
        Evaluate<LApp<K, Evaluate<LApp<crate::lambda::church::LFst, Evaluate<LApp<F, S>>>>>>,
        Evaluate<LApp<crate::lambda::church::LSnd, Evaluate<LApp<F, S>>>>,
    >: Eval,
{
    type Output = Evaluate<
        LApp<
            Evaluate<LApp<K, Evaluate<LApp<crate::lambda::church::LFst, Evaluate<LApp<F, S>>>>>>,
            Evaluate<LApp<crate::lambda::church::LSnd, Evaluate<LApp<F, S>>>>,
        >,
    >;
}

impl<F, K, S> Apply<S> for LBindState<F, K> {
    type Output = LApp<LBindState<F, K>, S>;
}
pub struct LReturn<A>(PhantomData<A>);
impl<A> Eval for LReturn<A> {
    type Output = LReturn<A>;
}

// Return<A> S -> (A, S)
impl<A, S> Eval for LApp<LReturn<A>, S>
where
    A: Eval,
    S: Eval,
{
    type Output = LPair2<Evaluate<A>, Evaluate<S>>;
}

impl<A, S> Apply<S> for LReturn<A>
where
    A: Eval,
    S: Eval,
{
    type Output = LPair2<Evaluate<A>, Evaluate<S>>;
}

// Return<A> >>= k  === k A
impl<A, K> LBind<K> for LReturn<A>
where
    K: Eval,
    A: Eval,
    LApp<K, A>: Eval,
{
    type Output = Evaluate<LApp<K, A>>;
}
pub struct LGet;
impl_eval_term!(LGet);

// Get S -> (S, S)
impl<S> Eval for LApp<LGet, S>
where
    S: Eval,
{
    type Output = LPair2<Evaluate<S>, Evaluate<S>>;
}

impl<S> Apply<S> for LGet
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
impl<K> Eval for LBindGet<K> {
    type Output = LBindGet<K>;
}

impl<K, S> Eval for LApp<LBindGet<K>, S>
where
    K: Eval,
    S: Eval + Clone, // Used twice
    // k s -> m'
    LApp<K, S>: Eval,
    // m' s
    LApp<Evaluate<LApp<K, S>>, S>: Eval,
{
    type Output = Evaluate<LApp<Evaluate<LApp<K, S>>, S>>;
}

impl<K, S> Apply<S> for LBindGet<K> {
    type Output = LApp<LBindGet<K>, S>;
}
pub struct LPut<NewS>(PhantomData<NewS>);
impl<NewS> Eval for LPut<NewS> {
    type Output = LPut<NewS>;
}

// Helper Unit type
pub struct Unit;
impl_eval_term!(Unit);

// Put<NewS> OldS -> ((), NewS)
impl<NewS, OldS> Eval for LApp<LPut<NewS>, OldS>
where
    NewS: Eval,
    OldS: Eval,
{
    type Output = LPair2<Unit, Evaluate<NewS>>;
}

impl<NewS, OldS> Apply<OldS> for LPut<NewS>
where
    NewS: Eval,
{
    type Output = LPair2<Unit, Evaluate<NewS>>;
}

// Put<NewS> >>= k
// \old. let ((), NewS) = ((), NewS) in k () NewS
impl<NewS, K> LBind<K> for LPut<NewS> {
    type Output = LState<LBindPut<NewS, K>>;
}

pub struct LBindPut<NewS, K>(PhantomData<(NewS, K)>);
impl<NewS, K> Eval for LBindPut<NewS, K> {
    type Output = LBindPut<NewS, K>;
}

impl<NewS, K, OldS> Eval for LApp<LBindPut<NewS, K>, OldS>
where
    NewS: Eval,
    K: Eval,
    OldS: Eval,
    // k () -> m'
    LApp<K, Unit>: Eval,
    // m' NewS
    LApp<Evaluate<LApp<K, Unit>>, NewS>: Eval,
{
    type Output = Evaluate<LApp<Evaluate<LApp<K, Unit>>, NewS>>;
}

impl<NewS, K, OldS> Apply<OldS> for LBindPut<NewS, K> {
    type Output = LApp<LBindPut<NewS, K>, OldS>;
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
        impl Eval for PutSucc {
            type Output = PutSucc;
        }

        impl<X> Eval for LApp<PutSucc, X>
        where
            X: crate::lambda::traits::LNat + Eval,
        {
            type Output = LPut<LSucc<Evaluate<X>>>;
        }

        // DoGet: \x. Get
        struct DoGet;
        impl Eval for DoGet {
            type Output = DoGet;
        }

        impl<X> Eval for LApp<DoGet, X>
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

        assert_type_eq_all!(Val, LSucc<LZero>);
        assert_type_eq_all!(St, LSucc<LZero>);
    }
}
