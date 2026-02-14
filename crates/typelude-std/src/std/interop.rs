//! Core interoperability traits.
//!
//! This module defines explicit normalization points into typelude's core IR.
//! Phase 1 keeps canonical backends:
//! - Numbers: `typenum`
//! - Strings: `tstr` / `Char`
//! - Sequences: `Array` / `Nil`

use tstr::{TStr, TStrArg};
use typenum::{B0, B1, Bit, UInt, UTerm, Unsigned};

use crate::{
    model::{
        col::array::{Array, IsList, Nil},
        prim::{
            bool::{False, True},
            str::Char,
        },
    },
    std::traits::Nat,
};

/// Convert an external boolean representation into the core boolean IR.
pub trait IntoCoreBool {
    type Output;
}

impl IntoCoreBool for True {
    type Output = True;
}

impl IntoCoreBool for False {
    type Output = False;
}

impl IntoCoreBool for B1 {
    type Output = True;
}

impl IntoCoreBool for B0 {
    type Output = False;
}

/// Convert an external natural-number representation into the core numeric IR.
///
/// In Phase 1, typenum is the canonical numeric backend.
pub trait IntoCoreNat {
    type Output: Nat;
}

impl IntoCoreNat for UTerm {
    type Output = UTerm;
}

impl<U: Unsigned, B: Bit> IntoCoreNat for UInt<U, B> {
    type Output = UInt<U, B>;
}

/// Convert an external sequence representation into the core list IR.
///
/// In Phase 1, `Array/Nil` is the canonical sequence backend.
pub trait IntoCoreSeq {
    type Output: IsList;
}

impl IntoCoreSeq for Nil {
    type Output = Nil;
}

impl<Head, Tail: IsList> IntoCoreSeq for Array<Head, Tail> {
    type Output = Array<Head, Tail>;
}

/// Convert an external string representation into the core string IR.
///
/// In Phase 1, `tstr::TStr<_>` and `Char<_>` are canonical.
pub trait IntoCoreStr {
    type Output;
}

impl<const C: char> IntoCoreStr for Char<C> {
    type Output = Char<C>;
}

impl<S: TStrArg> IntoCoreStr for TStr<S> {
    type Output = TStr<S>;
}
