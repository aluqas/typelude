//! **Type-Level Array**
//!
//! 型レベルの配列（コンスリスト）とその操作を提供します。

use std::{
    marker::PhantomData,
    ops::{Add, Sub},
};

use typenum::{B1, Sub1, U0, UInt, Unsigned};

use crate::{
    eval::{EApply, EApply2, EApply3, Evaluable, Evaluator, Sealed},
    std::bool::{TyFalse, TyTrue},
};

// =============================================================================
// Type-Level Array Types
// =============================================================================

/// **Marker Trait**
///
/// TyArrayのTailを示す。
pub trait Cons: Sealed {}

/// TyArrayの終端。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct TyNil;
impl Sealed for TyNil {}
impl Cons for TyNil {}

/// 型レベルの配列（コンスセル）
///
/// - `TyHead`: 任意の型 (`u32`, `String`, `TyArray<T, TyNil>` など)
/// - `Tail`: 残りの部分 (`TyArray` で再帰、`TyNil` が終端)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct TyArray<TyHead, Tail: Cons>(pub PhantomData<(TyHead, Tail)>);
impl<Ty, Tail: Cons> Sealed for TyArray<Ty, Tail> {}
impl<Ty, Tail: Cons> Cons for TyArray<Ty, Tail> {}

/// 型リストを簡単に生成するためのマクロ
#[macro_export]
macro_rules! tyarray {
    // 空のリスト
    () => { $crate::std::array::TyNil };
    // 長さ1のリスト (with optional trailing comma)
    ($n:ty $(,)?) => { $crate::std::array::TyArray<$n, $crate::std::array::TyNil> };
    // 長さ2以上のリスト (with optional trailing comma)
    ($n:ty, $($tail:ty),+ $(,)?) => { $crate::std::array::TyArray<$n, $crate::tyarray![$($tail),+]> };
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

// NOTE: Contains depends on Equality check.
use crate::std::cmp::IsEq;

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
    H: IsEq<X>,
{
    const VALUE: bool = <H as IsEq<X>>::EQ || <T as Contains<X>>::VALUE;
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
// Function Markers: Array Operations
// =============================================================================

/// 配列の長さ
pub struct FLen;
/// 配列の先頭要素
pub struct FHead;
/// 配列の先頭以外
pub struct FTail;
/// 配列が空かどうか
pub struct FIsEmpty;
/// 配列のインデックスアクセス
pub struct FGet;
/// 配列のインデックス更新
pub struct FSet;
/// 配列の結合
pub struct FConcat;
/// 配列の末尾に追加
pub struct FAppend;
/// 配列の先頭に追加
pub struct FPrepend;
/// 配列に要素が含まれるか
pub struct FContains;

impl Sealed for FLen {}
impl Sealed for FHead {}
impl Sealed for FTail {}
impl Sealed for FIsEmpty {}
impl Sealed for FGet {}
impl Sealed for FSet {}
impl Sealed for FConcat {}
impl Sealed for FAppend {}
impl Sealed for FPrepend {}
impl Sealed for FContains {}


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
impl<Array, Index, Value> Evaluable for EApply3<FSet, Array, Index, Value>
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
    (): crate::std::bool::Bool2TyBool<{ <Evaluator<Array> as Contains<Evaluator<X>>>::VALUE }>,
{
    type Output = Evaluator<crate::std::bool::Assert<{ <Evaluator<Array> as Contains<Evaluator<X>>>::VALUE }>>;
}

// --- FMap: 配列の各要素に関数を適用 ---
pub struct FMap;
impl Sealed for FMap {}

// Helper trait
pub trait MapHelper<F> {
    type Output: Cons;
}

impl<F> MapHelper<F> for TyNil {
    type Output = TyNil;
}

impl<F, H, T> MapHelper<F> for TyArray<H, T>
where
    F: crate::std::traits::EFunction<H>,
    T: Cons + MapHelper<F>,
    <T as MapHelper<F>>::Output: Cons,
{
    type Output = TyArray<<F as crate::std::traits::EFunction<H>>::Output, <T as MapHelper<F>>::Output>;
}

// Evaluable impl
impl<F, List> Evaluable for EApply2<FMap, F, List>
where
    List: Evaluable,
    Evaluator<List>: MapHelper<F>,
{
    type Output = <Evaluator<List> as MapHelper<F>>::Output;
}

// --- FFilter: 条件に一致する要素のみを残す ---
pub struct FFilter;
impl Sealed for FFilter {}

pub trait FilterHelper<Pred> {
    type Output: Cons;
}

impl<P> FilterHelper<P> for TyNil {
    type Output = TyNil;
}

impl<P, H, T> FilterHelper<P> for TyArray<H, T>
where
    P: crate::std::traits::EFunction<H>,
    T: Cons + FilterHelper<P>,
    <T as FilterHelper<P>>::Output: Cons,
    // Check Predicate
    <P as crate::std::traits::EFunction<H>>::Output: Evaluable,
    // EIf<Pred(H), Cons<H, Filter(T)>, Filter(T)>
    crate::eval::EIf<
        <P as crate::std::traits::EFunction<H>>::Output,
        crate::eval::ELit<TyArray<H, <T as FilterHelper<P>>::Output>>,
        crate::eval::ELit<<T as FilterHelper<P>>::Output>
    >: Evaluable,
    Evaluator<
        crate::eval::EIf<
            <P as crate::std::traits::EFunction<H>>::Output,
            crate::eval::ELit<TyArray<H, <T as FilterHelper<P>>::Output>>,
            crate::eval::ELit<<T as FilterHelper<P>>::Output>
        >
    >: Cons,
{
    type Output = Evaluator<
        crate::eval::EIf<
            <P as crate::std::traits::EFunction<H>>::Output,
            crate::eval::ELit<TyArray<H, <T as FilterHelper<P>>::Output>>,
            crate::eval::ELit<<T as FilterHelper<P>>::Output>
        >
    >;
}

impl<Pred, List> Evaluable for EApply2<FFilter, Pred, List>
where
    List: Evaluable,
    Evaluator<List>: FilterHelper<Pred>,
{
    type Output = <Evaluator<List> as FilterHelper<Pred>>::Output;
}

// --- FFold: リストの畳み込み (Left Fold) ---
pub struct FFold;
impl Sealed for FFold {}

pub trait FoldHelper<F, Acc> {
    type Output;
}

impl<F, Acc> FoldHelper<F, Acc> for TyNil {
    type Output = Acc;
}

impl<F, Acc, H, T> FoldHelper<F, Acc> for TyArray<H, T>
where
    F: crate::std::traits::EFunction<(Acc, H)>,
    T: Cons + FoldHelper<F, <F as crate::std::traits::EFunction<(Acc, H)>>::Output>,
{
    type Output = <T as FoldHelper<F, <F as crate::std::traits::EFunction<(Acc, H)>>::Output>>::Output;
}

impl<F, Init, List> Evaluable for EApply3<FFold, F, Init, List>
where
    Init: Evaluable,
    List: Evaluable,
    Evaluator<List>: FoldHelper<F, Evaluator<Init>>,
{
    type Output = <Evaluator<List> as FoldHelper<F, Evaluator<Init>>>::Output;
}

// =============================================================================
// Aliases
// =============================================================================

// Array (1 arg)
pub type ELen<A> = EApply<FLen, A>;
pub type EHead<A> = EApply<FHead, A>;
pub type ETail<A> = EApply<FTail, A>;
pub type EIsEmpty<A> = EApply<FIsEmpty, A>;

// Array (2 args)
pub type EGet<A, I> = EApply2<FGet, A, I>;
pub type EConcat<A, B> = EApply2<FConcat, A, B>;
pub type EAppend<A, E> = EApply2<FAppend, A, E>;
pub type EPrepend<E, A> = EApply2<FPrepend, E, A>;
pub type EContains<A, X> = EApply2<FContains, A, X>;
pub type EMap<F, A> = EApply2<FMap, F, A>;
pub type EFilter<P, A> = EApply2<FFilter, P, A>;

// Array (3 args)
pub type ESet<A, I, V> = EApply3<FSet, A, I, V>;
pub type EFold<F, Init, List> = EApply3<FFold, F, Init, List>;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typenum::{U0, U1, U2, U10, U12};

    use super::*;
    use crate::{
        eval::ELit,
        std::bool::{TyFalse, TyTrue},
    };

    type MyList =
        tyarray![i32, String, bool, f64, char, (), (), (), (), (), (usize, usize), [TyTrue; 100]];
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

    #[test]
    fn test_emap() {
        use crate::std::traits::EFunction;
        use typenum::{Add1, U1, U2, U3, U4};

        struct AddOne;
        impl<T> EFunction<T> for AddOne
        where
            T: std::ops::Add<typenum::B1>,
        {
            type Output = Add1<T>;
        }

        type List = tyarray![U1, U2, U3];
        type Mapped = EMap<AddOne, ELit<List>>;

        assert_type_eq_all!(Evaluator<Mapped>, tyarray![U2, U3, U4]);
    }

    #[test]
    fn test_efilter() {
        use crate::std::traits::EFunction;
        use typenum::{IsLess, U1, U2, U3, U4, U5};
        use crate::std::bool::{ToTyBoolOut, ToTyBool};

        struct LessThan3;
        impl<T> EFunction<T> for LessThan3
        where
            T: IsLess<U3>,
            <T as IsLess<U3>>::Output: ToTyBool,
        {
            type Output = ToTyBoolOut<<T as IsLess<U3>>::Output>;
        }

        type List = tyarray![U1, U5, U2, U4, U3]; // [1, 5, 2, 4, 3]
        // Filter < 3 -> [1, 2]
        type Filtered = EFilter<LessThan3, ELit<List>>;

        assert_type_eq_all!(Evaluator<Filtered>, tyarray![U1, U2]);
    }

    #[test]
    fn test_efold() {
        use crate::std::traits::EFunction;
        use typenum::{U0, U1, U2, U3, U6};

        // Sum: (Acc, Elem) -> Acc + Elem
        struct Sum;
        impl<Acc, Elem> EFunction<(Acc, Elem)> for Sum
        where
            Acc: std::ops::Add<Elem>,
        {
            type Output = <Acc as std::ops::Add<Elem>>::Output;
        }

        type List = tyarray![U1, U2, U3];
        // Fold Sum 0 [1, 2, 3] -> 6
        type Summed = EFold<Sum, ELit<U0>, ELit<List>>;

        assert_type_eq_all!(Evaluator<Summed>, U6);
    }
}
