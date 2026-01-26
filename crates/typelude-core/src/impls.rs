use typenum::{B0, B1, NInt, PInt, UInt, UTerm, Z0};

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
impl<U, B> Eval for UInt<U, B> {
    type Output = Self;
}

impl<U> Eval for PInt<U> {
    type Output = Self;
}

impl<U> Eval for NInt<U> {
    type Output = Self;
}
