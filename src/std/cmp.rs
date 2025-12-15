//! **Comparison Operations**
//!
//! Implementation of comparison operations using `typenum`.

use typenum::{Equal, Greater, IsGreater, IsGreaterOrEqual, IsLess, IsLessOrEqual, Less};

use crate::{
    eval::{EApply2, Evaluable, Evaluator, Sealed},
    std::bool::{Assert, ToTyBool, ToTyBoolOut, TyFalse, TyTrue},
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

// --- Helper Trait for Cmp to TyBool ---

#[doc(hidden)]
pub trait CmpToTyBool<Ordering> {
    type IsEq;
    type IsNeq;
    type IsLt;
    type IsLe;
    type IsGt;
    type IsGe;
}

impl<T> CmpToTyBool<Equal> for T {
    type IsEq = TyTrue;
    type IsNeq = TyFalse;
    type IsLt = TyFalse;
    type IsLe = TyTrue;
    type IsGt = TyFalse;
    type IsGe = TyTrue;
}

impl<T> CmpToTyBool<Less> for T {
    type IsEq = TyFalse;
    type IsNeq = TyTrue;
    type IsLt = TyTrue;
    type IsLe = TyTrue;
    type IsGt = TyFalse;
    type IsGe = TyFalse;
}

impl<T> CmpToTyBool<Greater> for T {
    type IsEq = TyFalse;
    type IsNeq = TyTrue;
    type IsLt = TyFalse;
    type IsLe = TyFalse;
    type IsGt = TyTrue;
    type IsGe = TyTrue;
}

// --- Implementations ---

// FEq: A == B (Generic via IsEq)
impl<A, B> Evaluable for EApply2<FEq, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: IsEq<Evaluator<B>>,
    (): crate::std::bool::Bool2TyBool<{ <Evaluator<A> as IsEq<Evaluator<B>>>::EQ }>,
{
    type Output = Evaluator<Assert<{ <Evaluator<A> as IsEq<Evaluator<B>>>::EQ }>>;
}

// FNeq: A != B (Generic via IsEq)
impl<A, B> Evaluable for EApply2<FNeq, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: IsEq<Evaluator<B>>,
    (): crate::std::bool::Bool2TyBool<{ !<Evaluator<A> as IsEq<Evaluator<B>>>::EQ }>,
{
    type Output = Evaluator<Assert<{ !<Evaluator<A> as IsEq<Evaluator<B>>>::EQ }>>;
}

// --- FLt: A < B ---
impl<A, B> Evaluable for EApply2<FLt, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: IsLess<Evaluator<B>>,
    <Evaluator<A> as IsLess<Evaluator<B>>>::Output: ToTyBool,
{
    type Output = ToTyBoolOut<<Evaluator<A> as IsLess<Evaluator<B>>>::Output>;
}

// --- FLe: A <= B ---
impl<A, B> Evaluable for EApply2<FLe, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: IsLessOrEqual<Evaluator<B>>,
    <Evaluator<A> as IsLessOrEqual<Evaluator<B>>>::Output: ToTyBool,
{
    type Output = ToTyBoolOut<<Evaluator<A> as IsLessOrEqual<Evaluator<B>>>::Output>;
}

// --- FGt: A > B ---
impl<A, B> Evaluable for EApply2<FGt, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: IsGreater<Evaluator<B>>,
    <Evaluator<A> as IsGreater<Evaluator<B>>>::Output: ToTyBool,
{
    type Output = ToTyBoolOut<<Evaluator<A> as IsGreater<Evaluator<B>>>::Output>;
}

// --- FGe: A >= B ---
impl<A, B> Evaluable for EApply2<FGe, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: IsGreaterOrEqual<Evaluator<B>>,
    <Evaluator<A> as IsGreaterOrEqual<Evaluator<B>>>::Output: ToTyBool,
{
    type Output = ToTyBoolOut<<Evaluator<A> as IsGreaterOrEqual<Evaluator<B>>>::Output>;
}

//
// Aliases
//

// Equality (2 args)
pub type EEq<A, B> = EApply2<FEq, A, B>;
pub type ENotEq<A, B> = EApply2<FNeq, A, B>;

// Comparison (2 args)
pub type ELt<A, B> = EApply2<FLt, A, B>;
pub type ELe<A, B> = EApply2<FLe, A, B>;
pub type EGt<A, B> = EApply2<FGt, A, B>;
pub type EGe<A, B> = EApply2<FGe, A, B>;

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
