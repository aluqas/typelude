//! Common operator marker structs used as first-class higher-order symbols.
//!
//! These marker types are the canonical operator surface for shared type-level
//! APIs. They do not require every expression to lower through
//! `Apply<OpX, Args>`, but common higher-order interfaces should refer to
//! these names when describing arithmetic, boolean, collection, control-flow,
//! and conversion operations.
//!
//! Canonical operator groups:
//! - Arithmetic: `OpAdd`, `OpSub`, `OpMul`, `OpDiv`, `OpRem`, `OpPow`
//! - Comparison: `OpEq`, `OpNeq`, `OpLt`, `OpLe`, `OpGt`
//! - Boolean: `OpNot`, `OpAnd`, `OpOr`, `OpXor`, `OpNand`
//! - Collections: `OpLen`, `OpHead`, `OpTail`, `OpGet`, `OpSet`, `OpConcat`,
//!   `OpAppend`, `OpPrepend`, `OpMap`, `OpFold`
//! - Control flow: `OpIf`, `OpWhile`
//! - Conversion: `OpInto`

use super::value::Value;

macro_rules! define_ops {
    ($($name:ident),+ $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $name;

            impl Value for $name {}
        )+
    };
}

define_ops!(
    OpAdd, OpSub, OpMul, OpDiv, OpRem, OpPow, OpEq, OpNeq, OpLt, OpGt, OpLe, OpLen, OpHead,
    OpTail, OpNot, OpAnd, OpOr, OpXor, OpNand, OpConcat, OpGet, OpSet, OpMap, OpFold, OpIf,
    OpWhile, OpInto, OpAppend, OpPrepend
);
