//! **Comparison Operations**
//!
//! `typenum` を利用した大小比較演算の実装。

use typenum::{Equal, Greater, Less, IsGreater, IsGreaterOrEqual, IsLess, IsLessOrEqual};

use crate::{
    eval::{EApply2, Evaluable, Evaluator, Sealed},
    std::bool::{ToTyBool, ToTyBoolOut, Assert, TyTrue, TyFalse},
};

// =============================================================================
// IsEq Trait (Generic Equality)
// =============================================================================

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
// Function Markers: Comparison Operations
// =============================================================================

/// 等価比較: A == B -> TyBool
pub struct FEq;
/// 不等価比較: A != B -> TyBool
pub struct FNeq;
/// 小なり: A < B
pub struct FLt;
/// 以下: A <= B
pub struct FLe;
/// 大なり: A > B
pub struct FGt;
/// 以上: A >= B
pub struct FGe;

impl Sealed for FEq {}
impl Sealed for FNeq {}
impl Sealed for FLt {}
impl Sealed for FLe {}
impl Sealed for FGt {}
impl Sealed for FGe {}

// =============================================================================
// Comparison Operations
// =============================================================================

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

// =============================================================================
// Aliases
// =============================================================================

// Equality (2 args)
pub type EEq<A, B> = EApply2<FEq, A, B>;
pub type ENotEq<A, B> = EApply2<FNeq, A, B>;

// Comparison (2 args)
pub type ELt<A, B> = EApply2<FLt, A, B>;
pub type ELe<A, B> = EApply2<FLe, A, B>;
pub type EGt<A, B> = EApply2<FGt, A, B>;
pub type EGe<A, B> = EApply2<FGe, A, B>;

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
    fn test_compare_unsigned() {
        // 1 < 2 -> True
        assert_type_eq_all!(Evaluator<ELt<ELit<U1>, ELit<U2>>>, TyTrue);
        // 2 < 1 -> False
        assert_type_eq_all!(Evaluator<ELt<ELit<U2>, ELit<U1>>>, TyFalse);

        // 1 <= 1 -> True
        assert_type_eq_all!(Evaluator<ELe<ELit<U1>, ELit<U1>>>, TyTrue);
        // 2 <= 1 -> False
        assert_type_eq_all!(Evaluator<ELe<ELit<U2>, ELit<U1>>>, TyFalse);

        // 2 > 1 -> True
        assert_type_eq_all!(Evaluator<EGt<ELit<U2>, ELit<U1>>>, TyTrue);
        // 1 > 2 -> False
        assert_type_eq_all!(Evaluator<EGt<ELit<U1>, ELit<U2>>>, TyFalse);

        // 2 >= 2 -> True
        assert_type_eq_all!(Evaluator<EGe<ELit<U2>, ELit<U2>>>, TyTrue);
        // 1 >= 2 -> False
        assert_type_eq_all!(Evaluator<EGe<ELit<U1>, ELit<U2>>>, TyFalse);
    }

    #[test]
    fn test_compare_signed() {
        // -1 < 1 -> True
        assert_type_eq_all!(Evaluator<ELt<ELit<N1>, ELit<P1>>>, TyTrue);
        // 2 > -1 -> True
        assert_type_eq_all!(Evaluator<EGt<ELit<P2>, ELit<N1>>>, TyTrue);
    }
}
