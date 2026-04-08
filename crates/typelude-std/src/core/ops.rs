//! Common operator marker structs used as first-class higher-order symbols.

macro_rules! define_ops {
    ($($name:ident),+ $(,)?) => {
        $(
            #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct $name;
        )+
    };
}

define_ops!(
    OpAdd, OpSub, OpMul, OpDiv, OpRem, OpPow, OpEq, OpNeq, OpLt, OpGt, OpLe, OpLen, OpHead,
    OpTail, OpNot, OpAnd, OpOr, OpXor, OpNand, OpConcat, OpGet, OpSet, OpMap, OpFold, OpIf,
    OpWhile, OpInto
);
