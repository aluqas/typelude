//! Identity Monad
//!
//! `Id<T>` allows treating a plain value `T` as a Monad.

use std::marker::PhantomData;

use crate::{
    eval::{Eval, Evaluate},
    lambda::{LApp, Lambda, traits::LBind},
};

/// Identity Monad: Id<T>
///
/// `Id<T>` allows treating a plain value `T` as a Monad.
pub struct LId<T>(PhantomData<T>);

impl<T> Lambda for LId<T> {
    type Output = LId<T>;
}

// Bind: Id<T> >>= F  ->  F T
// F must be a function that takes T and returns Id<U>.
impl<T, F> LBind<F> for LId<T>
where
    F: Eval,
    T: Eval,
    LApp<F, T>: Lambda,
{
    type Output = <LApp<F, T> as Lambda>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::lambda::church::{LSucc, LZero};

    #[derive(Clone)]
    struct AddOne;
    impl Lambda for AddOne { type Output = AddOne; }

    impl<X> Lambda for LApp<AddOne, X>
    where
        X: crate::lambda::traits::LNat + Eval,
    {
        type Output = LId<LSucc<Evaluate<X>>>;
    }

    #[test]
    fn test_identity_monad() {
        type IdInput = LId<LZero>;
        type IdResult = <IdInput as LBind<AddOne>>::Output;
        assert_type_eq_all!(IdResult, LId<LSucc<LZero>>);
    }
}
