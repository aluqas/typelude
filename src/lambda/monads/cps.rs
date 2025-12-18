//! Continuation-Passing Style (CPS) Monad
//!
//! `Cont<R, A>` represents a computation that, when given a continuation `(A -> R)`,
//! produces a result of type `R`.

use std::marker::PhantomData;

use crate::{
    kernel::traits::Apply,
    lambda::{Lambda, traits::Bind},
};

// =========================================================================
// Cont Monad: Cont<R, A> ~ (A -> R) -> R
// =========================================================================

/// Continuation Monad.
///
/// `Cont<R, A>` wraps a computation that takes a continuation and produces `R`.
/// Here, `F` is the internal function `(A -> R) -> R`.
pub struct Cont<F>(PhantomData<F>);

impl<F> Lambda for Cont<F> {
    type Output = Cont<F>;
}

/// Run a continuation with a given continuation function.
///
/// `RunCont<C, K>` applies continuation `K: A -> R` to `C: Cont<F>`.
pub struct RunCont<C, K>(PhantomData<(C, K)>);

impl<F, K> Lambda for RunCont<Cont<F>, K>
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
pub struct ContPure<A>(PhantomData<A>);

impl<A> Lambda for ContPure<A> {
    type Output = ContPure<A>;
}

// ContPure<A> is Cont where F = PureF<A>
// PureF<A> K -> K A
pub struct PureF<A>(PhantomData<A>);

impl<A, K> Apply<K> for PureF<A>
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
impl<F, G> Bind<G> for Cont<F> {
    type Output = Cont<BindF<F, G>>;
}

/// Internal bind function: `\k. runCont m (\a. runCont (f a) k)`
pub struct BindF<F, G>(PhantomData<(F, G)>);

impl<F, G, K> Apply<K> for BindF<F, G>
where
    // F is the inner function of Cont<F>
    // We need to apply F to a continuation that:
    // 1. Takes A
    // 2. Applies G to A to get Cont<G'>
    // 3. Runs Cont<G'> with K
    F: Apply<BindK<G, K>>,
{
    type Output = <F as Apply<BindK<G, K>>>::Output;
}

/// Inner continuation: `\a. runCont (f a) k`
pub struct BindK<G, K>(PhantomData<(G, K)>);

impl<G, K, A> Apply<A> for BindK<G, K>
where
    G: Apply<A>,                            // f a -> Cont<G'>
    <G as Apply<A>>::Output: ContRunner<K>, // runCont (f a) k
{
    type Output = <<G as Apply<A>>::Output as ContRunner<K>>::Output;
}

/// Helper trait to run a Cont with a continuation.
pub trait ContRunner<K> {
    type Output;
}

impl<F, K> ContRunner<K> for Cont<F>
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
        type Pure = Cont<PureF<A>>;
        type Result = <PureF<A> as Apply<IdK>>::Output;
        assert_type_eq_all!(Result, A);
    }

    #[test]
    fn test_cont_bind() {
        // Create a simple transformation: A -> Cont<PureF<B>>
        struct Transform;
        impl Apply<A> for Transform {
            type Output = Cont<PureF<B>>;
        }

        // ContPure<A> >>= Transform should give Cont that produces B
        type Bound = <Cont<PureF<A>> as Bind<Transform>>::Output;

        // Run with IdK
        type Result = <<Bound as ExtractF>::F as Apply<IdK>>::Output;
        assert_type_eq_all!(Result, B);
    }

    // Helper to extract F from Cont<F>
    trait ExtractF {
        type F;
    }
    impl<F> ExtractF for Cont<F> {
        type F = F;
    }
}
