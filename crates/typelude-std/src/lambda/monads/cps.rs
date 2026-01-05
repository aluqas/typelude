//! Continuation-Passing Style (CPS) Monad
//!
//! `Cont<R, A>` represents a computation that, when given a continuation `(A ->
//! R)`, produces a result of type `R`.

use std::marker::PhantomData;

use typelude_core::Eval;

use crate::lambda::{LApp, Lambda, traits::LBind};
/// Continuation Monad.
///
/// `Cont<R, A>` wraps a computation that takes a continuation and produces `R`.
/// Here, `F` is the internal function `(A -> R) -> R`.
pub struct LCont<F>(PhantomData<F>);

impl<F> Lambda for LCont<F> {
    type Output = LCont<F>;
}
impl<F> Eval for LCont<F> {
    type Output = Self;
}

/// Run a continuation with a given continuation function.
///
/// `RunCont<C, K>` applies continuation `K: A -> R` to `C: Cont<F>`.
pub struct LRunCont<C, K>(PhantomData<(C, K)>);

// RunCont<Cont<F>, K> -> F K
impl<F, K> Lambda for LRunCont<LCont<F>, K>
where
    F: Eval,
    K: Eval,
    LApp<F, K>: Lambda,
{
    type Output = <LApp<F, K> as Lambda>::Output;
}
/// Pure/Return for Cont: wraps a value in a continuation.
///
/// `ContPure<A>` represents `\k. k a`.
pub struct LContPure<A>(PhantomData<A>);

impl<A> Lambda for LContPure<A> {
    type Output = LContPure<A>;
}
impl<A> Eval for LContPure<A> {
    type Output = Self;
}

// ContPure<A> is Cont where F = PureF<A>
// PureF<A> K -> K A
pub struct LPureF<A>(PhantomData<A>);
impl<A> Lambda for LPureF<A> {
    type Output = LPureF<A>;
}
impl<A> Eval for LPureF<A> {
    type Output = Self;
}

impl<A, K> Lambda for LApp<LPureF<A>, K>
where
    A: Eval,
    K: Eval,
    LApp<K, A>: Lambda,
{
    type Output = <LApp<K, A> as Lambda>::Output;
}
/// Bind implementation for Cont.
///
/// `m >>= f` becomes `\k. runCont m (\a. runCont (f a) k)`
impl<F, G> LBind<G> for LCont<F> {
    type Output = LCont<LBindF<F, G>>;
}

/// Internal bind function: `\k. runCont m (\a. runCont (f a) k)`
pub struct LBindF<F, G>(PhantomData<(F, G)>);
impl<F, G> Lambda for LBindF<F, G> {
    type Output = LBindF<F, G>;
}
impl<F, G> Eval for LBindF<F, G> {
    type Output = Self;
}

impl<F, G, K> Lambda for LApp<LBindF<F, G>, K>
where
    F: Eval,
    G: Eval,
    K: Eval,
    // F (BindK G K)
    LApp<F, LBindK<G, K>>: Lambda,
{
    type Output = <LApp<F, LBindK<G, K>> as Lambda>::Output;
}

/// Inner continuation: `\a. runCont (f a) k`
pub struct LBindK<G, K>(PhantomData<(G, K)>);
impl<G, K> Lambda for LBindK<G, K> {
    type Output = LBindK<G, K>;
}
impl<G, K> Eval for LBindK<G, K> {
    type Output = Self;
}

impl<G, K, A> Lambda for LApp<LBindK<G, K>, A>
where
    G: Eval + Clone,
    K: Eval + Clone,
    A: Eval,
    // G A -> Cont<G'>
    LApp<G, A>: Lambda,
    // RunCont (G A) K
    // (G A) must evaluate to LCont<F'>
    // We assume (G A) returns something that can apply K.
    // LApp<<LApp<G, A> as Lambda>::Output, K>
    // But G A returns LCont<F'>. LCont doesn't implement LApp directly to run.
    // LApp<LCont<F'>, K> is not defined. LApp<F', K> is what we want.
    // We need to extract F' from LCont<F'>.
    <LApp<G, A> as Lambda>::Output: LContRunner<K>,
{
    type Output = <<LApp<G, A> as Lambda>::Output as LContRunner<K>>::Output;
}

/// Helper trait to run a Cont with a continuation via LApp
pub trait LContRunner<K> {
    type Output;
}

impl<F, K> LContRunner<K> for LCont<F>
where
    F: Eval,
    K: Eval,
    LApp<F, K>: Lambda,
{
    type Output = <LApp<F, K> as Lambda>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_core::Evaluate;

    use super::*; // Imported here for tests

    // Helper alias
    type App<F, A> = Evaluate<LApp<F, A>>;

    #[derive(Clone)]
    struct A;
    impl Lambda for A {
        type Output = A;
    }
    impl Eval for A {
        type Output = Self;
    }
    #[derive(Clone)]
    struct B;
    impl Lambda for B {
        type Output = B;
    }
    impl Eval for B {
        type Output = Self;
    }

    // Identity continuation: K A -> A
    #[derive(Clone)]
    struct IdK;
    impl Lambda for IdK {
        type Output = IdK;
    }
    impl Eval for IdK {
        type Output = Self;
    }

    impl<X> Lambda for LApp<IdK, X>
    where
        X: Eval,
    {
        type Output = Evaluate<X>;
    }

    #[test]
    fn test_cont_pure() {
        // ContPure<A> with IdK should give A
        // Run it: LApp<F, IdK>
        type Result = App<LPureF<A>, IdK>;
        assert_type_eq_all!(Result, A);
    }

    #[test]
    fn test_cont_bind() {
        // Create a simple transformation: A -> Cont<PureF<B>>
        #[derive(Clone)]
        struct Transform;
        impl Lambda for Transform {
            type Output = Transform;
        }
        impl Eval for Transform {
            type Output = Self;
        }

        impl<X> Lambda for LApp<Transform, X>
        where
            X: Eval,
        {
            type Output = LCont<LPureF<B>>;
        }

        // ContPure<A> >>= Transform should give Cont that produces B
        type Bound = <LCont<LPureF<A>> as LBind<Transform>>::Output;

        // Bound is LCont<LBindF<...>>.
        // Extract F
        type F = <Bound as ExtractF>::F;

        // Run F with IdK
        type Result = App<F, IdK>;
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
