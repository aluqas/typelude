//! **Comparison Operations**
//!
//! Implementation of comparison operations using `typenum`.

use typelude_core::{Eval, Evaluate};
use typelude_macros::def_op;
use typenum::{Bit, IsGreater, IsGreaterOrEqual, IsLess, IsLessOrEqual};

use crate::std::{
    bool::{ToTyBoolOut, TyFalse, TyTrue},
    ops::TyFrom,
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

/// Helper trait to convert const bool to TyBool
pub trait ConstToTyBool<const B: bool> {
    type Output;
}

impl ConstToTyBool<true> for () {
    type Output = TyTrue;
}

impl ConstToTyBool<false> for () {
    type Output = TyFalse;
}

#[cfg(feature = "nightly")]
pub type EqResult<L, R> = <() as ConstToTyBool<{ <L as IsEq<R>>::EQ }>>::Output;
#[cfg(feature = "nightly")]
pub type NeqResult<L, R> = <() as ConstToTyBool<{ !<L as IsEq<R>>::EQ }>>::Output;

// =============================================================================
// Operators - Using procedural macro
// =============================================================================

// Equality: A == B
#[cfg(feature = "nightly")]
def_op! {
    /// Equality: A == B -> TyBool
    name: OpEq,
    args: (Lhs, Rhs),
    ast: EEq {
        where: [
            Evaluate<Lhs>: IsEq<Evaluate<Rhs>>,
            (): ConstToTyBool<{ <Evaluate<Lhs> as IsEq<Evaluate<Rhs>>>::EQ }>
        ],
        type Output = EqResult<Evaluate<Lhs>, Evaluate<Rhs>>
    }
}

// Inequality: A != B
#[cfg(feature = "nightly")]
def_op! {
    /// Inequality: A != B -> TyBool
    name: OpNeq,
    args: (Lhs, Rhs),
    ast: ENeq {
        where: [
            Evaluate<Lhs>: IsEq<Evaluate<Rhs>>,
            (): ConstToTyBool<{ !<Evaluate<Lhs> as IsEq<Evaluate<Rhs>>>::EQ }>
        ],
        type Output = NeqResult<Evaluate<Lhs>, Evaluate<Rhs>>
    }
}

// Less than: A < B
def_op! {
    /// Less than: A < B
    name: OpLt,
    args: (Lhs, Rhs),
    ast: ELt {
        where: [
            Evaluate<Lhs>: IsLess<Evaluate<Rhs>>,
            <Evaluate<Lhs> as IsLess<Evaluate<Rhs>>>::Output: Bit,
            bool: TyFrom<<Evaluate<Lhs> as IsLess<Evaluate<Rhs>>>::Output>
        ],
        type Output = ToTyBoolOut<<Evaluate<Lhs> as IsLess<Evaluate<Rhs>>>::Output>
    }
}

// Less than or equal: A <= B
def_op! {
    /// Less than or equal: A <= B
    name: OpLe,
    args: (Lhs, Rhs),
    ast: ELe {
        where: [
            Evaluate<Lhs>: IsLessOrEqual<Evaluate<Rhs>>,
            <Evaluate<Lhs> as IsLessOrEqual<Evaluate<Rhs>>>::Output: Bit,
            bool: TyFrom<<Evaluate<Lhs> as IsLessOrEqual<Evaluate<Rhs>>>::Output>
        ],
        type Output = ToTyBoolOut<<Evaluate<Lhs> as IsLessOrEqual<Evaluate<Rhs>>>::Output>
    }
}

// Greater than: A > B
def_op! {
    /// Greater than: A > B
    name: OpGt,
    args: (Lhs, Rhs),
    ast: EGt {
        where: [
            Evaluate<Lhs>: IsGreater<Evaluate<Rhs>>,
            <Evaluate<Lhs> as IsGreater<Evaluate<Rhs>>>::Output: Bit,
            bool: TyFrom<<Evaluate<Lhs> as IsGreater<Evaluate<Rhs>>>::Output>
        ],
        type Output = ToTyBoolOut<<Evaluate<Lhs> as IsGreater<Evaluate<Rhs>>>::Output>
    }
}

// Greater than or equal: A >= B
def_op! {
    /// Greater than or equal: A >= B
    name: OpGe,
    args: (Lhs, Rhs),
    ast: EGe {
        where: [
            Evaluate<Lhs>: IsGreaterOrEqual<Evaluate<Rhs>>,
            <Evaluate<Lhs> as IsGreaterOrEqual<Evaluate<Rhs>>>::Output: Bit,
            bool: TyFrom<<Evaluate<Lhs> as IsGreaterOrEqual<Evaluate<Rhs>>>::Output>
        ],
        type Output = ToTyBoolOut<<Evaluate<Lhs> as IsGreaterOrEqual<Evaluate<Rhs>>>::Output>
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_core::ELit;
    use typenum::{N1, P1, P2, U1, U2};

    use super::*;

    #[test]
    #[cfg(feature = "nightly")]
    fn test_eq_neq() {
        assert_type_eq_all!(Evaluate<EEq<ELit<U1>, ELit<U1>>>, TyTrue);
        assert_type_eq_all!(Evaluate<EEq<ELit<U1>, ELit<U2>>>, TyFalse);

        assert_type_eq_all!(Evaluate<ENeq<ELit<U1>, ELit<U1>>>, TyFalse);
        assert_type_eq_all!(Evaluate<ENeq<ELit<U1>, ELit<U2>>>, TyTrue);
    }

    #[test]
    fn test_lt() {
        assert_type_eq_all!(Evaluate<ELt<ELit<U1>, ELit<U2>>>, TyTrue);
        assert_type_eq_all!(Evaluate<ELt<ELit<U2>, ELit<U1>>>, TyFalse);
        assert_type_eq_all!(Evaluate<ELt<ELit<N1>, ELit<P1>>>, TyTrue);
    }

    #[test]
    fn test_le() {
        assert_type_eq_all!(Evaluate<ELe<ELit<U1>, ELit<U1>>>, TyTrue);
        assert_type_eq_all!(Evaluate<ELe<ELit<U1>, ELit<U2>>>, TyTrue);
        assert_type_eq_all!(Evaluate<ELe<ELit<U2>, ELit<U1>>>, TyFalse);
    }

    #[test]
    fn test_gt() {
        assert_type_eq_all!(Evaluate<EGt<ELit<U2>, ELit<U1>>>, TyTrue);
        assert_type_eq_all!(Evaluate<EGt<ELit<U1>, ELit<U2>>>, TyFalse);
        assert_type_eq_all!(Evaluate<EGt<ELit<P2>, ELit<N1>>>, TyTrue);
    }

    #[test]
    fn test_ge() {
        assert_type_eq_all!(Evaluate<EGe<ELit<U2>, ELit<U2>>>, TyTrue);
        assert_type_eq_all!(Evaluate<EGe<ELit<U2>, ELit<U1>>>, TyTrue);
        assert_type_eq_all!(Evaluate<EGe<ELit<U1>, ELit<U2>>>, TyFalse);
    }
}
