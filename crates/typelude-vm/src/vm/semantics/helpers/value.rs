use typelude_std::core::{ELit, Eval, Evaluate};

use crate::opcode::numeric::{OpAdd, OpAnd, OpEq, OpGt, OpLt, OpNeq, OpOr, OpSub};

pub trait AsValueExpr {
    type Output;
}

impl<T> AsValueExpr for ELit<T> {
    type Output = ELit<T>;
}

impl AsValueExpr for typenum::UTerm {
    type Output = ELit<typenum::UTerm>;
}

impl<N, B> AsValueExpr for typenum::UInt<N, B> {
    type Output = ELit<typenum::UInt<N, B>>;
}

impl<U> AsValueExpr for typenum::PInt<U>
where
    U: typenum::Unsigned + typenum::NonZero,
{
    type Output = ELit<typenum::PInt<U>>;
}

impl<U> AsValueExpr for typenum::NInt<U>
where
    U: typenum::Unsigned + typenum::NonZero,
{
    type Output = ELit<typenum::NInt<U>>;
}

impl AsValueExpr for typenum::Z0 {
    type Output = ELit<typenum::Z0>;
}

impl AsValueExpr for typenum::B0 {
    type Output = ELit<typenum::B0>;
}

impl AsValueExpr for typenum::B1 {
    type Output = ELit<typenum::B1>;
}

impl AsValueExpr for typelude_std::std::prim::bool::True {
    type Output = ELit<typelude_std::std::prim::bool::True>;
}

impl AsValueExpr for typelude_std::std::prim::bool::False {
    type Output = ELit<typelude_std::std::prim::bool::False>;
}

pub trait BinaryResult<Lhs, Rhs> {
    type Output;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpAdd
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::EAdd<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::EAdd<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpSub
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::ESub<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::ESub<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

#[cfg(feature = "nightly")]
impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpEq
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::EEq<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::EEq<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

#[cfg(feature = "nightly")]
impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpNeq
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::ENeq<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::ENeq<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpLt
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::ELt<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::ELt<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpGt
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::EGt<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::EGt<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpAnd
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::EAnd<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::EAnd<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpOr
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::EOr<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::EOr<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}
