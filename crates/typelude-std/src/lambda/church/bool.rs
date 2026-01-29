//! Church Booleans
//!
//! Church encoding of boolean values:
//! - `True`: λt f. t
//! - `False`: λt f. f

use std::marker::PhantomData;

use typelude_std::core::{Eval, Evaluate};

use crate::{
    impl_eval_for_lambda, impl_eval_for_lambda_generic,
    lambda::{
        LApp,
        traits::{LBool, LTerm},
    },
};
/// Church True: λt f. t
pub struct LTrue;
impl LTerm for LTrue {}
impl LBool for LTrue {}
impl_eval_for_lambda!(LTrue);

/// Church False: λt f. f
pub struct LFalse;
impl LTerm for LFalse {}
impl LBool for LFalse {}
impl_eval_for_lambda!(LFalse);

// Partial Application States
pub struct LTrue1<T>(PhantomData<T>);
impl_eval_for_lambda_generic!(LTrue1, [T]);

pub struct LFalse1<T>(PhantomData<T>);
impl_eval_for_lambda_generic!(LFalse1, [T]);

// --- True Implementation ---
// True T -> True1<T>
impl<T> Eval for LApp<LTrue, T>
where
    T: Eval,
{
    type Output = LTrue1<Evaluate<T>>;
}

// True1<T> F -> T
impl<T, F> Eval for LApp<LTrue1<T>, F>
where
    T: Eval,
{
    type Output = Evaluate<T>;
}

// --- False Implementation ---
// False T -> False1<T>
impl<T> Eval for LApp<LFalse, T>
where
    T: Eval,
{
    type Output = LFalse1<Evaluate<T>>;
}

// False1<T> F -> F
impl<T, F> Eval for LApp<LFalse1<T>, F>
where
    F: Eval,
{
    type Output = Evaluate<F>;
}
/// Pure If Alias: ((P T) E)
pub type LPureIf<P, T, E> = Evaluate<LApp<LApp<LApp<LIf, P>, T>, E>>;

/// LIf: P T E -> ((P T) E)
pub struct LIf;
impl_eval_for_lambda!(LIf);

// If P -> If1<P>
pub struct LIf1<P>(PhantomData<P>);
impl_eval_for_lambda_generic!(LIf1, [P]);

impl<P> Eval for LApp<LIf, P>
where
    P: Eval,
{
    type Output = LIf1<Evaluate<P>>;
}

// If1<P> T -> If2<P, T>
pub struct LIf2<P, T>(PhantomData<(P, T)>);
impl_eval_for_lambda_generic!(LIf2, [P, T]);

impl<P, T> Eval for LApp<LIf1<P>, T>
where
    T: Eval,
{
    type Output = LIf2<P, Evaluate<T>>;
}

// If2<P, T> E -> P T E
impl<P, T, E> Eval for LApp<LIf2<P, T>, E>
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
        impl_eval_for_lambda!(A);

        #[derive(Clone)]
        struct B;
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
