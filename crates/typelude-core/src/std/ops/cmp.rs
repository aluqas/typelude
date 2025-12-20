//! **Comparison Operations**
//!
//! Implementation of comparison operations using `typenum`.

use typenum::{IsGreater, IsGreaterOrEqual, IsLess, IsLessOrEqual};

use crate::{
    def_op,
    op_bound, op_call,
    eval::{Eval, Evaluate},
    std::{
        bool::{Assert, ToTyBoolOut},
        into::TyFrom,
        reify::ReflectBool,
    },
};

//
// IsEq Trait (Generic Equality)
//

/// Helper for Type Equality: Returns const bool
pub trait IsEq<Other> {
    const EQ: bool;
}

impl<T, U> IsEq<U> for T {
    default const EQ: bool = false;
}

impl<T> IsEq<T> for T {
    const EQ: bool = true;
}

// =============================================================================
// Operators
// =============================================================================

// Equality: A == B
def_op! {
    /// Equality: A == B -> TyBool
    name: OpEq,
    args: (Lhs, Rhs),
    ast: EEq {
        where: [
            Lhs: Eval,
            Rhs: Eval,
            op_bound!(Lhs.IsEq(Rhs)),
            (): ReflectBool<{ <Evaluate<Lhs> as IsEq<Evaluate<Rhs>>>::EQ }>
        ],
        type Output = Evaluate<Assert<{ <Evaluate<Lhs> as IsEq<Evaluate<Rhs>>>::EQ }>>
    }
}

// Inequality: A != B
def_op! {
    /// Inequality: A != B -> TyBool
    name: OpNeq,
    args: (Lhs, Rhs),
    ast: ENeq {
        where: [
            Lhs: Eval,
            Rhs: Eval,
            op_bound!(Lhs.IsEq(Rhs)),
            (): ReflectBool<{ !<Evaluate<Lhs> as IsEq<Evaluate<Rhs>>>::EQ }>
        ],
        type Output = Evaluate<Assert<{ !<Evaluate<Lhs> as IsEq<Evaluate<Rhs>>>::EQ }>>
    }
}

// Less than: A < B
def_op! {
    /// Less than: A < B
    name: OpLt,
    args: (Lhs, Rhs),
    ast: ELt {
        where: [
            Lhs: Eval,
            Rhs: Eval,
            op_bound!(Lhs.IsLess(Rhs)),
            bool: TyFrom<<Evaluate<Lhs> as IsLess<Evaluate<Rhs>>>::Output>
        ],
        type Output = ToTyBoolOut<op_call!(Lhs.IsLess(Rhs))>
    }
}

// Less than or equal: A <= B
def_op! {
    /// Less than or equal: A <= B
    name: OpLe,
    args: (Lhs, Rhs),
    ast: ELe {
        where: [
            Lhs: Eval,
            Rhs: Eval,
            op_bound!(Lhs.IsLessOrEqual(Rhs)),
            bool: TyFrom<<Evaluate<Lhs> as IsLessOrEqual<Evaluate<Rhs>>>::Output>
        ],
        type Output = ToTyBoolOut<op_call!(Lhs.IsLessOrEqual(Rhs))>
    }
}

// Greater than: A > B
def_op! {
    /// Greater than: A > B
    name: OpGt,
    args: (Lhs, Rhs),
    ast: EGt {
        where: [
            Lhs: Eval,
            Rhs: Eval,
            op_bound!(Lhs.IsGreater(Rhs)),
            bool: TyFrom<<Evaluate<Lhs> as IsGreater<Evaluate<Rhs>>>::Output>
        ],
        type Output = ToTyBoolOut<op_call!(Lhs.IsGreater(Rhs))>
    }
}

// Greater than or equal: A >= B
def_op! {
    /// Greater than or equal: A >= B
    name: OpGe,
    args: (Lhs, Rhs),
    ast: EGe {
        where: [
            Lhs: Eval,
            Rhs: Eval,
            op_bound!(Lhs.IsGreaterOrEqual(Rhs)),
            bool: TyFrom<<Evaluate<Lhs> as IsGreaterOrEqual<Evaluate<Rhs>>>::Output>
        ],
        type Output = ToTyBoolOut<op_call!(Lhs.IsGreaterOrEqual(Rhs))>
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typenum::{N1, P1, P2, U1, U2};

    use super::*;
    use crate::{
        eval::ELit,
        std::bool::{TyFalse, TyTrue},
        kernel::traits::Apply, // Import Apply explicitly
    };

    #[test]
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
