//! Value normalization helpers shared by pure instruction semantics.

use typelude_bool::{False, True};
use typelude_std::core::{Add, And, Eq, Gt, Lt, Neq, Not, Or, Sub};

use crate::{
    opcode::numeric::{OpAdd, OpAnd, OpEq, OpGt, OpLt, OpNeq, OpNot, OpOr, OpSub},
    vm::value::Lit,
};

/// Canonicalizes all boolean-like result types to `True` or `False`.
pub trait NormalizeBool {
    type Output;
}

impl NormalizeBool for True {
    type Output = True;
}

impl NormalizeBool for False {
    type Output = False;
}

impl NormalizeBool for typenum::B1 {
    type Output = True;
}

impl NormalizeBool for typenum::B0 {
    type Output = False;
}

pub trait UnaryInstrResult<Inst> {
    type Output;
}

pub trait BinaryInstrResult<Inst, Rhs> {
    type Output;
}

impl<Value> UnaryInstrResult<OpNot> for Lit<Value>
where
    Value: Not,
    <Value as Not>::Output: NormalizeBool,
{
    type Output = Lit<<<Value as Not>::Output as NormalizeBool>::Output>;
}

impl<Lhs, Rhs> BinaryInstrResult<OpAdd, Lit<Rhs>> for Lit<Lhs>
where
    Lhs: Add<Rhs>,
{
    type Output = Lit<<Lhs as Add<Rhs>>::Output>;
}

impl<Lhs, Rhs> BinaryInstrResult<OpSub, Lit<Rhs>> for Lit<Lhs>
where
    Lhs: Sub<Rhs>,
{
    type Output = Lit<<Lhs as Sub<Rhs>>::Output>;
}

impl<Lhs, Rhs> BinaryInstrResult<OpEq, Lit<Rhs>> for Lit<Lhs>
where
    Lhs: Eq<Rhs>,
    <Lhs as Eq<Rhs>>::Output: NormalizeBool,
{
    type Output = Lit<<<Lhs as Eq<Rhs>>::Output as NormalizeBool>::Output>;
}

impl<Lhs, Rhs> BinaryInstrResult<OpNeq, Lit<Rhs>> for Lit<Lhs>
where
    Lhs: Neq<Rhs>,
    <Lhs as Neq<Rhs>>::Output: NormalizeBool,
{
    type Output = Lit<<<Lhs as Neq<Rhs>>::Output as NormalizeBool>::Output>;
}

impl<Lhs, Rhs> BinaryInstrResult<OpLt, Lit<Rhs>> for Lit<Lhs>
where
    Lhs: Lt<Rhs>,
    <Lhs as Lt<Rhs>>::Output: NormalizeBool,
{
    type Output = Lit<<<Lhs as Lt<Rhs>>::Output as NormalizeBool>::Output>;
}

impl<Lhs, Rhs> BinaryInstrResult<OpGt, Lit<Rhs>> for Lit<Lhs>
where
    Lhs: Gt<Rhs>,
    <Lhs as Gt<Rhs>>::Output: NormalizeBool,
{
    type Output = Lit<<<Lhs as Gt<Rhs>>::Output as NormalizeBool>::Output>;
}

impl<Lhs, Rhs> BinaryInstrResult<OpAnd, Lit<Rhs>> for Lit<Lhs>
where
    Lhs: And<Rhs>,
    <Lhs as And<Rhs>>::Output: NormalizeBool,
{
    type Output = Lit<<<Lhs as And<Rhs>>::Output as NormalizeBool>::Output>;
}

impl<Lhs, Rhs> BinaryInstrResult<OpOr, Lit<Rhs>> for Lit<Lhs>
where
    Lhs: Or<Rhs>,
    <Lhs as Or<Rhs>>::Output: NormalizeBool,
{
    type Output = Lit<<<Lhs as Or<Rhs>>::Output as NormalizeBool>::Output>;
}
