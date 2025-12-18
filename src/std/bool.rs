//! **Type-Level Boolean**
//!
//! Type-level booleans (`TyTrue`, `TyFalse`) and logical operations.

use std::marker::PhantomData;

use typenum::{B0, B1};

// Re-export kernel types
pub use crate::kernel::bool::{TyBool, TyFalse, TyTrue};
use crate::{
    eval::{Eval, Evaluate, Sealed},
    kernel::traits::Apply,
};

//
// Identity Eval Implementation (Should typically be in eval crate, but std is fine)
//

impl Eval for TyTrue {
    type Output = TyTrue;
}
impl Eval for TyFalse {
    type Output = TyFalse;
}

//
// KindBool (Extension)
//

/// Marker Trait: TyTrue, TyFalse
pub trait KindBool: Sealed + Eval {
    const BOOL: bool;
    type Or<Rhs: KindBool>: KindBool;
}

impl KindBool for TyTrue {
    const BOOL: bool = true;
    type Or<Rhs: KindBool> = TyTrue;
}
impl KindBool for TyFalse {
    const BOOL: bool = false;
    type Or<Rhs: KindBool> = Rhs;
}

//
// Bool Conversion Utilities
//

pub struct Assert<const COND: bool>;

impl<const COND: bool> Eval for Assert<COND>
where
    (): crate::std::reify::ReflectBool<COND>,
    <() as crate::std::reify::ReflectBool<COND>>::Output: Eval,
{
    type Output = <() as crate::std::reify::ReflectBool<COND>>::Output;
}

use crate::std::into::TyFrom;

impl TyFrom<B1> for bool {
    type Output = TyTrue;
}
impl TyFrom<B0> for bool {
    type Output = TyFalse;
}

pub type ToTyBoolOut<T> = <bool as TyFrom<T>>::Output;

//
// Helper Traits
//

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

#[doc(hidden)]
pub trait NandHelper<Rhs> {
    type Output;
}
impl<Rhs> NandHelper<Rhs> for TyFalse {
    type Output = TyTrue;
}
impl<Rhs> NandHelper<Rhs> for TyTrue
where
    Rhs: Eval,
    Evaluate<Rhs>: NotHelper,
{
    type Output = <Evaluate<Rhs> as NotHelper>::Output;
}

//
// Expression Structs (Direct Style)
//

/// Expression: NOT A
pub struct ENot<Val>(PhantomData<Val>);

impl<Val> Eval for ENot<Val>
where
    Val: Eval,
    Evaluate<Val>: NotHelper,
{
    type Output = <Evaluate<Val> as NotHelper>::Output;
}

/// Expression: A NAND B
pub struct ENand<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for ENand<Lhs, Rhs>
where
    Lhs: Eval,
    Evaluate<Lhs>: NandHelper<Rhs>,
{
    type Output = <Evaluate<Lhs> as NandHelper<Rhs>>::Output;
}

/// Expression: A AND B
pub struct EAnd<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for EAnd<Lhs, Rhs>
where
    ENot<ENand<Lhs, Rhs>>: Eval,
{
    type Output = Evaluate<ENot<ENand<Lhs, Rhs>>>;
}

/// Expression: A OR B
pub struct EOr<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for EOr<Lhs, Rhs>
where
    ENand<ENot<Lhs>, ENot<Rhs>>: Eval,
{
    type Output = Evaluate<ENand<ENot<Lhs>, ENot<Rhs>>>;
}

/// Expression: A NOR B
pub struct ENor<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for ENor<Lhs, Rhs>
where
    ENot<EOr<Lhs, Rhs>>: Eval,
{
    type Output = Evaluate<ENot<EOr<Lhs, Rhs>>>;
}

/// Expression: A XOR B
pub struct EXor<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for EXor<Lhs, Rhs>
where
    EAnd<EOr<Lhs, Rhs>, ENand<Lhs, Rhs>>: Eval,
{
    type Output = Evaluate<EAnd<EOr<Lhs, Rhs>, ENand<Lhs, Rhs>>>;
}

/// Expression: A XNOR B
pub struct EXnor<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for EXnor<Lhs, Rhs>
where
    ENot<EXor<Lhs, Rhs>>: Eval,
{
    type Output = Evaluate<ENot<EXor<Lhs, Rhs>>>;
}

//
// Operator Symbols (For HOF)
//

pub struct OpNot;
pub struct OpAnd;
pub struct OpOr;
pub struct OpNand;
pub struct OpNor;
pub struct OpXor;
pub struct OpXnor;

impl Sealed for OpNot {}
impl Sealed for OpAnd {}
impl Sealed for OpOr {}
impl Sealed for OpNand {}
impl Sealed for OpNor {}
impl Sealed for OpXor {}
impl Sealed for OpXnor {}

impl<Val> Apply<Val> for OpNot {
    type Output = ENot<Val>;
}

impl<Lhs, Rhs> Apply<(Lhs, Rhs)> for OpAnd {
    type Output = EAnd<Lhs, Rhs>;
}

impl<Lhs, Rhs> Apply<(Lhs, Rhs)> for OpOr {
    type Output = EOr<Lhs, Rhs>;
}

impl<Lhs, Rhs> Apply<(Lhs, Rhs)> for OpNand {
    type Output = ENand<Lhs, Rhs>;
}

impl<Lhs, Rhs> Apply<(Lhs, Rhs)> for OpNor {
    type Output = ENor<Lhs, Rhs>;
}

impl<Lhs, Rhs> Apply<(Lhs, Rhs)> for OpXor {
    type Output = EXor<Lhs, Rhs>;
}

impl<Lhs, Rhs> Apply<(Lhs, Rhs)> for OpXnor {
    type Output = EXnor<Lhs, Rhs>;
}

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
        assert_type_eq_all!(Evaluate<ENot<ELit<TyTrue>>>, TyFalse);
        assert_type_eq_all!(Evaluate<ENot<ELit<TyFalse>>>, TyTrue);
    }

    #[test]
    fn test_nand() {
        assert_type_eq_all!(Evaluate<ENand<ELit<TyTrue>, ELit<TyTrue>>>, TyFalse);
        assert_type_eq_all!(Evaluate<ENand<ELit<TyTrue>, ELit<TyFalse>>>, TyTrue);
        assert_type_eq_all!(Evaluate<ENand<ELit<TyFalse>, ELit<TyTrue>>>, TyTrue);
        assert_type_eq_all!(Evaluate<ENand<ELit<TyFalse>, ELit<TyFalse>>>, TyTrue);
    }

    #[test]
    fn test_and() {
        assert_type_eq_all!(Evaluate<EAnd<ELit<TyTrue>, ELit<TyTrue>>>, TyTrue);
        assert_type_eq_all!(Evaluate<EAnd<ELit<TyTrue>, ELit<TyFalse>>>, TyFalse);
        assert_type_eq_all!(Evaluate<EAnd<ELit<TyFalse>, ELit<TyTrue>>>, TyFalse);
        assert_type_eq_all!(Evaluate<EAnd<ELit<TyFalse>, ELit<TyFalse>>>, TyFalse);
    }

    #[test]
    fn test_or() {
        assert_type_eq_all!(Evaluate<EOr<ELit<TyTrue>, ELit<TyTrue>>>, TyTrue);
        assert_type_eq_all!(Evaluate<EOr<ELit<TyTrue>, ELit<TyFalse>>>, TyTrue);
        assert_type_eq_all!(Evaluate<EOr<ELit<TyFalse>, ELit<TyTrue>>>, TyTrue);
        assert_type_eq_all!(Evaluate<EOr<ELit<TyFalse>, ELit<TyFalse>>>, TyFalse);
    }

    #[test]
    fn test_nor() {
        assert_type_eq_all!(Evaluate<ENor<ELit<TyTrue>, ELit<TyTrue>>>, TyFalse);
        assert_type_eq_all!(Evaluate<ENor<ELit<TyTrue>, ELit<TyFalse>>>, TyFalse);
        assert_type_eq_all!(Evaluate<ENor<ELit<TyFalse>, ELit<TyTrue>>>, TyFalse);
        assert_type_eq_all!(Evaluate<ENor<ELit<TyFalse>, ELit<TyFalse>>>, TyTrue);
    }

    #[test]
    fn test_xor() {
        assert_type_eq_all!(Evaluate<EXor<ELit<TyTrue>, ELit<TyTrue>>>, TyFalse);
        assert_type_eq_all!(Evaluate<EXor<ELit<TyTrue>, ELit<TyFalse>>>, TyTrue);
        assert_type_eq_all!(Evaluate<EXor<ELit<TyFalse>, ELit<TyTrue>>>, TyTrue);
        assert_type_eq_all!(Evaluate<EXor<ELit<TyFalse>, ELit<TyFalse>>>, TyFalse);
    }

    #[test]
    fn test_xnor() {
        assert_type_eq_all!(Evaluate<EXnor<ELit<TyTrue>, ELit<TyTrue>>>, TyTrue);
        assert_type_eq_all!(Evaluate<EXnor<ELit<TyTrue>, ELit<TyFalse>>>, TyFalse);
        assert_type_eq_all!(Evaluate<EXnor<ELit<TyFalse>, ELit<TyTrue>>>, TyFalse);
        assert_type_eq_all!(Evaluate<EXnor<ELit<TyFalse>, ELit<TyFalse>>>, TyTrue);
    }

    #[test]
    fn test_composition() {
        // NOT (True AND False) = True
        assert_type_eq_all!(Evaluate<ENot<EAnd<ELit<TyTrue>, ELit<TyFalse>>>>, TyTrue);

        // (True OR False) AND (False OR True) = True
        assert_type_eq_all!(
            Evaluate<EAnd<EOr<ELit<TyTrue>, ELit<TyFalse>>, EOr<ELit<TyFalse>, ELit<TyTrue>>>>,
            TyTrue
        );
    }
}
