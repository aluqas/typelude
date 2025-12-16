//! **Trace Trait**
//!
//! A trait for converting type-level structures into runtime strings for debugging and tracing.

use typenum::{B0, B1, Integer, NInt, PInt, UInt, UTerm, Unsigned, Z0};

use crate::std::array::{Cons, TyArray, TyNil};

/// A trait for types that can be traced at runtime.
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

impl Trace for TyNil {
    fn fmt() -> String {
        "[]".to_string()
    }
}

// Helper to handle list formatting internally
trait TraceList {
    fn fmt_list() -> String;
}

impl TraceList for TyNil {
    fn fmt_list() -> String {
        "".to_string()
    }
}

impl<Head, Tail> TraceList for TyArray<Head, Tail>
where
    Head: Trace,
    Tail: TraceList + Cons,
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

impl<Head, Tail> Trace for TyArray<Head, Tail>
where
    Head: Trace,
    Tail: TraceList + Cons,
{
    fn fmt() -> String {
        format!("[{}]", <Self as TraceList>::fmt_list())
    }
}
