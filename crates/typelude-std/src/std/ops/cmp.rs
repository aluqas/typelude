//! **Comparison Operations**
//!
//! Implementation of comparison operations using `typenum`.

use typenum::{Bit, IsEqual, IsGreater, IsGreaterOrEqual, IsLess, IsLessOrEqual};

use crate::std::{
    ops::From,
    prim::bool::ToBoolOut,
    traits::Bool,
};

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

// Equality: A == B
crate::def_expr_via_trait!(
    /// Equality: A == B -> Bool
    pub EEq<Lhs, Rhs>
    args [Lhs, Rhs]
    where [~Lhs: EqDecide<~Rhs>]
    => <~Lhs as EqDecide<~Rhs>>::Output
);

// Inequality: A != B
crate::def_expr_via_trait!(
    /// Inequality: A != B -> Bool
    pub ENeq<Lhs, Rhs>
    args [Lhs, Rhs]
    where [
        ~Lhs: EqDecide<~Rhs>,
        <~Lhs as EqDecide<~Rhs>>::Output: Bool
    ]
    => <<~Lhs as EqDecide<~Rhs>>::Output as Bool>::Not
);

// Less than: A < B
crate::def_expr_via_trait!(
    /// Less than: A < B
    pub ELt<Lhs, Rhs>
    args [Lhs, Rhs]
    where [
        ~Lhs: IsLess<~Rhs>,
        <~Lhs as IsLess<~Rhs>>::Output: Bit,
        bool: From<<~Lhs as IsLess<~Rhs>>::Output>
    ]
    => ToBoolOut<<~Lhs as IsLess<~Rhs>>::Output>
);

// Less than or equal: A <= B
crate::def_expr_via_trait!(
    /// Less than or equal: A <= B
    pub ELe<Lhs, Rhs>
    args [Lhs, Rhs]
    where [
        ~Lhs: IsLessOrEqual<~Rhs>,
        <~Lhs as IsLessOrEqual<~Rhs>>::Output: Bit,
        bool: From<<~Lhs as IsLessOrEqual<~Rhs>>::Output>
    ]
    => ToBoolOut<<~Lhs as IsLessOrEqual<~Rhs>>::Output>
);

// Greater than: A > B
crate::def_expr_via_trait!(
    /// Greater than: A > B
    pub EGt<Lhs, Rhs>
    args [Lhs, Rhs]
    where [
        ~Lhs: IsGreater<~Rhs>,
        <~Lhs as IsGreater<~Rhs>>::Output: Bit,
        bool: From<<~Lhs as IsGreater<~Rhs>>::Output>
    ]
    => ToBoolOut<<~Lhs as IsGreater<~Rhs>>::Output>
);

// Greater than or equal: A >= B
crate::def_expr_via_trait!(
    /// Greater than or equal: A >= B
    pub EGe<Lhs, Rhs>
    args [Lhs, Rhs]
    where [
        ~Lhs: IsGreaterOrEqual<~Rhs>,
        <~Lhs as IsGreaterOrEqual<~Rhs>>::Output: Bit,
        bool: From<<~Lhs as IsGreaterOrEqual<~Rhs>>::Output>
    ]
    => ToBoolOut<<~Lhs as IsGreaterOrEqual<~Rhs>>::Output>
);

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
