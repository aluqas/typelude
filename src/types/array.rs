//! **Type-Level Array**
//!
//! 型レベルの配列（コンスリスト）とその操作を提供します。

use std::marker::PhantomData;
use std::ops::{Add, Sub};
use typenum::{B1, Sub1, U0, UInt, Unsigned};

use crate::eval::{EApply, EApply2, Evaluable, Evaluator, Sealed};
use crate::func::{FAppend, FConcat, FContains, FGet, FHead, FIsEmpty, FLen, FPrepend, FSet, FTail};
use crate::types::bool::{TyFalse, TyTrue};
use crate::types::eq::{_TypeEqConst, AssertBool};

// =============================================================================
// Type-Level Array Types
// =============================================================================

/// **Marker Trait**
///
/// TyArrayのTailを示す。
pub trait Cons: Sealed {}

/// TyArrayの終端。
#[derive(Debug, Clone, Copy)]
pub struct TyNil;
impl Sealed for TyNil {}
impl Cons for TyNil {}

/// 型レベルの配列（コンスセル）
///
/// - `TyHead`: 任意の型 (`u32`, `String`, `TyArray<T, TyNil>` など)
/// - `Tail`: 残りの部分 (`TyArray` で再帰、`TyNil` が終端)
#[derive(Debug, Clone, Copy)]
pub struct TyArray<TyHead, Tail: Cons>(pub PhantomData<(TyHead, Tail)>);
impl<Ty, Tail: Cons> Sealed for TyArray<Ty, Tail> {}
impl<Ty, Tail: Cons> Cons for TyArray<Ty, Tail> {}

/// 型リストを簡単に生成するためのマクロ
#[macro_export]
macro_rules! tyarray {
    // 空のリスト
    () => { $crate::types::array::TyNil };
    // 長さ1のリスト (with optional trailing comma)
    ($n:ty $(,)?) => { $crate::types::array::TyArray<$n, $crate::types::array::TyNil> };
    // 長さ2以上のリスト (with optional trailing comma)
    ($n:ty, $($tail:ty),+ $(,)?) => { $crate::types::array::TyArray<$n, $crate::tyarray![$($tail),+]> };
}

// =============================================================================
// Helper Traits (Internal)
// =============================================================================

/// 配列の長さ
#[doc(hidden)]
pub trait Len {
    type Output: Unsigned;
}

impl Len for TyNil {
    type Output = U0;
}

impl<H, T: Cons + Len> Len for TyArray<H, T>
where
    <T as Len>::Output: Add<B1>,
    <<T as Len>::Output as Add<B1>>::Output: Unsigned,
{
    type Output = <<T as Len>::Output as Add<B1>>::Output;
}

/// 配列の先頭要素
#[doc(hidden)]
pub trait Head {
    type Output;
}

impl<H, T: Cons> Head for TyArray<H, T> {
    type Output = H;
}

/// 配列の先頭以外
#[doc(hidden)]
pub trait Tail {
    type Output: Cons;
}

impl<H, T: Cons> Tail for TyArray<H, T> {
    type Output = T;
}

/// 配列が空かどうか
#[doc(hidden)]
pub trait IsEmpty {
    type Output;
}

impl IsEmpty for TyNil {
    type Output = TyTrue;
}

impl<H, T: Cons> IsEmpty for TyArray<H, T> {
    type Output = TyFalse;
}

/// インデックスアクセス
#[doc(hidden)]
pub trait Get<IDX: Unsigned> {
    type Output;
}

impl<H, T: Cons> Get<U0> for TyArray<H, T> {
    type Output = H;
}

impl<H, T: Cons, N: Unsigned, B: typenum::Bit> Get<UInt<N, B>> for TyArray<H, T>
where
    UInt<N, B>: Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    T: Get<Sub1<UInt<N, B>>>,
{
    type Output = <T as Get<Sub1<UInt<N, B>>>>::Output;
}

/// インデックス更新 (Set)
#[doc(hidden)]
pub trait Set<IDX: Unsigned, VAL> {
    type Output: Cons;
}

impl<H, T: Cons, VAL> Set<U0, VAL> for TyArray<H, T> {
    type Output = TyArray<VAL, T>;
}

impl<H, T: Cons, N: Unsigned, B: typenum::Bit, VAL> Set<UInt<N, B>, VAL> for TyArray<H, T>
where
    UInt<N, B>: Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    T: Set<Sub1<UInt<N, B>>, VAL>,
{
    type Output = TyArray<H, <T as Set<Sub1<UInt<N, B>>, VAL>>::Output>;
}

/// 配列の結合
#[doc(hidden)]
pub trait Concat<Other: Cons> {
    type Output: Cons;
}

impl<Other: Cons> Concat<Other> for TyNil {
    type Output = Other;
}

impl<H, T: Cons, Other: Cons> Concat<Other> for TyArray<H, T>
where
    T: Concat<Other>,
{
    type Output = TyArray<H, <T as Concat<Other>>::Output>;
}

/// 配列に要素が含まれるか (const版)
#[doc(hidden)]
pub trait Contains<X> {
    const VALUE: bool;
}

impl<X> Contains<X> for TyNil {
    const VALUE: bool = false;
}

impl<H, T: Cons + Contains<X>, X> Contains<X> for TyArray<H, T>
where
    (): _TypeEqConst<H, X>,
{
    const VALUE: bool = <() as _TypeEqConst<H, X>>::ARE_EQUAL || <T as Contains<X>>::VALUE;
}

// =============================================================================
// Evaluable: Self-evaluating for concrete types
// =============================================================================

impl Evaluable for TyNil {
    type Output = TyNil;
}

impl<H, T: Cons> Evaluable for TyArray<H, T> {
    type Output = TyArray<H, T>;
}

// =============================================================================
// Evaluable Implementations for Array Functions
// =============================================================================

// --- FLen: 配列の長さ ---
impl<Array> Evaluable for EApply<FLen, Array>
where
    Array: Evaluable,
    Evaluator<Array>: Len,
{
    type Output = <Evaluator<Array> as Len>::Output;
}

// --- FHead: 先頭要素 ---
impl<Array> Evaluable for EApply<FHead, Array>
where
    Array: Evaluable,
    Evaluator<Array>: Head,
{
    type Output = <Evaluator<Array> as Head>::Output;
}

// --- FTail: 先頭以外 ---
impl<Array> Evaluable for EApply<FTail, Array>
where
    Array: Evaluable,
    Evaluator<Array>: Tail,
{
    type Output = <Evaluator<Array> as Tail>::Output;
}

// --- FIsEmpty: 空かどうか ---
impl<Array> Evaluable for EApply<FIsEmpty, Array>
where
    Array: Evaluable,
    Evaluator<Array>: IsEmpty,
{
    type Output = <Evaluator<Array> as IsEmpty>::Output;
}

// --- FGet: インデックスアクセス ---
impl<Array, Index> Evaluable for EApply2<FGet, Array, Index>
where
    Array: Evaluable,
    Index: Evaluable,
    Evaluator<Index>: Unsigned,
    Evaluator<Array>: Get<Evaluator<Index>>,
{
    type Output = <Evaluator<Array> as Get<Evaluator<Index>>>::Output;
}

// --- FSet: インデックス更新 ---
impl<Array, Index, Value> Evaluable for crate::eval::EApply3<FSet, Array, Index, Value>
where
    Array: Evaluable,
    Index: Evaluable,
    Value: Evaluable,
    Evaluator<Index>: Unsigned,
    Evaluator<Array>: Set<Evaluator<Index>, Evaluator<Value>>,
{
    type Output = <Evaluator<Array> as Set<Evaluator<Index>, Evaluator<Value>>>::Output;
}

// --- FConcat: 配列の結合 ---
impl<A, B> Evaluable for EApply2<FConcat, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<B>: Cons,
    Evaluator<A>: Concat<Evaluator<B>>,
{
    type Output = <Evaluator<A> as Concat<Evaluator<B>>>::Output;
}

// --- FAppend: 末尾に追加 ---
impl<Array, Elem> Evaluable for EApply2<FAppend, Array, Elem>
where
    Array: Evaluable,
    Elem: Evaluable,
    Evaluator<Array>: Concat<TyArray<Evaluator<Elem>, TyNil>>,
{
    type Output = <Evaluator<Array> as Concat<TyArray<Evaluator<Elem>, TyNil>>>::Output;
}

// --- FPrepend: 先頭に追加 ---
impl<Elem, Array> Evaluable for EApply2<FPrepend, Elem, Array>
where
    Elem: Evaluable,
    Array: Evaluable,
    Evaluator<Array>: Cons,
{
    type Output = TyArray<Evaluator<Elem>, Evaluator<Array>>;
}

// --- FContains: 配列に要素が含まれるか ---
impl<Array, X> Evaluable for EApply2<FContains, Array, X>
where
    Array: Evaluable,
    X: Evaluable,
    Evaluator<Array>: Contains<Evaluator<X>>,
    AssertBool<{ <Evaluator<Array> as Contains<Evaluator<X>>>::VALUE }>: Evaluable,
{
    type Output = Evaluator<AssertBool<{ <Evaluator<Array> as Contains<Evaluator<X>>>::VALUE }>>;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::ELit;
    use crate::func::{EAppend, EConcat, EContains, EGet, EHead, EIsEmpty, ELen, EPrepend, ETail};
    use static_assertions::assert_type_eq_all;
    use typenum::{U0, U1, U2, U10, U12};

    type MyList = tyarray![
        i32,
        String,
        bool,
        f64,
        char,
        (),
        (),
        (),
        (),
        (),
        (usize, usize),
        [TyTrue; 100]
    ];
    type MyListExpr = ELit<MyList>;

    #[test]
    fn test_simple_evals() {
        // ELen
        assert_type_eq_all!(Evaluator<ELen<ELit<TyNil>>>, U0);
        assert_type_eq_all!(Evaluator<ELen<MyListExpr>>, U12);

        // EHead/ETail
        type List3Expr = ELit<tyarray![i32, f64, bool]>;
        assert_type_eq_all!(Evaluator<EHead<List3Expr>>, i32);
        assert_type_eq_all!(Evaluator<ETail<List3Expr>>, tyarray![f64, bool]);

        // EIsEmpty
        assert_type_eq_all!(Evaluator<EIsEmpty<ELit<TyNil>>>, TyTrue);
        assert_type_eq_all!(Evaluator<EIsEmpty<ELit<tyarray![i32]>>>, TyFalse);

        // EGet
        assert_type_eq_all!(Evaluator<EGet<MyListExpr, ELit<U0>>>, i32);
        assert_type_eq_all!(Evaluator<EGet<MyListExpr, ELit<U1>>>, String);
        assert_type_eq_all!(Evaluator<EGet<MyListExpr, ELit<U10>>>, (usize, usize));
    }

    #[test]
    fn test_composition() {
        type ListA = tyarray![i32, f64];
        type ListB = tyarray![bool, char];

        // EConcat
        type Concatenated = EConcat<ELit<ListA>, ELit<ListB>>;
        assert_type_eq_all!(Evaluator<Concatenated>, tyarray![i32, f64, bool, char]);

        // ELen<EConcat<...>> - 式のネスト！
        assert_type_eq_all!(Evaluator<ELen<Concatenated>>, typenum::U4);

        // EAppend
        type Appended = EAppend<ELit<ListA>, ELit<bool>>;
        assert_type_eq_all!(Evaluator<Appended>, tyarray![i32, f64, bool]);

        // EPrepend
        type Prepended = EPrepend<ELit<bool>, ELit<ListA>>;
        assert_type_eq_all!(Evaluator<Prepended>, tyarray![bool, i32, f64]);

        // EGet<EConcat<...>> - 式のネスト！
        assert_type_eq_all!(Evaluator<EGet<Concatenated, ELit<U2>>>, bool);
    }

    #[test]
    fn test_econtains() {
        type ListExpr = ELit<tyarray![i32, f64, bool, char]>;

        // 含まれる場合 → TyTrue
        assert_type_eq_all!(Evaluator<EContains<ListExpr, ELit<i32>>>, TyTrue);
        assert_type_eq_all!(Evaluator<EContains<ListExpr, ELit<char>>>, TyTrue);

        // 含まれない場合 → TyFalse
        assert_type_eq_all!(Evaluator<EContains<ListExpr, ELit<String>>>, TyFalse);
        assert_type_eq_all!(Evaluator<EContains<ListExpr, ELit<()>>>, TyFalse);

        // 合成したリストでテスト - 式のネスト！
        type Concatenated = EConcat<ELit<tyarray![i32]>, ELit<tyarray![f64]>>;
        assert_type_eq_all!(Evaluator<EContains<Concatenated, ELit<f64>>>, TyTrue);
        assert_type_eq_all!(Evaluator<EContains<Concatenated, ELit<bool>>>, TyFalse);
    }
}
