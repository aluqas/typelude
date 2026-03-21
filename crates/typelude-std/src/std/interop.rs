//! Core interoperability traits.
//!
//! This module defines explicit normalization points into typelude's core IR.
//! Phase 1 keeps canonical backends:
//! - Numbers: `typenum`
//! - Strings: `Array<Char, ...>`
//! - Sequences: `Array` / `Nil`

use typenum::{B0, B1, Bit, UInt, UTerm, Unsigned};

use crate::{
    model::{
        col::array::{Array, IsList, Nil},
        prim::{
            bool::{False, True},
            str::Char,
        },
    },
    std::{
        prim::str::IsCharArray,
        traits::{Nat, ToArray},
    },
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
#[deprecated(since = "0.1.0", note = "Use `std::traits::ToArray` instead")]
pub trait IntoCoreSeq {
    type Output: IsList;
}

impl<T> IntoCoreSeq for T
where
    T: ToArray,
{
    type Output = <T as ToArray>::Output;
}

/// Convert an external string representation into the core string IR.
///
/// In Phase 1, `Array<Char, ...>` is canonical.
#[deprecated(since = "0.1.0", note = "Use `std::traits::ToChars` instead")]
pub trait IntoCoreStr {
    type Output: IsCharArray;
}

impl<const C: char> IntoCoreStr for Char<C> {
    type Output = Array<Char<C>, Nil>;
}

impl IntoCoreStr for Nil {
    type Output = Nil;
}

impl<const C: char, Tail: IsCharArray> IntoCoreStr for Array<Char<C>, Tail> {
    type Output = Array<Char<C>, Tail>;
}
