//! ペアノ数による型レベル自然数実装。
//!
//! Zero と Succ<T> による標準的なペアノ数表現。
//! 再帰的な型構造により、コンパイル時に数値計算と比較検証を実施できる。
//!
//! ## 主要操作
//!
//! - Add<Rhs>: 加算（ペアノ帰納法の再帰実装）
//! - Mul<Rhs>: 乗算（加算の反復）
//! - Sub<Rhs>: 減算（下限ゼロ）
//! - CanSub<Rhs>: 減算可能判定（ヘルパートレイト）

use core::marker::PhantomData;

use typelude_std::core::{Add, Div, Mul, Sub, Value};

/// 後続者（successor）型。
///
/// 汎用型パラメータ `T` の後続者を型レベルで表現。
/// Succ<Zero> = 1, Succ<Succ<Zero>> = 2, ... の使い方で任意の自然数を構成。
pub struct Succ<T>(PhantomData<T>);
/// ペアノ数ゼロ型。
///
/// 数値ゼロを型レベルで表現。一般的なペアノ数の基底ケース。
pub struct Zero;

impl<T> Value for Succ<T> {}
impl Value for Zero {}

/// ペアノ数トレイト。
///
/// 型レベル自然数（Zero と Succ による再帰評価）を表現し、
/// 定数値関連付け `VAL` により、実行時に数値へ変換可能。
/// あらゆるペアノ数は Nat を実装していることが保証される。
pub trait Nat {
    /// 型レベル自然数に対応する実行時定数値。
    const VAL: usize;
}

impl Nat for Zero {
    const VAL: usize = 0;
}

impl<T: Nat> Nat for Succ<T> {
    const VAL: usize = T::VAL + 1;
}

impl<Rhs: Nat> Add<Rhs> for Zero {
    type Output = Rhs;
}

impl<Lhs: Nat, Rhs: Nat> Add<Rhs> for Succ<Lhs>
where
    Lhs: Add<Rhs>,
{
    type Output = Succ<<Lhs as Add<Rhs>>::Output>;
}

impl<Rhs: Nat> Mul<Rhs> for Zero {
    type Output = Zero;
}

impl<Lhs: Nat, Rhs: Nat> Mul<Rhs> for Succ<Lhs>
where
    Lhs: Mul<Rhs>,
    Rhs: Add<<Lhs as Mul<Rhs>>::Output>,
{
    type Output = <Rhs as Add<<Lhs as Mul<Rhs>>::Output>>::Output;
}

/// Hyper Operatorトレイト。
///
/// hyper(a, n, b) =
/// if n = 0 then b + 1
/// if n = 1, b = 0 then a
/// if n = 2, b = 0 then 0
/// if n >= 3, b = 0 then 1
/// otherwise hyper(hyper(a, n - 1, b), n, b - 1)
trait Hyper<n: Nat, Rhs: Nat> {
    type Output;
}

// hyper(a, 0, b) = b + 1
impl<Lhs: Nat, Rhs: Nat> Hyper<Zero, Rhs> for Lhs {
    type Output = Succ<Lhs>;
}

// hyper(a, 1, 0) = a
impl<Lhs: Nat, Rhs: Nat> Hyper<Succ<Zero>, Zero> for Lhs {
    type Output = Lhs;
}

// hyper(a, 2, 0) = 0
impl<Lhs: Nat, Rhs: Nat> Hyper<Succ<Succ<Zero>>, Zero> for Lhs {
    type Output = Zero;
}

// hyper(a, n, 0) = 1 (n > 1)
impl<n: Nat, Lhs: Nat, Rhs: Nat> Hyper<Succ<Succ<n>>, Zero> for Lhs {
    type Output = Succ<Zero>;
}

// hyper(a, n, b) = hyper(hyper(a, n - 1, b), n, b - 1) (n > 0, b > 0)
impl<n: Nat, Lhs: Nat, Rhs: Nat> Hyper<Succ<Zero>, Succ<Rhs>> for Lhs
where
    Lhs: Hyper<Zero, Succ<Rhs>>,
    <Lhs as Hyper<Zero, Succ<Rhs>>>::Output: Hyper<Succ<Zero>, Rhs>,
{
    type Output = <<Lhs as Hyper<Zero, Succ<Rhs>>>::Output as Hyper<Succ<Zero>, Rhs>>::Output;
}

type Tet<Lhs, Rhs> = <Lhs as Hyper<Succ<Succ<Succ<Succ<Zero>>>>, Rhs>>::Output;

// struct HyperOp;
// impl Apply<OpHyper, (Lhs, n, Rhs)> for HyperOp

// TODO: Implementation Graham's number lol

impl<Rhs: Nat> Sub<Rhs> for Zero {
    type Output = Zero;
}

impl<Lhs: Nat> Sub<Zero> for Succ<Lhs> {
    type Output = Succ<Lhs>;
}

impl<Lhs: Nat, Rhs: Nat> Sub<Succ<Rhs>> for Succ<Lhs>
where
    Lhs: Sub<Rhs>,
{
    type Output = <Lhs as Sub<Rhs>>::Output;
}

#[doc(hidden)]
pub struct CanSubYes;
#[doc(hidden)]
pub struct CanSubNo;

#[doc(hidden)]
pub trait CanSub<Rhs> {
    type Output;
}

impl CanSub<Zero> for Zero {
    type Output = CanSubYes;
}

impl<Rhs: Nat> CanSub<Succ<Rhs>> for Zero {
    type Output = CanSubNo;
}

impl<Lhs: Nat> CanSub<Zero> for Succ<Lhs> {
    type Output = CanSubYes;
}

impl<Lhs: Nat, Rhs: Nat> CanSub<Succ<Rhs>> for Succ<Lhs>
where
    Lhs: CanSub<Rhs>,
{
    type Output = <Lhs as CanSub<Rhs>>::Output;
}

#[doc(hidden)]
pub trait DivStep<Divisor, Flag> {
    type Output;
}

impl<Dividend: Nat, Divisor: Nat> DivStep<Divisor, CanSubNo> for Dividend {
    type Output = Zero;
}

impl<Dividend: Nat, Divisor: Nat> DivStep<Divisor, CanSubYes> for Dividend
where
    Dividend: Sub<Succ<Divisor>>,
    <Dividend as Sub<Succ<Divisor>>>::Output: Div<Succ<Divisor>>,
{
    type Output = Succ<<<Dividend as Sub<Succ<Divisor>>>::Output as Div<Succ<Divisor>>>::Output>;
}

impl<Divisor: Nat> Div<Succ<Divisor>> for Zero {
    type Output = Zero;
}

impl<Dividend: Nat, Divisor: Nat> Div<Succ<Divisor>> for Succ<Dividend>
where
    Succ<Dividend>: CanSub<Succ<Divisor>>,
    Succ<Dividend>: DivStep<Divisor, <Succ<Dividend> as CanSub<Succ<Divisor>>>::Output>,
{
    type Output = <Succ<Dividend> as DivStep<
        Divisor,
        <Succ<Dividend> as CanSub<Succ<Divisor>>>::Output,
    >>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_std::core::{Apply, Evaluate, OpAdd, OpDiv, OpMul, OpSub};

    use super::*;

    type N0 = Zero;
    type N1 = Succ<Zero>;
    type N2 = Succ<Succ<Zero>>;
    type N3 = Succ<Succ<Succ<Zero>>>;
    type N4 = Succ<Succ<Succ<Succ<Zero>>>>;
    type N5 = Succ<N4>;
    type N6 = Succ<N5>;

    #[test]
    fn test_add() {
        assert_eq!(<N2 as Add<N0>>::Output::VAL, 2);
        assert_eq!(<N2 as Add<N3>>::Output::VAL, 5);
        assert_type_eq_all!(Evaluate<Apply<OpAdd, (N2, N3)>>, N5);
    }

    #[test]
    fn test_mul() {
        assert_eq!(<N2 as Mul<N0>>::Output::VAL, 0);
        assert_eq!(<N2 as Mul<N3>>::Output::VAL, 6);
        assert_type_eq_all!(Evaluate<Apply<OpMul, (N2, N3)>>, N6);
    }

    #[test]
    fn test_sub() {
        assert_eq!(<N3 as Sub<N0>>::Output::VAL, 3);
        assert_eq!(<N3 as Sub<N1>>::Output::VAL, 2);
        assert_eq!(<N3 as Sub<N3>>::Output::VAL, 0);
        assert_eq!(<N3 as Sub<N4>>::Output::VAL, 0);
        assert_type_eq_all!(Evaluate<Apply<OpSub, (N3, N1)>>, N2);
    }

    #[test]
    fn test_div() {
        assert_eq!(<N6 as Div<N2>>::Output::VAL, 3);
        assert_eq!(<N5 as Div<N2>>::Output::VAL, 2);
        assert_type_eq_all!(Evaluate<Apply<OpDiv, (N6, N2)>>, N3);
    }
}
