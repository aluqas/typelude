//! Identity Monad
//!
//! `Id<T>` allows treating a plain value `T` as a Monad.

use std::marker::PhantomData;

use typelude_std::core::{Eval, Evaluate};

use crate::lambda::{LApp, traits::LBind};

/// Identity Monad: Id<T>
///
/// `Id<T>` allows treating a plain value `T` as a Monad.
pub struct LId<T>(PhantomData<T>);

impl<T> Eval for LId<T> {
    type Output = LId<T>;
}

// Bind: Id<T> >>= F  ->  F T
// F must be a function that takes T and returns Id<U>.
impl<T, F> LBind<F> for LId<T>
where
    F: Eval,
    T: Eval,
    LApp<F, T>: Eval,
{
    type Output = Evaluate<LApp<F, T>>;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_std::core::Evaluate;

    use super::*;
    use crate::lambda::church::{LSucc, LZero};

    #[derive(Clone)]
    struct AddOne;
    impl Eval for AddOne {
        type Output = AddOne;
    }

    impl<X> Eval for LApp<AddOne, X>
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
