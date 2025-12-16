use crate::{machine::instruction::*, std::trace::Trace};

impl<Val: Trace> Trace for OpPush<Val> {
    fn fmt() -> String {
        format!("Push({})", Val::fmt())
    }
}

impl Trace for OpAdd {
    fn fmt() -> String {
        "Add".to_string()
    }
}
impl Trace for OpSub {
    fn fmt() -> String {
        "Sub".to_string()
    }
}
impl Trace for OpDup {
    fn fmt() -> String {
        "Dup".to_string()
    }
}
impl Trace for OpSwap {
    fn fmt() -> String {
        "Swap".to_string()
    }
}
impl Trace for OpDrop {
    fn fmt() -> String {
        "Drop".to_string()
    }
}
impl Trace for OpEq {
    fn fmt() -> String {
        "Eq".to_string()
    }
}
impl Trace for OpNeq {
    fn fmt() -> String {
        "Neq".to_string()
    }
}
impl Trace for OpLt {
    fn fmt() -> String {
        "Lt".to_string()
    }
}
impl Trace for OpGt {
    fn fmt() -> String {
        "Gt".to_string()
    }
}
impl Trace for OpNot {
    fn fmt() -> String {
        "Not".to_string()
    }
}
impl Trace for OpAnd {
    fn fmt() -> String {
        "And".to_string()
    }
}
impl Trace for OpOr {
    fn fmt() -> String {
        "Or".to_string()
    }
}
impl Trace for OpLoad {
    fn fmt() -> String {
        "Load".to_string()
    }
}
impl Trace for OpStore {
    fn fmt() -> String {
        "Store".to_string()
    }
}
impl Trace for OpReturn {
    fn fmt() -> String {
        "Return".to_string()
    }
}
impl Trace for OpLet {
    fn fmt() -> String {
        "Let".to_string()
    }
}
impl Trace for OpDropLocal {
    fn fmt() -> String {
        "DropLocal".to_string()
    }
}

impl<T: Trace, E: Trace> Trace for OpIf<T, E> {
    fn fmt() -> String {
        "If(...)".to_string()
    }
}

impl<C, B> Trace for OpWhile<C, B> {
    fn fmt() -> String {
        "While(...)".to_string()
    }
}

impl<T> Trace for OpCall<T> {
    fn fmt() -> String {
        "Call".to_string()
    }
}

impl<I: Trace> Trace for OpGetLocal<I> {
    fn fmt() -> String {
        format!("GetLocal({})", I::fmt())
    }
}

impl<I: Trace> Trace for OpSetLocal<I> {
    fn fmt() -> String {
        format!("SetLocal({})", I::fmt())
    }
}
