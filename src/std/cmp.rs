//! **Comparison Operations**
//!
//! Implementation of comparison operations using `typenum`.

use typenum::{IsGreater, IsGreaterOrEqual, IsLess, IsLessOrEqual};

use crate::{
    eval::{Eval, Evaluate, Sealed},
    kernel::traits::Apply,
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
// Expression Structs
// =============================================================================

use std::marker::PhantomData;

/// Equality: A == B
pub struct EEq<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for EEq<Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Evaluate<Lhs>: IsEq<Evaluate<Rhs>>,
    (): ReflectBool<{ <Evaluate<Lhs> as IsEq<Evaluate<Rhs>>>::EQ }>,
{
    type Output = Evaluate<Assert<{ <Evaluate<Lhs> as IsEq<Evaluate<Rhs>>>::EQ }>>;
}

/// Inequality: A != B
pub struct ENeq<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for ENeq<Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Evaluate<Lhs>: IsEq<Evaluate<Rhs>>,
    (): ReflectBool<{ !<Evaluate<Lhs> as IsEq<Evaluate<Rhs>>>::EQ }>,
{
    type Output = Evaluate<Assert<{ !<Evaluate<Lhs> as IsEq<Evaluate<Rhs>>>::EQ }>>;
}

/// Less than: A < B
pub struct ELt<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for ELt<Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Evaluate<Lhs>: IsLess<Evaluate<Rhs>>,
    bool: TyFrom<<Evaluate<Lhs> as IsLess<Evaluate<Rhs>>>::Output>,
{
    type Output = ToTyBoolOut<<Evaluate<Lhs> as IsLess<Evaluate<Rhs>>>::Output>;
}

/// Less than or equal: A <= B
pub struct ELe<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for ELe<Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Evaluate<Lhs>: IsLessOrEqual<Evaluate<Rhs>>,
    bool: TyFrom<<Evaluate<Lhs> as IsLessOrEqual<Evaluate<Rhs>>>::Output>,
{
    type Output = ToTyBoolOut<<Evaluate<Lhs> as IsLessOrEqual<Evaluate<Rhs>>>::Output>;
}

/// Greater than: A > B
pub struct EGt<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for EGt<Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Evaluate<Lhs>: IsGreater<Evaluate<Rhs>>,
    bool: TyFrom<<Evaluate<Lhs> as IsGreater<Evaluate<Rhs>>>::Output>,
{
    type Output = ToTyBoolOut<<Evaluate<Lhs> as IsGreater<Evaluate<Rhs>>>::Output>;
}

/// Greater than or equal: A >= B
pub struct EGe<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for EGe<Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Evaluate<Lhs>: IsGreaterOrEqual<Evaluate<Rhs>>,
    bool: TyFrom<<Evaluate<Lhs> as IsGreaterOrEqual<Evaluate<Rhs>>>::Output>,
{
    type Output = ToTyBoolOut<<Evaluate<Lhs> as IsGreaterOrEqual<Evaluate<Rhs>>>::Output>;
}

// =============================================================================
// Operator Symbols (OpCodes)
// =============================================================================

/// Equality: A == B -> TyBool
pub struct OpEq;
/// Inequality: A != B -> TyBool
pub struct OpNeq;
/// Less than: A < B
pub struct OpLt;
/// Less than or equal: A <= B
pub struct OpLe;
/// Greater than: A > B
pub struct OpGt;
/// Greater than or equal: A >= B
pub struct OpGe;

impl Sealed for OpEq {}
impl Sealed for OpNeq {}
impl Sealed for OpLt {}
impl Sealed for OpLe {}
impl Sealed for OpGt {}
impl Sealed for OpGe {}

impl<Lhs, Rhs> Apply<(Lhs, Rhs)> for OpEq {
    type Output = EEq<Lhs, Rhs>;
}
impl<Lhs, Rhs> Apply<(Lhs, Rhs)> for OpNeq {
    type Output = ENeq<Lhs, Rhs>;
}
impl<Lhs, Rhs> Apply<(Lhs, Rhs)> for OpLt {
    type Output = ELt<Lhs, Rhs>;
}
impl<Lhs, Rhs> Apply<(Lhs, Rhs)> for OpLe {
    type Output = ELe<Lhs, Rhs>;
}
impl<Lhs, Rhs> Apply<(Lhs, Rhs)> for OpGt {
    type Output = EGt<Lhs, Rhs>;
}
impl<Lhs, Rhs> Apply<(Lhs, Rhs)> for OpGe {
    type Output = EGe<Lhs, Rhs>;
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
