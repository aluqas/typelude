//! Canonical expression aliases built on top of `Apply<OpX, Args>`.

use super::{
    Apply, OpAdd, OpAnd, OpAppend, OpConcat, OpDiv, OpEq, OpFold, OpGet, OpGt, OpHead, OpInto,
    OpLe, OpLen, OpLt, OpMap, OpMul, OpNand, OpNeq, OpNot, OpOr, OpPow, OpPrepend, OpRem, OpSet,
    OpSub, OpTail, OpXor,
};

/// Unary boolean negation.
pub type ENot<Arg> = Apply<OpNot, Arg>;
/// Binary boolean conjunction.
pub type EAnd<Lhs, Rhs> = Apply<OpAnd, (Lhs, Rhs)>;
/// Binary boolean disjunction.
pub type EOr<Lhs, Rhs> = Apply<OpOr, (Lhs, Rhs)>;
/// Binary boolean exclusive-or.
pub type EXor<Lhs, Rhs> = Apply<OpXor, (Lhs, Rhs)>;
/// Binary boolean nand.
pub type ENand<Lhs, Rhs> = Apply<OpNand, (Lhs, Rhs)>;

/// Binary addition.
pub type EAdd<Lhs, Rhs> = Apply<OpAdd, (Lhs, Rhs)>;
/// Binary subtraction.
pub type ESub<Lhs, Rhs> = Apply<OpSub, (Lhs, Rhs)>;
/// Binary multiplication.
pub type EMul<Lhs, Rhs> = Apply<OpMul, (Lhs, Rhs)>;
/// Binary division.
pub type EDiv<Lhs, Rhs> = Apply<OpDiv, (Lhs, Rhs)>;
/// Binary remainder.
pub type ERem<Lhs, Rhs> = Apply<OpRem, (Lhs, Rhs)>;
/// Binary exponentiation.
pub type EPow<Lhs, Rhs> = Apply<OpPow, (Lhs, Rhs)>;

/// Equality comparison.
pub type EEq<Lhs, Rhs> = Apply<OpEq, (Lhs, Rhs)>;
/// Inequality comparison.
pub type ENeq<Lhs, Rhs> = Apply<OpNeq, (Lhs, Rhs)>;
/// Strict less-than comparison.
pub type ELt<Lhs, Rhs> = Apply<OpLt, (Lhs, Rhs)>;
/// Less-than-or-equal comparison.
pub type ELe<Lhs, Rhs> = Apply<OpLe, (Lhs, Rhs)>;
/// Strict greater-than comparison.
pub type EGt<Lhs, Rhs> = Apply<OpGt, (Lhs, Rhs)>;

/// Collection length.
pub type ELen<Col> = Apply<OpLen, Col>;
/// Collection head.
pub type EHead<Col> = Apply<OpHead, Col>;
/// Collection tail.
pub type ETail<Col> = Apply<OpTail, Col>;
/// Indexed collection access.
pub type EGet<Col, Idx> = Apply<OpGet, (Col, Idx)>;
/// Indexed collection update.
pub type ESet<Col, Idx, Val> = Apply<OpSet, (Col, Idx, Val)>;
/// Collection concatenation.
pub type EConcat<Lhs, Rhs> = Apply<OpConcat, (Lhs, Rhs)>;
/// Append a single element to the end of a collection.
pub type EAppend<Col, Elem> = Apply<OpAppend, (Col, Elem)>;
/// Prepend a single element to the front of a collection.
pub type EPrepend<Elem, Col> = Apply<OpPrepend, (Elem, Col)>;
/// Map a first-class operator over a collection.
pub type EMap<Op, Col> = Apply<OpMap, (Op, Col)>;
/// Fold a collection with a first-class operator and initial accumulator.
pub type EFold<Op, Init, Col> = Apply<OpFold, (Op, Init, Col)>;

/// Type-level conversion.
pub type EInto<Source, Target> = Apply<OpInto, (Source, Target)>;
