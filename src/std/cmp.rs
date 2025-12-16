//! **Comparison Operations**
//!
//! Implementation of comparison operations using `typenum`.

use typenum::{IsGreater, IsGreaterOrEqual, IsLess, IsLessOrEqual};

use crate::{
    eval::{EApply2, Evaluable, Evaluator, Sealed},
    std::{
        bool::{Assert, ToTyBoolOut},
        conv::TyFrom,
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

// Function Markers: Comparison Operations

/// Equality: A == B -> TyBool
pub struct FEq;
/// Inequality: A != B -> TyBool
pub struct FNeq;
/// Less than: A < B
pub struct FLt;
/// Less than or equal: A <= B
pub struct FLe;
/// Greater than: A > B
pub struct FGt;
/// Greater than or equal: A >= B
pub struct FGe;

impl Sealed for FEq {}
impl Sealed for FNeq {}
impl Sealed for FLt {}
impl Sealed for FLe {}
impl Sealed for FGt {}
impl Sealed for FGe {}

// Comparison Operations

// --- Implementations ---

// FEq: Lhs == Rhs (Generic via IsEq)
impl<Lhs, Rhs> Evaluable for EApply2<FEq, Lhs, Rhs>
where
    Lhs: Evaluable,
    Rhs: Evaluable,
    Evaluator<Lhs>: IsEq<Evaluator<Rhs>>,
    (): ReflectBool<{ <Evaluator<Lhs> as IsEq<Evaluator<Rhs>>>::EQ }>,
{
    type Output = Evaluator<Assert<{ <Evaluator<Lhs> as IsEq<Evaluator<Rhs>>>::EQ }>>;
}

// FNeq: Lhs != Rhs (Generic via IsEq)
impl<Lhs, Rhs> Evaluable for EApply2<FNeq, Lhs, Rhs>
where
    Lhs: Evaluable,
    Rhs: Evaluable,
    Evaluator<Lhs>: IsEq<Evaluator<Rhs>>,
    (): ReflectBool<{ !<Evaluator<Lhs> as IsEq<Evaluator<Rhs>>>::EQ }>,
{
    type Output = Evaluator<Assert<{ !<Evaluator<Lhs> as IsEq<Evaluator<Rhs>>>::EQ }>>;
}

// --- FLt: Lhs < Rhs ---
impl<Lhs, Rhs> Evaluable for EApply2<FLt, Lhs, Rhs>
where
    Lhs: Evaluable,
    Rhs: Evaluable,
    Evaluator<Lhs>: IsLess<Evaluator<Rhs>>,
    bool: TyFrom<<Evaluator<Lhs> as IsLess<Evaluator<Rhs>>>::Output>,
{
    type Output = ToTyBoolOut<<Evaluator<Lhs> as IsLess<Evaluator<Rhs>>>::Output>;
}

// --- FLe: Lhs <= Rhs ---
impl<Lhs, Rhs> Evaluable for EApply2<FLe, Lhs, Rhs>
where
    Lhs: Evaluable,
    Rhs: Evaluable,
    Evaluator<Lhs>: IsLessOrEqual<Evaluator<Rhs>>,
    bool: TyFrom<<Evaluator<Lhs> as IsLessOrEqual<Evaluator<Rhs>>>::Output>,
{
    type Output = ToTyBoolOut<<Evaluator<Lhs> as IsLessOrEqual<Evaluator<Rhs>>>::Output>;
}

// --- FGt: Lhs > Rhs ---
impl<Lhs, Rhs> Evaluable for EApply2<FGt, Lhs, Rhs>
where
    Lhs: Evaluable,
    Rhs: Evaluable,
    Evaluator<Lhs>: IsGreater<Evaluator<Rhs>>,
    bool: TyFrom<<Evaluator<Lhs> as IsGreater<Evaluator<Rhs>>>::Output>,
{
    type Output = ToTyBoolOut<<Evaluator<Lhs> as IsGreater<Evaluator<Rhs>>>::Output>;
}

// --- FGe: Lhs >= Rhs ---
impl<Lhs, Rhs> Evaluable for EApply2<FGe, Lhs, Rhs>
where
    Lhs: Evaluable,
    Rhs: Evaluable,
    Evaluator<Lhs>: IsGreaterOrEqual<Evaluator<Rhs>>,
    bool: TyFrom<<Evaluator<Lhs> as IsGreaterOrEqual<Evaluator<Rhs>>>::Output>,
{
    type Output = ToTyBoolOut<<Evaluator<Lhs> as IsGreaterOrEqual<Evaluator<Rhs>>>::Output>;
}

//
// Aliases
//

// Equality (2 args)
pub type EEq<Lhs, Rhs> = EApply2<FEq, Lhs, Rhs>;
pub type ENotEq<Lhs, Rhs> = EApply2<FNeq, Lhs, Rhs>;

// Comparison (2 args)
pub type ELt<Lhs, Rhs> = EApply2<FLt, Lhs, Rhs>;
pub type ELe<Lhs, Rhs> = EApply2<FLe, Lhs, Rhs>;
pub type EGt<Lhs, Rhs> = EApply2<FGt, Lhs, Rhs>;
pub type EGe<Lhs, Rhs> = EApply2<FGe, Lhs, Rhs>;

//
// Tests
//

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typenum::{N1, P1, P2, U1, U2};

    use super::*;
    use crate::{
        eval::ELit,
        std::bool::{TyFalse, TyTrue},
    };

    #[test]
    fn test_eq_neq() {
        assert_type_eq_all!(Evaluator<EEq<ELit<U1>, ELit<U1>>>, TyTrue);
        assert_type_eq_all!(Evaluator<EEq<ELit<U1>, ELit<U2>>>, TyFalse);

        assert_type_eq_all!(Evaluator<ENotEq<ELit<U1>, ELit<U1>>>, TyFalse);
        assert_type_eq_all!(Evaluator<ENotEq<ELit<U1>, ELit<U2>>>, TyTrue);
    }

    #[test]
    fn test_lt() {
        assert_type_eq_all!(Evaluator<ELt<ELit<U1>, ELit<U2>>>, TyTrue);
        assert_type_eq_all!(Evaluator<ELt<ELit<U2>, ELit<U1>>>, TyFalse);
        assert_type_eq_all!(Evaluator<ELt<ELit<N1>, ELit<P1>>>, TyTrue);
    }

    #[test]
    fn test_le() {
        assert_type_eq_all!(Evaluator<ELe<ELit<U1>, ELit<U1>>>, TyTrue);
        assert_type_eq_all!(Evaluator<ELe<ELit<U1>, ELit<U2>>>, TyTrue);
        assert_type_eq_all!(Evaluator<ELe<ELit<U2>, ELit<U1>>>, TyFalse);
    }

    #[test]
    fn test_gt() {
        assert_type_eq_all!(Evaluator<EGt<ELit<U2>, ELit<U1>>>, TyTrue);
        assert_type_eq_all!(Evaluator<EGt<ELit<U1>, ELit<U2>>>, TyFalse);
        assert_type_eq_all!(Evaluator<EGt<ELit<P2>, ELit<N1>>>, TyTrue);
    }

    #[test]
    fn test_ge() {
        assert_type_eq_all!(Evaluator<EGe<ELit<U2>, ELit<U2>>>, TyTrue);
        assert_type_eq_all!(Evaluator<EGe<ELit<U2>, ELit<U1>>>, TyTrue);
        assert_type_eq_all!(Evaluator<EGe<ELit<U1>, ELit<U2>>>, TyFalse);
    }
}
