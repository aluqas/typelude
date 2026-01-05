//! Church Booleans
//!
//! Church encoding of boolean values:
//! - `True`: λt f. t
//! - `False`: λt f. f

use std::marker::PhantomData;

use typelude_core::{Eval, Evaluate};

use crate::{
    impl_eval_for_lambda, impl_eval_for_lambda_generic,
    lambda::{
        LApp, Lambda,
        traits::{LBool, LTerm},
    },
};

// =========================================================================
// Church Booleans
// =========================================================================

/// Church True: λt f. t
pub struct LTrue;
impl LTerm for LTrue {}
impl LBool for LTrue {}
impl Lambda for LTrue {
    type Output = LTrue;
}
impl_eval_for_lambda!(LTrue);

/// Church False: λt f. f
pub struct LFalse;
impl LTerm for LFalse {}
impl LBool for LFalse {}
impl Lambda for LFalse {
    type Output = LFalse;
}
impl_eval_for_lambda!(LFalse);

// Partial Application States
pub struct LTrue1<T>(PhantomData<T>);
impl<T> Lambda for LTrue1<T> {
    type Output = LTrue1<T>;
}
impl_eval_for_lambda_generic!(LTrue1, [T]);

pub struct LFalse1<T>(PhantomData<T>);
impl<T> Lambda for LFalse1<T> {
    type Output = LFalse1<T>;
}
impl_eval_for_lambda_generic!(LFalse1, [T]);

// --- True Implementation ---
// True T -> True1<T>
impl<T> Lambda for LApp<LTrue, T>
where
    T: Eval,
{
    type Output = LTrue1<Evaluate<T>>;
}

// True1<T> F -> T
impl<T, F> Lambda for LApp<LTrue1<T>, F>
where
    T: Eval,
{
    type Output = Evaluate<T>;
}

// --- False Implementation ---
// False T -> False1<T>
impl<T> Lambda for LApp<LFalse, T>
where
    T: Eval,
{
    type Output = LFalse1<Evaluate<T>>;
}

// False1<T> F -> F
impl<T, F> Lambda for LApp<LFalse1<T>, F>
where
    F: Eval,
{
    type Output = Evaluate<F>;
}

// =========================================================================
// If Expression
// =========================================================================

/// Pure If Alias: ((P T) E)
pub type LPureIf<P, T, E> = Evaluate<LApp<LApp<LApp<LIf, P>, T>, E>>;

/// LIf: P T E -> ((P T) E)
pub struct LIf;
impl Lambda for LIf {
    type Output = LIf;
}
impl_eval_for_lambda!(LIf);

// If P -> If1<P>
pub struct LIf1<P>(PhantomData<P>);
impl<P> Lambda for LIf1<P> {
    type Output = LIf1<P>;
}
impl_eval_for_lambda_generic!(LIf1, [P]);

impl<P> Lambda for LApp<LIf, P>
where
    P: Eval,
{
    type Output = LIf1<Evaluate<P>>;
}

// If1<P> T -> If2<P, T>
pub struct LIf2<P, T>(PhantomData<(P, T)>);
impl<P, T> Lambda for LIf2<P, T> {
    type Output = LIf2<P, T>;
}
impl_eval_for_lambda_generic!(LIf2, [P, T]);

impl<P, T> Lambda for LApp<LIf1<P>, T>
where
    T: Eval,
{
    type Output = LIf2<P, Evaluate<T>>;
}

// If2<P, T> E -> P T E
impl<P, T, E> Lambda for LApp<LIf2<P, T>, E>
where
    P: Eval,
    T: Eval,
    E: Eval,
    // (P T)
    LApp<P, T>: Eval,
    // ((P T) E)
    LApp<Evaluate<LApp<P, T>>, E>: Eval,
{
    type Output = Evaluate<LApp<Evaluate<LApp<P, T>>, E>>;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;

    // Helper alias
    type App<F, A> = Evaluate<LApp<F, A>>;

    #[test]
    fn test_church_bools_basic() {
        #[derive(Clone)]
        struct A;
        impl Lambda for A {
            type Output = A;
        }
        impl_eval_for_lambda!(A);

        #[derive(Clone)]
        struct B;
        impl Lambda for B {
            type Output = B;
        }
        impl_eval_for_lambda!(B);

        // True A B -> A
        type TrueRes = App<App<App<LIf, LTrue>, A>, B>;
        assert_type_eq_all!(TrueRes, A);

        // False A B -> B
        type FalseRes = App<App<App<LIf, LFalse>, A>, B>;
        assert_type_eq_all!(FalseRes, B);

        // Direct application without If
        assert_type_eq_all!(App<App<LTrue, A>, B>, A);
        assert_type_eq_all!(App<App<LFalse, A>, B>, B);
    }
}
