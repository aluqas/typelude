use paste::paste;

use crate::machine::instruction::*;
use typelude_core::std::trace::Trace;

macro_rules! impl_trace_simple {
    ( $($name:ident),* ) => {
        paste! {
            $(
                impl Trace for [<Op $name>] {
                    fn fmt() -> String {
                        stringify!($name).to_string()
                    }
                }
            )*
        }
    };
}

impl<Val: Trace> Trace for OpPush<Val> {
    fn fmt() -> String {
        format!("Push({})", Val::fmt())
    }
}

impl_trace_simple!(
    Add, Sub, Dup, Swap, Drop, Eq, Neq, Lt, Gt, Not, And, Or, Load, Store, Return, Let, DropLocal
);

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
