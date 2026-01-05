//! **Trace Trait**
//!
//! A trait for converting type-level structures into runtime strings for
//! debugging and tracing.

use typenum::{B0, B1, Integer, NInt, PInt, UInt, UTerm, Unsigned, Z0};

use crate::{
    data::col::array::IsList,
    std::col::array::{Array, Nil},
};

/// A trait for types that can be traced at runtime.
#[diagnostic::on_unimplemented(
    message = "`{Self}` does not implement Trace",
    label = "Trace not implemented",
    note = "implement `Trace` for `{Self}` to enable runtime debugging string generation"
)]
pub trait Trace {
    fn fmt() -> String;
}

impl Trace for UTerm {
    fn fmt() -> String {
        "0".to_string()
    }
}

impl<U, B> Trace for UInt<U, B>
where
    UInt<U, B>: Unsigned,
{
    fn fmt() -> String {
        format!("{}", <UInt<U, B> as Unsigned>::to_u64())
    }
}

impl Trace for Z0 {
    fn fmt() -> String {
        "0".to_string()
    }
}

impl<U> Trace for PInt<U>
where
    U: Unsigned + typenum::NonZero,
{
    fn fmt() -> String {
        format!("{}", <PInt<U> as Integer>::to_i64())
    }
}

impl<U> Trace for NInt<U>
where
    U: Unsigned + typenum::NonZero,
{
    fn fmt() -> String {
        format!("{}", <NInt<U> as Integer>::to_i64())
    }
}

impl Trace for B0 {
    fn fmt() -> String {
        "false".to_string()
    }
}

impl Trace for B1 {
    fn fmt() -> String {
        "true".to_string()
    }
}

use crate::std::prim::bool::{False, True};

impl Trace for True {
    fn fmt() -> String {
        "true".to_string()
    }
}

impl Trace for False {
    fn fmt() -> String {
        "false".to_string()
    }
}

impl Trace for Nil {
    fn fmt() -> String {
        "[]".to_string()
    }
}

// Helper to handle list formatting internally
trait TraceList {
    fn fmt_list() -> String;
}

impl TraceList for Nil {
    fn fmt_list() -> String {
        "".to_string()
    }
}

impl<Head, Tail> TraceList for Array<Head, Tail>
where
    Head: Trace,
    Tail: TraceList + IsList,
{
    fn fmt_list() -> String {
        let head = Head::fmt();
        let tail = Tail::fmt_list();
        if tail.is_empty() {
            head
        } else {
            format!("{}, {}", head, tail)
        }
    }
}

impl<Head, Tail> Trace for Array<Head, Tail>
where
    Head: Trace,
    Tail: TraceList + IsList,
{
    fn fmt() -> String {
        format!("[{}]", <Self as TraceList>::fmt_list())
    }
}

impl<T: Trace> Trace for typelude_core::ELit<T> {
    fn fmt() -> String {
        T::fmt()
    }
}

// Trace implementations for std::ops operators
use crate::std::ops::{OpAdd, OpAnd, OpGt, OpLt, OpNot, OpOr, OpSub};

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

impl Trace for OpNot {
    fn fmt() -> String {
        "Not".to_string()
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
