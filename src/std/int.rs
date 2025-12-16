//! **Type-Level Integer**
//!
//! Integration with the `typenum` crate and integer arithmetic.

use std::ops::{Add, Div, Mul, Rem, Sub};

use typenum::{B0, B1, NInt, PInt, Pow, UInt, UTerm, Unsigned, Z0};

use crate::eval::{EApply2, Evaluable, Evaluator, Sealed};

//
// Evaluable Implementation for typenum Types
//

// --- Unsigned Integers ---

impl Evaluable for UTerm {
    type Output = Self;
}

impl<U, B> Evaluable for UInt<U, B> {
    type Output = Self;
}

// --- Signed Integers ---

impl Evaluable for Z0 {
    type Output = Self;
}

impl<U: Unsigned + typenum::NonZero> Evaluable for PInt<U> {
    type Output = Self;
}

impl<U: Unsigned + typenum::NonZero> Evaluable for NInt<U> {
    type Output = Self;
}

// --- Bits ---

impl Evaluable for B0 {
    type Output = Self;
}

impl Evaluable for B1 {
    type Output = Self;
}

//
// Arithmetic Functions
//

/// Addition: A + B
pub struct FAdd;
impl Sealed for FAdd {}

/// Subtraction: A - B
pub struct FSub;
impl Sealed for FSub {}

/// Multiplication: A * B
pub struct FMul;
impl Sealed for FMul {}

/// Division: A / B
pub struct FDiv;
impl Sealed for FDiv {}

/// Remainder: A % B
pub struct FRem;
impl Sealed for FRem {}

/// Exponentiation: A ^ B
pub struct FPow;
impl Sealed for FPow {}

// --- Implementations ---

// --- Implementations ---

impl<Lhs, Rhs> Evaluable for EApply2<FAdd, Lhs, Rhs>
where
    Lhs: Evaluable,
    Rhs: Evaluable,
    Evaluator<Lhs>: Add<Evaluator<Rhs>>,
    <Evaluator<Lhs> as Add<Evaluator<Rhs>>>::Output: Evaluable,
{
    type Output = <Evaluator<Lhs> as Add<Evaluator<Rhs>>>::Output;
}

impl<Lhs, Rhs> Evaluable for EApply2<FSub, Lhs, Rhs>
where
    Lhs: Evaluable,
    Rhs: Evaluable,
    Evaluator<Lhs>: Sub<Evaluator<Rhs>>,
    <Evaluator<Lhs> as Sub<Evaluator<Rhs>>>::Output: Evaluable,
{
    type Output = <Evaluator<Lhs> as Sub<Evaluator<Rhs>>>::Output;
}

impl<Lhs, Rhs> Evaluable for EApply2<FMul, Lhs, Rhs>
where
    Lhs: Evaluable,
    Rhs: Evaluable,
    Evaluator<Lhs>: Mul<Evaluator<Rhs>>,
    <Evaluator<Lhs> as Mul<Evaluator<Rhs>>>::Output: Evaluable,
{
    type Output = <Evaluator<Lhs> as Mul<Evaluator<Rhs>>>::Output;
}

impl<Lhs, Rhs> Evaluable for EApply2<FDiv, Lhs, Rhs>
where
    Lhs: Evaluable,
    Rhs: Evaluable,
    Evaluator<Lhs>: Div<Evaluator<Rhs>>,
    <Evaluator<Lhs> as Div<Evaluator<Rhs>>>::Output: Evaluable,
{
    type Output = <Evaluator<Lhs> as Div<Evaluator<Rhs>>>::Output;
}

impl<Lhs, Rhs> Evaluable for EApply2<FRem, Lhs, Rhs>
where
    Lhs: Evaluable,
    Rhs: Evaluable,
    Evaluator<Lhs>: Rem<Evaluator<Rhs>>,
    <Evaluator<Lhs> as Rem<Evaluator<Rhs>>>::Output: Evaluable,
{
    type Output = <Evaluator<Lhs> as Rem<Evaluator<Rhs>>>::Output;
}

impl<Lhs, Rhs> Evaluable for EApply2<FPow, Lhs, Rhs>
where
    Lhs: Evaluable,
    Rhs: Evaluable,
    Evaluator<Lhs>: Pow<Evaluator<Rhs>>,
    <Evaluator<Lhs> as Pow<Evaluator<Rhs>>>::Output: Evaluable,
{
    type Output = <Evaluator<Lhs> as Pow<Evaluator<Rhs>>>::Output;
}

//
// Aliases
//

pub type EAdd<Lhs, Rhs> = EApply2<FAdd, Lhs, Rhs>;
pub type ESub<Lhs, Rhs> = EApply2<FSub, Lhs, Rhs>;
pub type EMul<Lhs, Rhs> = EApply2<FMul, Lhs, Rhs>;
pub type EDiv<Lhs, Rhs> = EApply2<FDiv, Lhs, Rhs>;
pub type ERem<Lhs, Rhs> = EApply2<FRem, Lhs, Rhs>;
pub type EPow<Lhs, Rhs> = EApply2<FPow, Lhs, Rhs>;

//
// Tests
//

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typenum::{N1, N2, P1, P4, P5, U1, U2, U3, U5, U8, U9, U10, U20};

    use super::*;

    #[test]
    fn test_eval_typenum() {
        assert_type_eq_all!(Evaluator<U1>, U1);
        assert_type_eq_all!(Evaluator<P5>, P5);
        assert_type_eq_all!(Evaluator<N2>, N2);
        assert_type_eq_all!(Evaluator<B0>, B0);
        assert_type_eq_all!(Evaluator<B1>, B1);
    }

    #[test]
    fn test_add() {
        assert_type_eq_all!(Evaluator<EAdd<U1, U2>>, U3);
        assert_type_eq_all!(Evaluator<EAdd<P1, N2>>, N1);
    }

    #[test]
    fn test_sub() {
        assert_type_eq_all!(Evaluator<ESub<U10, U2>>, U8);
        assert_type_eq_all!(Evaluator<ESub<N1, P1>>, N2);
    }

    #[test]
    fn test_mul() {
        assert_type_eq_all!(Evaluator<EMul<U2, U10>>, U20);
        assert_type_eq_all!(Evaluator<EMul<N2, N2>>, P4);
    }

    #[test]
    fn test_div() {
        assert_type_eq_all!(Evaluator<EDiv<U10, U2>>, U5);
    }

    #[test]
    fn test_rem() {
        assert_type_eq_all!(Evaluator<ERem<U10, U3>>, U1);
    }

    #[test]
    fn test_pow() {
        assert_type_eq_all!(Evaluator<EPow<U2, U3>>, U8);
    }

    #[test]
    fn test_composition() {
        // (1 + 2) * 3 = 9
        assert_type_eq_all!(Evaluator<EMul<EAdd<U1, U2>, U3>>, U9);
    }
}
