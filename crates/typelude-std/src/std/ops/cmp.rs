//! **Comparison Operations**
//!
//! Implementation of comparison operations using `typenum`.

use typelude_macros::ty_fn;
use typelude_std::core::Evaluate;
use typenum::{Bit, IsGreater, IsGreaterOrEqual, IsLess, IsLessOrEqual};

use crate::std::{
    ops::From,
    prim::bool::{False, ToBoolOut, True},
};

//
// IsEq Trait (Generic Equality)
//

/// Helper for Type Equality: Returns const bool
#[diagnostic::on_unimplemented(
    message = "Cannot compare `{Self}` with `{Other}` for equality",
    label = "equality check not implemented",
    note = "IsEq is currently only implemented for types on Nightly Rust"
)]
pub trait IsEq<Other> {
    const EQ: bool;
}

#[cfg(feature = "nightly")]
impl<T, U> IsEq<U> for T {
    default const EQ: bool = false;
}

impl<T> IsEq<T> for T {
    const EQ: bool = true;
}

/// Helper trait to convert const bool to Bool
pub trait ConstToBool<const B: bool> {
    type Output;
}

impl ConstToBool<true> for () {
    type Output = True;
}

impl ConstToBool<false> for () {
    type Output = False;
}

#[cfg(feature = "nightly")]
pub type EqResult<L, R> = <() as ConstToBool<{ <L as IsEq<R>>::EQ }>>::Output;
#[cfg(feature = "nightly")]
pub type NeqResult<L, R> = <() as ConstToBool<{ !<L as IsEq<R>>::EQ }>>::Output;

// Equality: A == B
#[cfg(feature = "nightly")]
ty_fn! {
    /// Equality: A == B -> Bool
    pub struct EEq<Lhs, Rhs>
    where
        Lhs, Rhs,
        ~Lhs: IsEq<~Rhs>,
        (): ConstToBool<{ <Evaluate<Lhs> as IsEq<Evaluate<Rhs>>>::EQ }>
    {
        type Output = EqResult<~Lhs, ~Rhs>;
    }
}

// Inequality: A != B
#[cfg(feature = "nightly")]
ty_fn! {
    /// Inequality: A != B -> Bool
    pub struct ENeq<Lhs, Rhs>
    where
        Lhs, Rhs,
        ~Lhs: IsEq<~Rhs>,
        (): ConstToBool<{ !<Evaluate<Lhs> as IsEq<Evaluate<Rhs>>>::EQ }>
    {
        type Output = NeqResult<~Lhs, ~Rhs>;
    }
}

// Less than: A < B
ty_fn! {
    /// Less than: A < B
    pub struct ELt<Lhs, Rhs>
    where
        Lhs, Rhs,
        ~Lhs: IsLess<~Rhs>,
        <~Lhs as IsLess<~Rhs>>::Output: Bit,
        bool: From<<~Lhs as IsLess<~Rhs>>::Output>
    {
        type Output = ToBoolOut<<~Lhs as IsLess<~Rhs>>::Output>;
    }
}

// Less than or equal: A <= B
ty_fn! {
    /// Less than or equal: A <= B
    pub struct ELe<Lhs, Rhs>
    where
        Lhs, Rhs,
        ~Lhs: IsLessOrEqual<~Rhs>,
        <~Lhs as IsLessOrEqual<~Rhs>>::Output: Bit,
        bool: From<<~Lhs as IsLessOrEqual<~Rhs>>::Output>
    {
        type Output = ToBoolOut<<~Lhs as IsLessOrEqual<~Rhs>>::Output>;
    }
}

// Greater than: A > B
ty_fn! {
    /// Greater than: A > B
    pub struct EGt<Lhs, Rhs>
    where
        Lhs, Rhs,
        ~Lhs: IsGreater<~Rhs>,
        <~Lhs as IsGreater<~Rhs>>::Output: Bit,
        bool: From<<~Lhs as IsGreater<~Rhs>>::Output>
    {
        type Output = ToBoolOut<<~Lhs as IsGreater<~Rhs>>::Output>;
    }
}

// Greater than or equal: A >= B
ty_fn! {
    /// Greater than or equal: A >= B
    pub struct EGe<Lhs, Rhs>
    where
        Lhs, Rhs,
        ~Lhs: IsGreaterOrEqual<~Rhs>,
        <~Lhs as IsGreaterOrEqual<~Rhs>>::Output: Bit,
        bool: From<<~Lhs as IsGreaterOrEqual<~Rhs>>::Output>
    {
        type Output = ToBoolOut<<~Lhs as IsGreaterOrEqual<~Rhs>>::Output>;
    }
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_std::core::ELit;
    use typenum::{N1, P1, P2, U1, U2};

    use super::*;

    #[test]
    #[cfg(feature = "nightly")]
    fn test_eq_neq() {
        assert_type_eq_all!(Evaluate<EEq<ELit<U1>, ELit<U1>>>, True);
        assert_type_eq_all!(Evaluate<EEq<ELit<U1>, ELit<U2>>>, False);

        assert_type_eq_all!(Evaluate<ENeq<ELit<U1>, ELit<U1>>>, False);
        assert_type_eq_all!(Evaluate<ENeq<ELit<U1>, ELit<U2>>>, True);
    }

    #[test]
    fn test_lt() {
        assert_type_eq_all!(Evaluate<ELt<ELit<U1>, ELit<U2>>>, True);
        assert_type_eq_all!(Evaluate<ELt<ELit<U2>, ELit<U1>>>, False);
        assert_type_eq_all!(Evaluate<ELt<ELit<N1>, ELit<P1>>>, True);
    }

    #[test]
    fn test_le() {
        assert_type_eq_all!(Evaluate<ELe<ELit<U1>, ELit<U1>>>, True);
        assert_type_eq_all!(Evaluate<ELe<ELit<U1>, ELit<U2>>>, True);
        assert_type_eq_all!(Evaluate<ELe<ELit<U2>, ELit<U1>>>, False);
    }

    #[test]
    fn test_gt() {
        assert_type_eq_all!(Evaluate<EGt<ELit<U2>, ELit<U1>>>, True);
        assert_type_eq_all!(Evaluate<EGt<ELit<U1>, ELit<U2>>>, False);
        assert_type_eq_all!(Evaluate<EGt<ELit<P2>, ELit<N1>>>, True);
    }

    #[test]
    fn test_ge() {
        assert_type_eq_all!(Evaluate<EGe<ELit<U2>, ELit<U2>>>, True);
        assert_type_eq_all!(Evaluate<EGe<ELit<U2>, ELit<U1>>>, True);
        assert_type_eq_all!(Evaluate<EGe<ELit<U1>, ELit<U2>>>, False);
    }
}
