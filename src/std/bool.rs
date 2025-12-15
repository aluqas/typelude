//! **Type-Level Boolean**
//!
//! 型レベルブール値 (`TyTrue`, `TyFalse`) と論理演算を提供します。

use typenum::{B0, B1};

use crate::eval::{EApply, EApply2, Evaluable, Evaluator, Sealed};

// =============================================================================
// Type-Level Boolean Types
// =============================================================================

/// Marker Trait: TyTrue, TyFalse
pub trait AsBool: Sealed + Evaluable {
    const BOOL: bool;
    type Or<Rhs: AsBool>: AsBool;
}

/// TyTrue: True を表す型
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, PartialOrd, Ord)]
pub struct TyTrue;

/// TyFalse: False を表す型
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, PartialOrd, Ord)]
pub struct TyFalse;

impl Sealed for TyTrue {}
impl Sealed for TyFalse {}

impl AsBool for TyTrue {
    const BOOL: bool = true;
    type Or<Rhs: AsBool> = TyTrue;
}
impl AsBool for TyFalse {
    const BOOL: bool = false;
    type Or<Rhs: AsBool> = Rhs;
}

impl Evaluable for TyTrue {
    type Output = TyTrue;
}
impl Evaluable for TyFalse {
    type Output = TyFalse;
}

/// Marker Trait: TyTrue
pub trait IsTrue: AsBool {}
impl IsTrue for TyTrue {}

/// Marker Trait: TyFalse
pub trait IsFalse: AsBool {}
impl IsFalse for TyFalse {}

// =============================================================================
// Bool Conversion Utilities
// =============================================================================

/// 真偽値をTyBool型に変換するトレイト
#[doc(hidden)]
pub trait Bool2TyBool<const COND: bool> {
    type Output: AsBool;
}
impl Bool2TyBool<true> for () {
    type Output = TyTrue;
}
impl Bool2TyBool<false> for () {
    type Output = TyFalse;
}

pub struct Assert<const COND: bool>;

impl<const COND: bool> Evaluable for Assert<COND>
where
    (): Bool2TyBool<COND>,
{
    type Output = <() as Bool2TyBool<COND>>::Output;
}

/// typenum::B0/B1 → TyFalse/TyTrue 変換
pub trait ToTyBool {
    type Output;
}

impl ToTyBool for B1 {
    type Output = TyTrue;
}
impl ToTyBool for B0 {
    type Output = TyFalse;
}

pub type ToTyBoolOut<T> = <T as ToTyBool>::Output;

// =============================================================================
// Helper Traits for Logical Operations
// =============================================================================

/// NOT のヘルパー
#[doc(hidden)]
pub trait _NotHelper {
    type Output;
}

impl _NotHelper for TyTrue {
    type Output = TyFalse;
}
impl _NotHelper for TyFalse {
    type Output = TyTrue;
}

/// Lazy NAND Helper (短絡評価)
#[doc(hidden)]
pub trait _NandHelper<Rhs> {
    type Output;
}

// False NAND X = True (右辺を評価しない)
impl<Rhs> _NandHelper<Rhs> for TyFalse {
    type Output = TyTrue;
}

// True NAND X = NOT X
impl<Rhs> _NandHelper<Rhs> for TyTrue
where
    Rhs: Evaluable,
    Evaluator<Rhs>: _NotHelper,
{
    type Output = <Evaluator<Rhs> as _NotHelper>::Output;
}

// =============================================================================
// Function Markers: Boolean Operations
// =============================================================================

/// NOT: !A
pub struct FNot;
/// AND: A && B
pub struct FAnd;
/// OR: A || B
pub struct FOr;
/// NAND: !(A && B)
pub struct FNand;
/// NOR: !(A || B)
pub struct FNor;
/// XOR: A ^ B
pub struct FXor;
/// XNOR: !(A ^ B)
pub struct FXnor;

impl Sealed for FNot {}
impl Sealed for FAnd {}
impl Sealed for FOr {}
impl Sealed for FNand {}
impl Sealed for FNor {}
impl Sealed for FXor {}
impl Sealed for FXnor {}

// =============================================================================
// Evaluable Implementations for Boolean Functions
// =============================================================================

// --- FNot: NOT A ---
impl<A> Evaluable for EApply<FNot, A>
where
    A: Evaluable,
    Evaluator<A>: _NotHelper,
{
    type Output = <Evaluator<A> as _NotHelper>::Output;
}

// --- FNand: A NAND B (with short-circuit) ---
impl<A, B> Evaluable for EApply2<FNand, A, B>
where
    A: Evaluable,
    Evaluator<A>: _NandHelper<B>,
{
    type Output = <Evaluator<A> as _NandHelper<B>>::Output;
}

// --- FAnd: A AND B = NOT (A NAND B) ---
impl<A, B> Evaluable for EApply2<FAnd, A, B>
where
    EApply<FNot, EApply2<FNand, A, B>>: Evaluable,
{
    type Output = Evaluator<EApply<FNot, EApply2<FNand, A, B>>>;
}

// --- FOr: A OR B = (NOT A) NAND (NOT B) ---
impl<A, B> Evaluable for EApply2<FOr, A, B>
where
    EApply2<FNand, EApply<FNot, A>, EApply<FNot, B>>: Evaluable,
{
    type Output = Evaluator<EApply2<FNand, EApply<FNot, A>, EApply<FNot, B>>>;
}

// --- FNor: A NOR B = NOT (A OR B) ---
impl<A, B> Evaluable for EApply2<FNor, A, B>
where
    EApply<FNot, EApply2<FOr, A, B>>: Evaluable,
{
    type Output = Evaluator<EApply<FNot, EApply2<FOr, A, B>>>;
}

// --- FXor: A XOR B = (A OR B) AND (A NAND B) ---
impl<A, B> Evaluable for EApply2<FXor, A, B>
where
    EApply2<FAnd, EApply2<FOr, A, B>, EApply2<FNand, A, B>>: Evaluable,
{
    type Output = Evaluator<EApply2<FAnd, EApply2<FOr, A, B>, EApply2<FNand, A, B>>>;
}

// --- FXnor: A XNOR B = NOT (A XOR B) ---
impl<A, B> Evaluable for EApply2<FXnor, A, B>
where
    EApply<FNot, EApply2<FXor, A, B>>: Evaluable,
{
    type Output = Evaluator<EApply<FNot, EApply2<FXor, A, B>>>;
}

// =============================================================================
// Aliases
// =============================================================================

// Boolean (1 arg)
pub type ENot<A> = EApply<FNot, A>;

// Boolean (2 args)
pub type EAnd<A, B> = EApply2<FAnd, A, B>;
pub type EOr<A, B> = EApply2<FOr, A, B>;
pub type ENand<A, B> = EApply2<FNand, A, B>;
pub type ENor<A, B> = EApply2<FNor, A, B>;
pub type EXor<A, B> = EApply2<FXor, A, B>;
pub type EXnor<A, B> = EApply2<FXnor, A, B>;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::eval::ELit;

    #[test]
    fn test_not() {
        assert_type_eq_all!(Evaluator<ENot<ELit<TyTrue>>>, TyFalse);
        assert_type_eq_all!(Evaluator<ENot<ELit<TyFalse>>>, TyTrue);
    }

    #[test]
    fn test_nand() {
        assert_type_eq_all!(Evaluator<ENand<ELit<TyTrue>, ELit<TyTrue>>>, TyFalse);
        assert_type_eq_all!(Evaluator<ENand<ELit<TyTrue>, ELit<TyFalse>>>, TyTrue);
        assert_type_eq_all!(Evaluator<ENand<ELit<TyFalse>, ELit<TyTrue>>>, TyTrue);
        assert_type_eq_all!(Evaluator<ENand<ELit<TyFalse>, ELit<TyFalse>>>, TyTrue);
    }

    #[test]
    fn test_and() {
        assert_type_eq_all!(Evaluator<EAnd<ELit<TyTrue>, ELit<TyTrue>>>, TyTrue);
        assert_type_eq_all!(Evaluator<EAnd<ELit<TyTrue>, ELit<TyFalse>>>, TyFalse);
        assert_type_eq_all!(Evaluator<EAnd<ELit<TyFalse>, ELit<TyTrue>>>, TyFalse);
        assert_type_eq_all!(Evaluator<EAnd<ELit<TyFalse>, ELit<TyFalse>>>, TyFalse);
    }

    #[test]
    fn test_or() {
        assert_type_eq_all!(Evaluator<EOr<ELit<TyTrue>, ELit<TyTrue>>>, TyTrue);
        assert_type_eq_all!(Evaluator<EOr<ELit<TyTrue>, ELit<TyFalse>>>, TyTrue);
        assert_type_eq_all!(Evaluator<EOr<ELit<TyFalse>, ELit<TyTrue>>>, TyTrue);
        assert_type_eq_all!(Evaluator<EOr<ELit<TyFalse>, ELit<TyFalse>>>, TyFalse);
    }

    #[test]
    fn test_nor() {
        assert_type_eq_all!(Evaluator<ENor<ELit<TyTrue>, ELit<TyTrue>>>, TyFalse);
        assert_type_eq_all!(Evaluator<ENor<ELit<TyTrue>, ELit<TyFalse>>>, TyFalse);
        assert_type_eq_all!(Evaluator<ENor<ELit<TyFalse>, ELit<TyTrue>>>, TyFalse);
        assert_type_eq_all!(Evaluator<ENor<ELit<TyFalse>, ELit<TyFalse>>>, TyTrue);
    }

    #[test]
    fn test_xor() {
        assert_type_eq_all!(Evaluator<EXor<ELit<TyTrue>, ELit<TyTrue>>>, TyFalse);
        assert_type_eq_all!(Evaluator<EXor<ELit<TyTrue>, ELit<TyFalse>>>, TyTrue);
        assert_type_eq_all!(Evaluator<EXor<ELit<TyFalse>, ELit<TyTrue>>>, TyTrue);
        assert_type_eq_all!(Evaluator<EXor<ELit<TyFalse>, ELit<TyFalse>>>, TyFalse);
    }

    #[test]
    fn test_xnor() {
        assert_type_eq_all!(Evaluator<EXnor<ELit<TyTrue>, ELit<TyTrue>>>, TyTrue);
        assert_type_eq_all!(Evaluator<EXnor<ELit<TyTrue>, ELit<TyFalse>>>, TyFalse);
        assert_type_eq_all!(Evaluator<EXnor<ELit<TyFalse>, ELit<TyTrue>>>, TyFalse);
        assert_type_eq_all!(Evaluator<EXnor<ELit<TyFalse>, ELit<TyFalse>>>, TyTrue);
    }

    #[test]
    fn test_composition() {
        // NOT (True AND False) = True
        assert_type_eq_all!(Evaluator<ENot<EAnd<ELit<TyTrue>, ELit<TyFalse>>>>, TyTrue);

        // (True OR False) AND (False OR True) = True
        assert_type_eq_all!(
            Evaluator<EAnd<EOr<ELit<TyTrue>, ELit<TyFalse>>, EOr<ELit<TyFalse>, ELit<TyTrue>>>>,
            TyTrue
        );
    }
}
