//! **Type-Level Integer**
//!
//! `typenum` クレートとの統合、および整数演算を提供します。

use std::ops::{Add, Div, Mul, Rem, Sub};

use typenum::{B0, B1, NInt, PInt, Pow, UInt, UTerm, Unsigned, Z0};

use crate::eval::{EApply2, Evaluable, Evaluator, Sealed};

// =============================================================================
// Evaluable Implementation for typenum Types
// =============================================================================

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

// =============================================================================
// Arithmetic Functions
// =============================================================================

/// 加算: A + B
pub struct FAdd;
impl Sealed for FAdd {}

/// 減算: A - B
pub struct FSub;
impl Sealed for FSub {}

/// 乗算: A * B
pub struct FMul;
impl Sealed for FMul {}

/// 除算: A / B
pub struct FDiv;
impl Sealed for FDiv {}

/// 剰余: A % B
pub struct FRem;
impl Sealed for FRem {}

/// べき乗: A ^ B
pub struct FPow;
impl Sealed for FPow {}

// --- Implementations ---

impl<A, B> Evaluable for EApply2<FAdd, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: Add<Evaluator<B>>,
    <Evaluator<A> as Add<Evaluator<B>>>::Output: Evaluable,
{
    type Output = <Evaluator<A> as Add<Evaluator<B>>>::Output;
}

impl<A, B> Evaluable for EApply2<FSub, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: Sub<Evaluator<B>>,
    <Evaluator<A> as Sub<Evaluator<B>>>::Output: Evaluable,
{
    type Output = <Evaluator<A> as Sub<Evaluator<B>>>::Output;
}

impl<A, B> Evaluable for EApply2<FMul, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: Mul<Evaluator<B>>,
    <Evaluator<A> as Mul<Evaluator<B>>>::Output: Evaluable,
{
    type Output = <Evaluator<A> as Mul<Evaluator<B>>>::Output;
}

impl<A, B> Evaluable for EApply2<FDiv, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: Div<Evaluator<B>>,
    <Evaluator<A> as Div<Evaluator<B>>>::Output: Evaluable,
{
    type Output = <Evaluator<A> as Div<Evaluator<B>>>::Output;
}

impl<A, B> Evaluable for EApply2<FRem, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: Rem<Evaluator<B>>,
    <Evaluator<A> as Rem<Evaluator<B>>>::Output: Evaluable,
{
    type Output = <Evaluator<A> as Rem<Evaluator<B>>>::Output;
}

impl<A, B> Evaluable for EApply2<FPow, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: Pow<Evaluator<B>>,
    <Evaluator<A> as Pow<Evaluator<B>>>::Output: Evaluable,
{
    type Output = <Evaluator<A> as Pow<Evaluator<B>>>::Output;
}

// =============================================================================
// Aliases
// =============================================================================

pub type EAdd<A, B> = EApply2<FAdd, A, B>;
pub type ESub<A, B> = EApply2<FSub, A, B>;
pub type EMul<A, B> = EApply2<FMul, A, B>;
pub type EDiv<A, B> = EApply2<FDiv, A, B>;
pub type ERem<A, B> = EApply2<FRem, A, B>;
pub type EPow<A, B> = EApply2<FPow, A, B>;

// =============================================================================
// Tests
// =============================================================================

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
