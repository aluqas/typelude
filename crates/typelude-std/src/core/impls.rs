use typenum::{B0, B1, Bit, NInt, NonZero, PInt, UInt, UTerm, Unsigned, Z0};

use crate::Eval;

// Base Terminals
impl Eval for UTerm {
    type Output = Self;
}

impl Eval for Z0 {
    type Output = Self;
}

impl Eval for B0 {
    type Output = Self;
}

impl Eval for B1 {
    type Output = Self;
}

// Recursive Structures
impl<U: Unsigned, B: Bit> Eval for UInt<U, B> {
    type Output = Self;
}

impl<U: Unsigned + NonZero> Eval for PInt<U> {
    type Output = Self;
}

impl<U: Unsigned + NonZero> Eval for NInt<U> {
    type Output = Self;
}
