//! Continuation-Passing Style (CPS) Monad
//!
//! `Cont<R, A>` represents a computation that, when given a continuation `(A -> R)`,
//! produces a result of type `R`.

use std::marker::PhantomData;

use crate::{
    kernel::traits::Apply,
    lambda::{Lambda, traits::LBind},
};

// =========================================================================
// Cont Monad: Cont<R, A> ~ (A -> R) -> R
// =========================================================================

/// Continuation Monad.
///
/// `Cont<R, A>` wraps a computation that takes a continuation and produces `R`.
/// Here, `F` is the internal function `(A -> R) -> R`.
pub struct LCont<F>(PhantomData<F>);

impl<F> Lambda for LCont<F> {
    type Output = LCont<F>;
}

/// Run a continuation with a given continuation function.
///
/// `RunCont<C, K>` applies continuation `K: A -> R` to `C: Cont<F>`.
pub struct LRunCont<C, K>(PhantomData<(C, K)>);

impl<F, K> Lambda for LRunCont<LCont<F>, K>
where
    F: Apply<K>,
{
    type Output = <F as Apply<K>>::Output;
}

// =========================================================================
// Pure / Return: a -> Cont<R, A>
// =========================================================================

/// Pure/Return for Cont: wraps a value in a continuation.
///
/// `ContPure<A>` represents `\k. k a`.
pub struct LContPure<A>(PhantomData<A>);

impl<A> Lambda for LContPure<A> {
    type Output = LContPure<A>;
}

// ContPure<A> is Cont where F = PureF<A>
// PureF<A> K -> K A
pub struct LPureF<A>(PhantomData<A>);

impl<A, K> Apply<K> for LPureF<A>
where
    K: Apply<A>,
{
    type Output = <K as Apply<A>>::Output;
}

// =========================================================================
// Bind: Cont<R, A> >>= (A -> Cont<R, B>) -> Cont<R, B>
// =========================================================================

/// Bind implementation for Cont.
///
/// `m >>= f` becomes `\k. runCont m (\a. runCont (f a) k)`
impl<F, G> LBind<G> for LCont<F> {
    type Output = LCont<LBindF<F, G>>;
}

/// Internal bind function: `\k. runCont m (\a. runCont (f a) k)`
pub struct LBindF<F, G>(PhantomData<(F, G)>);

impl<F, G, K> Apply<K> for LBindF<F, G>
where
    // F is the inner function of Cont<F>
    // We need to apply F to a continuation that:
    // 1. Takes A
    // 2. Applies G to A to get Cont<G'>
    // 3. Runs Cont<G'> with K
    F: Apply<LBindK<G, K>>,
{
    type Output = <F as Apply<LBindK<G, K>>>::Output;
}

/// Inner continuation: `\a. runCont (f a) k`
pub struct LBindK<G, K>(PhantomData<(G, K)>);

impl<G, K, A> Apply<A> for LBindK<G, K>
where
    G: Apply<A>,                            // f a -> Cont<G'>
    <G as Apply<A>>::Output: LContRunner<K>, // runCont (f a) k
{
    type Output = <<G as Apply<A>>::Output as LContRunner<K>>::Output;
}

/// Helper trait to run a Cont with a continuation.
pub trait LContRunner<K> {
    type Output;
}

impl<F, K> LContRunner<K> for LCont<F>
where
    F: Apply<K>,
{
    type Output = <F as Apply<K>>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;

    struct A;
    struct B;
    struct R;

    // Identity continuation: K A -> A
    struct IdK;
    impl<X> Apply<X> for IdK {
        type Output = X;
    }

    #[test]
    fn test_cont_pure() {
        // ContPure<A> with IdK should give A
        type Pure = LCont<LPureF<A>>;
        type Result = <LPureF<A> as Apply<IdK>>::Output;
        assert_type_eq_all!(Result, A);
    }

    #[test]
    fn test_cont_bind() {
        // Create a simple transformation: A -> Cont<PureF<B>>
        struct Transform;
        impl Apply<A> for Transform {
            type Output = LCont<LPureF<B>>;
        }

        // ContPure<A> >>= Transform should give Cont that produces B
        type Bound = <LCont<LPureF<A>> as LBind<Transform>>::Output;

        // Run with IdK
        type Result = <<Bound as ExtractF>::F as Apply<IdK>>::Output;
        assert_type_eq_all!(Result, B);
    }

    // Helper to extract F from Cont<F>
    trait ExtractF {
        type F;
    }
    impl<F> ExtractF for LCont<F> {
        type F = F;
    }
}
