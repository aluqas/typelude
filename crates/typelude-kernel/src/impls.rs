//! **Typenum Integration**
//!
//! Implements `Eval` for `typenum` types.

use typenum::{B0, B1, NInt, PInt, UInt, UTerm, Z0};

use crate::Eval;

// --- Unsigned Integers ---

impl Eval for UTerm {
    type Output = Self;
}
impl<U, B> Eval for UInt<U, B> {
    type Output = Self;
}

// --- Signed Integers ---

impl Eval for Z0 {
    type Output = Self;
}
impl<U: typenum::Unsigned + typenum::NonZero> Eval for PInt<U> {
    type Output = Self;
}
impl<U: typenum::Unsigned + typenum::NonZero> Eval for NInt<U> {
    type Output = Self;
}

// --- Bits ---

impl Eval for B0 {
    type Output = Self;
}
impl Eval for B1 {
    type Output = Self;
}
