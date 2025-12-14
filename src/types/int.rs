//! **Type-Level Integer Integration**
//!
//! `typenum` クレートとの統合を提供します。
//! `typenum` の数値を直接 `Evaluable` として扱えるようにし、
//! `typelude` の評価システム内で算術演算や比較を行えるようにします。

use std::ops::{Add, Div, Mul, Rem, Sub};

use typenum::{B0, B1, Cmp, Equal, Greater, Less, NInt, PInt, Pow, UInt, UTerm, Unsigned, Z0};

use crate::{
    eval::{EApply2, Evaluable, Evaluator, Sealed},
    types::bool::{TyFalse, TyTrue},
};

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

/// 足し算: A + B
pub struct FAdd;
impl Sealed for FAdd {}

/// 引き算: A - B
pub struct FSub;
impl Sealed for FSub {}

/// 掛け算: A * B
pub struct FMul;
impl Sealed for FMul {}

/// 割り算: A / B
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
// Comparison Functions
// =============================================================================

/// 等価比較: A == B -> TyBool
pub struct FEq;
impl Sealed for FEq {}

/// 不等価比較: A != B -> TyBool
pub struct FNeq;
impl Sealed for FNeq {}

/// 小なり: A < B -> TyBool
pub struct FLt;
impl Sealed for FLt {}

/// 小なりイコール: A <= B -> TyBool
pub struct FLe;
impl Sealed for FLe {}

/// 大なり: A > B -> TyBool
pub struct FGt;
impl Sealed for FGt {}

/// 大なりイコール: A >= B -> TyBool
pub struct FGe;
impl Sealed for FGe {}

// --- Helper Trait for Cmp to TyBool ---

#[doc(hidden)]
pub trait CmpToTyBool<Ordering> {
    type IsEq;
    type IsNeq;
    type IsLt;
    type IsLe;
    type IsGt;
    type IsGe;
}

impl<T> CmpToTyBool<Equal> for T {
    type IsEq = TyTrue;
    type IsNeq = TyFalse;
    type IsLt = TyFalse;
    type IsLe = TyTrue;
    type IsGt = TyFalse;
    type IsGe = TyTrue;
}

impl<T> CmpToTyBool<Less> for T {
    type IsEq = TyFalse;
    type IsNeq = TyTrue;
    type IsLt = TyTrue;
    type IsLe = TyTrue;
    type IsGt = TyFalse;
    type IsGe = TyFalse;
}

impl<T> CmpToTyBool<Greater> for T {
    type IsEq = TyFalse;
    type IsNeq = TyTrue;
    type IsLt = TyFalse;
    type IsLe = TyFalse;
    type IsGt = TyTrue;
    type IsGe = TyTrue;
}

// --- Implementations ---

// FEq: A == B
impl<A, B> Evaluable for EApply2<FEq, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: Cmp<Evaluator<B>>,
    (): CmpToTyBool<<Evaluator<A> as Cmp<Evaluator<B>>>::Output>,
{
    type Output = <() as CmpToTyBool<<Evaluator<A> as Cmp<Evaluator<B>>>::Output>>::IsEq;
}

// FNeq: A != B
impl<A, B> Evaluable for EApply2<FNeq, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: Cmp<Evaluator<B>>,
    (): CmpToTyBool<<Evaluator<A> as Cmp<Evaluator<B>>>::Output>,
{
    type Output = <() as CmpToTyBool<<Evaluator<A> as Cmp<Evaluator<B>>>::Output>>::IsNeq;
}

// FLt: A < B
impl<A, B> Evaluable for EApply2<FLt, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: Cmp<Evaluator<B>>,
    (): CmpToTyBool<<Evaluator<A> as Cmp<Evaluator<B>>>::Output>,
{
    type Output = <() as CmpToTyBool<<Evaluator<A> as Cmp<Evaluator<B>>>::Output>>::IsLt;
}

// FLe: A <= B
impl<A, B> Evaluable for EApply2<FLe, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: Cmp<Evaluator<B>>,
    (): CmpToTyBool<<Evaluator<A> as Cmp<Evaluator<B>>>::Output>,
{
    type Output = <() as CmpToTyBool<<Evaluator<A> as Cmp<Evaluator<B>>>::Output>>::IsLe;
}

// FGt: A > B
impl<A, B> Evaluable for EApply2<FGt, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: Cmp<Evaluator<B>>,
    (): CmpToTyBool<<Evaluator<A> as Cmp<Evaluator<B>>>::Output>,
{
    type Output = <() as CmpToTyBool<<Evaluator<A> as Cmp<Evaluator<B>>>::Output>>::IsGt;
}

// FGe: A >= B
impl<A, B> Evaluable for EApply2<FGe, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: Cmp<Evaluator<B>>,
    (): CmpToTyBool<<Evaluator<A> as Cmp<Evaluator<B>>>::Output>,
{
    type Output = <() as CmpToTyBool<<Evaluator<A> as Cmp<Evaluator<B>>>::Output>>::IsGe;
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

pub type EEq<A, B> = EApply2<FEq, A, B>;
pub type ENeq<A, B> = EApply2<FNeq, A, B>;
pub type ELt<A, B> = EApply2<FLt, A, B>;
pub type ELe<A, B> = EApply2<FLe, A, B>;
pub type EGt<A, B> = EApply2<FGt, A, B>;
pub type EGe<A, B> = EApply2<FGe, A, B>;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typenum::{N1, N2, P1, P4, P5, U1, U2, U3, U5, U8, U9, U10, U20};

    use super::*;
    use crate::types::bool::{TyFalse, TyTrue};

    #[test]
    fn test_eval_typenum() {
        // typenum がそのまま評価されるか
        assert_type_eq_all!(Evaluator<U1>, U1);
        assert_type_eq_all!(Evaluator<P5>, P5);
        assert_type_eq_all!(Evaluator<N2>, N2);
    }

    #[test]
    fn test_arithmetic() {
        // Add
        assert_type_eq_all!(Evaluator<EAdd<U1, U2>>, U3);
        assert_type_eq_all!(Evaluator<EAdd<P1, N2>>, N1);

        // Sub
        assert_type_eq_all!(Evaluator<ESub<U10, U2>>, U8);
        assert_type_eq_all!(Evaluator<ESub<N1, P1>>, N2);

        // Mul
        assert_type_eq_all!(Evaluator<EMul<U2, U10>>, U20);
        assert_type_eq_all!(Evaluator<EMul<N2, N2>>, P4);

        // Div
        assert_type_eq_all!(Evaluator<EDiv<U10, U2>>, U5);

        // Rem
        assert_type_eq_all!(Evaluator<ERem<U10, U3>>, U1); // 10 % 3 = 1

        // Pow
        assert_type_eq_all!(Evaluator<EPow<U2, U3>>, U8);

        // Composition
        // (1 + 2) * 3 = 9
        assert_type_eq_all!(Evaluator<EMul<EAdd<U1, U2>, U3>>, U9);
    }

    #[test]
    fn test_comparison() {
        // Eq
        assert_type_eq_all!(Evaluator<EEq<U1, U1>>, TyTrue);
        assert_type_eq_all!(Evaluator<EEq<U1, U2>>, TyFalse);

        // Neq
        assert_type_eq_all!(Evaluator<ENeq<U1, U1>>, TyFalse);
        assert_type_eq_all!(Evaluator<ENeq<U1, U2>>, TyTrue);

        // Lt
        assert_type_eq_all!(Evaluator<ELt<U1, U2>>, TyTrue);
        assert_type_eq_all!(Evaluator<ELt<U2, U1>>, TyFalse);

        // Le
        assert_type_eq_all!(Evaluator<ELe<U1, U2>>, TyTrue);
        assert_type_eq_all!(Evaluator<ELe<U1, U1>>, TyTrue);
        assert_type_eq_all!(Evaluator<ELe<U2, U1>>, TyFalse);

        // Gt
        assert_type_eq_all!(Evaluator<EGt<U2, U1>>, TyTrue);
        assert_type_eq_all!(Evaluator<EGt<U1, U2>>, TyFalse);

        // Ge
        assert_type_eq_all!(Evaluator<EGe<U2, U1>>, TyTrue);
        assert_type_eq_all!(Evaluator<EGe<U1, U1>>, TyTrue);
        assert_type_eq_all!(Evaluator<EGe<U1, U2>>, TyFalse);
    }
}
