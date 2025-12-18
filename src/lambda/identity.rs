use std::marker::PhantomData;

use crate::{
    kernel::traits::Apply,
    lambda::{Lambda, monad::Bind},
};

/// Identity Monad: Id<T>
///
/// `Id<T>` allows treating a plain value `T` as a Monad.
pub struct Id<T>(PhantomData<T>);
impl<T> Lambda for Id<T> {
    type Output = Id<T>;
}

// Bind: Id<T> >>= F  ->  F T
// F must be a function that takes T and returns Id<U>.
impl<T, F> Bind<F> for Id<T>
where
    F: Apply<T>,
{
    type Output = <F as Apply<T>>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::lambda::church::{Succ, Zero};

    struct AddOne;
    impl<X> Apply<X> for AddOne {
        type Output = Id<Succ<X>>;
    }

    #[test]
    fn test_identity_monad() {
        type IdInput = Id<Zero>;
        type IdResult = <IdInput as Bind<AddOne>>::Output;
        assert_type_eq_all!(IdResult, Id<Succ<Zero>>);
    }
}
