//! **Comparison Operations**
//!
//! Implementation of comparison operations using `typenum`.

use typenum::{Bit, IsEqual, IsGreater, IsGreaterOrEqual, IsLess, IsLessOrEqual};

use crate::std::{ops::From, prim::bool::ToBoolOut, traits::Bool};

/// Explicit equality decision.
///
/// Implement this trait when two types have a decidable equality relation.
///
/// ```rust,compile_fail
/// use typelude_std::core::{ELit, Evaluate};
/// use typelude_std::std::ops::EEq;
///
/// struct A;
/// struct B;
///
/// // `EqDecide` is not implemented for these custom types.
/// fn main() {
///     let _: Evaluate<EEq<ELit<A>, ELit<B>>>;
/// }
/// ```
#[diagnostic::on_unimplemented(
    message = "Cannot decide equality for `{Self}` and `{Rhs}`",
    label = "equality decision not implemented",
    note = "implement `EqDecide<{Rhs}>` for `{Self}`"
)]
pub trait EqDecide<Rhs> {
    type Output: Bool;
}

impl<Lhs, Rhs> EqDecide<Rhs> for Lhs
where
    Lhs: IsEqual<Rhs>,
    <Lhs as IsEqual<Rhs>>::Output: Bit,
    bool: From<<Lhs as IsEqual<Rhs>>::Output>,
    <bool as From<<Lhs as IsEqual<Rhs>>::Output>>::Output: Bool,
{
    type Output = ToBoolOut<<Lhs as IsEqual<Rhs>>::Output>;
}

crate::typelude_macros::ty_fn! {
    /// Equality: A == B -> Bool
    pub struct FEq<Lhs>
    {
        type Output = FEqCaptured<Lhs>;
    }
}

crate::typelude_macros::ty_fn! {
    /// Equality: A == B -> Bool
    pub struct FEqCaptured<Lhs, Rhs>
    where [Lhs: EqDecide<Rhs>]
    {
        type Output = <Lhs as EqDecide<Rhs>>::Output;
    }
}

pub type EEq<Lhs, Rhs> = crate::core::ECall2<FEq, Lhs, Rhs>;

crate::typelude_macros::ty_fn! {
    /// Inequality: A != B -> Bool
    pub struct FNeq<Lhs>
    {
        type Output = FNeqCaptured<Lhs>;
    }
}

crate::typelude_macros::ty_fn! {
    /// Inequality: A != B -> Bool
    pub struct FNeqCaptured<Lhs, Rhs>
    where [
        Lhs: EqDecide<Rhs>,
        <Lhs as EqDecide<Rhs>>::Output: Bool
    ]
    {
        type Output = <<Lhs as EqDecide<Rhs>>::Output as Bool>::Not;
    }
}

pub type ENeq<Lhs, Rhs> = crate::core::ECall2<FNeq, Lhs, Rhs>;

crate::typelude_macros::ty_fn! {
    /// Less than: A < B
    pub struct FLt<Lhs>
    {
        type Output = FLtCaptured<Lhs>;
    }
}

crate::typelude_macros::ty_fn! {
    /// Less than: A < B
    pub struct FLtCaptured<Lhs, Rhs>
    where [
        Lhs: IsLess<Rhs>,
        <Lhs as IsLess<Rhs>>::Output: Bit,
        bool: From<<Lhs as IsLess<Rhs>>::Output>
    ]
    {
        type Output = ToBoolOut<<Lhs as IsLess<Rhs>>::Output>;
    }
}

pub type ELt<Lhs, Rhs> = crate::core::ECall2<FLt, Lhs, Rhs>;

crate::typelude_macros::ty_fn! {
    /// Less than or equal: A <= B
    pub struct FLe<Lhs>
    {
        type Output = FLeCaptured<Lhs>;
    }
}

crate::typelude_macros::ty_fn! {
    /// Less than or equal: A <= B
    pub struct FLeCaptured<Lhs, Rhs>
    where [
        Lhs: IsLessOrEqual<Rhs>,
        <Lhs as IsLessOrEqual<Rhs>>::Output: Bit,
        bool: From<<Lhs as IsLessOrEqual<Rhs>>::Output>
    ]
    {
        type Output = ToBoolOut<<Lhs as IsLessOrEqual<Rhs>>::Output>;
    }
}

pub type ELe<Lhs, Rhs> = crate::core::ECall2<FLe, Lhs, Rhs>;

crate::typelude_macros::ty_fn! {
    /// Greater than: A > B
    pub struct FGt<Lhs>
    {
        type Output = FGtCaptured<Lhs>;
    }
}

crate::typelude_macros::ty_fn! {
    /// Greater than: A > B
    pub struct FGtCaptured<Lhs, Rhs>
    where [
        Lhs: IsGreater<Rhs>,
        <Lhs as IsGreater<Rhs>>::Output: Bit,
        bool: From<<Lhs as IsGreater<Rhs>>::Output>
    ]
    {
        type Output = ToBoolOut<<Lhs as IsGreater<Rhs>>::Output>;
    }
}

pub type EGt<Lhs, Rhs> = crate::core::ECall2<FGt, Lhs, Rhs>;

crate::typelude_macros::ty_fn! {
    /// Greater than or equal: A >= B
    pub struct FGe<Lhs>
    {
        type Output = FGeCaptured<Lhs>;
    }
}

crate::typelude_macros::ty_fn! {
    /// Greater than or equal: A >= B
    pub struct FGeCaptured<Lhs, Rhs>
    where [
        Lhs: IsGreaterOrEqual<Rhs>,
        <Lhs as IsGreaterOrEqual<Rhs>>::Output: Bit,
        bool: From<<Lhs as IsGreaterOrEqual<Rhs>>::Output>
    ]
    {
        type Output = ToBoolOut<<Lhs as IsGreaterOrEqual<Rhs>>::Output>;
    }
}

pub type EGe<Lhs, Rhs> = crate::core::ECall2<FGe, Lhs, Rhs>;

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_std::core::{ELit, Evaluate};
    use typenum::{N1, P1, P2, U1, U2};

    use super::*;
    use crate::std::prim::bool::{False, True};

    #[test]
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
