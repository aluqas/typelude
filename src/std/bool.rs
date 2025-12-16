//! **Type-Level Boolean**
//!
//! Type-level booleans (`TyTrue`, `TyFalse`) and logical operations.

use typenum::{B0, B1};

use crate::eval::{EApply, EApply2, Evaluable, Evaluator, Sealed};

//
// Type-Level Boolean Types
//

/// Marker Trait: TyTrue, TyFalse
pub trait KindBool: Sealed + Evaluable {
    const BOOL: bool;
    type Or<Rhs: KindBool>: KindBool;
}

/// TyTrue: Type representing True
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, PartialOrd, Ord)]
pub struct TyTrue;

/// TyFalse: Type representing False
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, PartialOrd, Ord)]
pub struct TyFalse;

impl Sealed for TyTrue {}
impl Sealed for TyFalse {}

impl KindBool for TyTrue {
    const BOOL: bool = true;
    type Or<Rhs: KindBool> = TyTrue;
}
impl KindBool for TyFalse {
    const BOOL: bool = false;
    type Or<Rhs: KindBool> = Rhs;
}

impl Evaluable for TyTrue {
    type Output = TyTrue;
}
impl Evaluable for TyFalse {
    type Output = TyFalse;
}

//
// Bool Conversion Utilities
//

pub struct Assert<const COND: bool>;

impl<const COND: bool> Evaluable for Assert<COND>
where
    (): crate::std::reify::ReflectBool<COND>,
    <() as crate::std::reify::ReflectBool<COND>>::Output: Evaluable,
{
    type Output = <() as crate::std::reify::ReflectBool<COND>>::Output;
}

// Re-export for compatibility if needed, or just remove local traits.
// User asked to abstract it, implies replacement.

// Previous ToTyBool adaptation:
// B1 -> TyTrue, B0 -> TyFalse
// We use Translate<bool> for this if we want generic "To Boolean Type".
// Implementation of Translate<bool> for B1/B0 should be in `reify.rs` or here?
// Ideally here if B0/B1 are external, but `reify` is central.
// Let's implement Translate<bool> for B0/B1 here.

use crate::std::conv::TyFrom;

impl TyFrom<B1> for bool {
    type Output = TyTrue;
}
impl TyFrom<B0> for bool {
    type Output = TyFalse;
}

pub type ToTyBoolOut<T> = <bool as TyFrom<T>>::Output;

//
// Helper Traits for Logical Operations
//

/// Helper for NOT
#[doc(hidden)]
pub trait NotHelper {
    type Output;
}

impl NotHelper for TyTrue {
    type Output = TyFalse;
}
impl NotHelper for TyFalse {
    type Output = TyTrue;
}

/// Lazy NAND Helper (Short-circuit evaluation)
#[doc(hidden)]
pub trait NandHelper<Rhs> {
    type Output;
}

// False NAND X = True (Right side not evaluated)
impl<Rhs> NandHelper<Rhs> for TyFalse {
    type Output = TyTrue;
}

// True NAND X = NOT X
impl<Rhs> NandHelper<Rhs> for TyTrue
where
    Rhs: Evaluable,
    Evaluator<Rhs>: NotHelper,
{
    type Output = <Evaluator<Rhs> as NotHelper>::Output;
}

//
// Function Markers: Boolean Operations
//

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

//
// Evaluable Implementations for Boolean Functions
//

// FNot: NOT Val
impl<Val> Evaluable for EApply<FNot, Val>
where
    Val: Evaluable,
    Evaluator<Val>: NotHelper,
{
    type Output = <Evaluator<Val> as NotHelper>::Output;
}

// FNand: Lhs NAND Rhs (with short-circuit)
impl<Lhs, Rhs> Evaluable for EApply2<FNand, Lhs, Rhs>
where
    Lhs: Evaluable,
    Evaluator<Lhs>: NandHelper<Rhs>,
{
    type Output = <Evaluator<Lhs> as NandHelper<Rhs>>::Output;
}

// FAnd: Lhs AND Rhs = NOT (Lhs NAND Rhs)
impl<Lhs, Rhs> Evaluable for EApply2<FAnd, Lhs, Rhs>
where
    EApply<FNot, EApply2<FNand, Lhs, Rhs>>: Evaluable,
{
    type Output = Evaluator<EApply<FNot, EApply2<FNand, Lhs, Rhs>>>;
}

// FOr: Lhs OR Rhs = (NOT Lhs) NAND (NOT Rhs)
impl<Lhs, Rhs> Evaluable for EApply2<FOr, Lhs, Rhs>
where
    EApply2<FNand, EApply<FNot, Lhs>, EApply<FNot, Rhs>>: Evaluable,
{
    type Output = Evaluator<EApply2<FNand, EApply<FNot, Lhs>, EApply<FNot, Rhs>>>;
}

// FNor: Lhs NOR Rhs = NOT (Lhs OR Rhs)
impl<Lhs, Rhs> Evaluable for EApply2<FNor, Lhs, Rhs>
where
    EApply<FNot, EApply2<FOr, Lhs, Rhs>>: Evaluable,
{
    type Output = Evaluator<EApply<FNot, EApply2<FOr, Lhs, Rhs>>>;
}

// FXor: Lhs XOR Rhs = (Lhs OR Rhs) AND (Lhs NAND Rhs)
impl<Lhs, Rhs> Evaluable for EApply2<FXor, Lhs, Rhs>
where
    EApply2<FAnd, EApply2<FOr, Lhs, Rhs>, EApply2<FNand, Lhs, Rhs>>: Evaluable,
{
    type Output = Evaluator<EApply2<FAnd, EApply2<FOr, Lhs, Rhs>, EApply2<FNand, Lhs, Rhs>>>;
}

// FXnor: Lhs XNOR Rhs = NOT (Lhs XOR Rhs)
impl<Lhs, Rhs> Evaluable for EApply2<FXnor, Lhs, Rhs>
where
    EApply<FNot, EApply2<FXor, Lhs, Rhs>>: Evaluable,
{
    type Output = Evaluator<EApply<FNot, EApply2<FXor, Lhs, Rhs>>>;
}

//
// Aliases
//

// Boolean (1 arg)
pub type ENot<Val> = EApply<FNot, Val>;

// Boolean (2 args)
pub type EAnd<Lhs, Rhs> = EApply2<FAnd, Lhs, Rhs>;
pub type EOr<Lhs, Rhs> = EApply2<FOr, Lhs, Rhs>;
pub type ENand<Lhs, Rhs> = EApply2<FNand, Lhs, Rhs>;
pub type ENor<Lhs, Rhs> = EApply2<FNor, Lhs, Rhs>;
pub type EXor<Lhs, Rhs> = EApply2<FXor, Lhs, Rhs>;
pub type EXnor<Lhs, Rhs> = EApply2<FXnor, Lhs, Rhs>;

//
// Tests
//

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
