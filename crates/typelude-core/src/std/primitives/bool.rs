//! **Type-Level Boolean**
//!
//! Type-level booleans (`TyTrue`, `TyFalse`) and logical operations.

use std::marker::PhantomData;

use typenum::{B0, B1};

// Re-export kernel types
pub use crate::kernel::bool::{Bool as TyBool, TyFalse, TyTrue};
use crate::{
    eval::{Eval, Evaluate, Sealed},
    kernel::traits::Apply,
    std::traits::TypeBool,
};

//
// Adapter Implementation: TypeBool for TyTrue, TyFalse, B0, B1
//

// --- TyTrue / TyFalse ---

impl TypeBool for TyTrue {
    type Not = TyFalse;
    type And<Rhs: TypeBool> = Rhs;
    type Or<Rhs: TypeBool> = TyTrue;
    type Xor<Rhs: TypeBool> = Rhs::Not;
    type Nand<Rhs: TypeBool> = Rhs::Not;
    type Nor<Rhs: TypeBool> = TyFalse;
    type Xnor<Rhs: TypeBool> = Rhs;
}

impl TypeBool for TyFalse {
    type Not = TyTrue;
    type And<Rhs: TypeBool> = TyFalse;
    type Or<Rhs: TypeBool> = Rhs;
    type Xor<Rhs: TypeBool> = Rhs;
    type Nand<Rhs: TypeBool> = TyTrue;
    type Nor<Rhs: TypeBool> = Rhs::Not;
    type Xnor<Rhs: TypeBool> = Rhs::Not;
}

// --- B0 / B1 (typenum interoperability) ---

impl TypeBool for B0 {
    type Not = B1;
    type And<Rhs: TypeBool> = B0;
    type Or<Rhs: TypeBool> = Rhs;
    type Xor<Rhs: TypeBool> = Rhs;
    type Nand<Rhs: TypeBool> = B1;
    type Nor<Rhs: TypeBool> = Rhs::Not;
    type Xnor<Rhs: TypeBool> = Rhs::Not;
}

impl TypeBool for B1 {
    type Not = B0;
    type And<Rhs: TypeBool> = Rhs;
    type Or<Rhs: TypeBool> = B1;
    type Xor<Rhs: TypeBool> = Rhs::Not;
    type Nand<Rhs: TypeBool> = Rhs::Not;
    type Nor<Rhs: TypeBool> = B0;
    type Xnor<Rhs: TypeBool> = Rhs;
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
// Expression Structs (Refactored to use TypeBool)
//

/// Expression: NOT A
pub struct ENot<Val>(PhantomData<Val>);

impl<Val> Eval for ENot<Val>
where
    Val: Eval,
    Evaluate<Val>: TypeBool,
{
    type Output = <Evaluate<Val> as TypeBool>::Not;
}

/// Expression: A NAND B
pub struct ENand<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for ENand<Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Evaluate<Lhs>: TypeBool,
    Evaluate<Rhs>: TypeBool,
{
    type Output = <Evaluate<Lhs> as TypeBool>::Nand<Evaluate<Rhs>>;
}

/// Expression: A AND B
pub struct EAnd<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for EAnd<Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Evaluate<Lhs>: TypeBool,
    Evaluate<Rhs>: TypeBool,
{
    type Output = <Evaluate<Lhs> as TypeBool>::And<Evaluate<Rhs>>;
}

/// Expression: A OR B
pub struct EOr<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for EOr<Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Evaluate<Lhs>: TypeBool,
    Evaluate<Rhs>: TypeBool,
{
    type Output = <Evaluate<Lhs> as TypeBool>::Or<Evaluate<Rhs>>;
}

/// Expression: A NOR B
pub struct ENor<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for ENor<Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Evaluate<Lhs>: TypeBool,
    Evaluate<Rhs>: TypeBool,
{
    type Output = <Evaluate<Lhs> as TypeBool>::Nor<Evaluate<Rhs>>;
}

/// Expression: A XOR B
pub struct EXor<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for EXor<Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Evaluate<Lhs>: TypeBool,
    Evaluate<Rhs>: TypeBool,
{
    type Output = <Evaluate<Lhs> as TypeBool>::Xor<Evaluate<Rhs>>;
}

/// Expression: A XNOR B
pub struct EXnor<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for EXnor<Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Evaluate<Lhs>: TypeBool,
    Evaluate<Rhs>: TypeBool,
{
    type Output = <Evaluate<Lhs> as TypeBool>::Xnor<Evaluate<Rhs>>;
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
        // Interop test
        assert_type_eq_all!(Evaluate<ENot<ELit<B1>>>, B0);
    }

    #[test]
    fn test_nand() {
        assert_type_eq_all!(Evaluate<ENand<ELit<TyTrue>, ELit<TyTrue>>>, TyFalse);
        assert_type_eq_all!(Evaluate<ENand<ELit<TyTrue>, ELit<TyFalse>>>, TyTrue);
    }

    #[test]
    fn test_and() {
        assert_type_eq_all!(Evaluate<EAnd<ELit<TyTrue>, ELit<TyTrue>>>, TyTrue);
        assert_type_eq_all!(Evaluate<EAnd<ELit<TyTrue>, ELit<TyFalse>>>, TyFalse);
        // Interop: TyTrue AND B0 -> TyFalse
        // Note: The Output type depends on Lhs implementation.
        // TyTrue::And<B0> -> B0.
        assert_type_eq_all!(Evaluate<EAnd<ELit<TyTrue>, ELit<B0>>>, B0);
        // B1::And<TyTrue> -> TyTrue
        assert_type_eq_all!(Evaluate<EAnd<ELit<B1>, ELit<TyTrue>>>, TyTrue);
    }

    #[test]
    fn test_or() {
        assert_type_eq_all!(Evaluate<EOr<ELit<TyTrue>, ELit<TyTrue>>>, TyTrue);
        assert_type_eq_all!(Evaluate<EOr<ELit<TyFalse>, ELit<TyFalse>>>, TyFalse);
    }
}
