//! **Type-Level Array**
//!
//! Type-level array (cons list) and its operations.

use std::{
    marker::PhantomData,
    ops::{Add, Sub},
};

use typenum::{B1, Sub1, U0, UInt, Unsigned};

use crate::{
    eval::{EApply, EApply2, EApply3, Eval, Evaluate, Sealed},
    std::bool::{TyFalse, TyTrue},
};

// =============================================================================
// Layer 1: Values (Data Structure)
// =============================================================================

/// **Marker Trait**
///
/// Represents the Tail of TyArray.
pub trait Cons: Sealed {}

/// Termination of TyArray.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct TyNil;
impl Sealed for TyNil {}
impl Cons for TyNil {}

/// Type-level array (Cons Cell)
///
/// - `Head`: Any type (`u32`, `String`, `TyArray<T, TyNil>`, etc.)
/// - `Tail`: Rest part (recursive `TyArray`, `TyNil` is termination)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct TyArray<Head, Tail: Cons>(pub PhantomData<(Head, Tail)>);
impl<Head, Tail: Cons> Sealed for TyArray<Head, Tail> {}
impl<Head, Tail: Cons> Cons for TyArray<Head, Tail> {}

/// Macro for easily creating type lists
#[macro_export]
macro_rules! tyarray {
    // Empty list
    () => { $crate::std::array::TyNil };
    // List with length 1 (with optional trailing comma)
    ($n:ty $(,)?) => { $crate::std::array::TyArray<$n, $crate::std::array::TyNil> };
    // List with length 2 or more (with optional trailing comma)
    ($n:ty, $($tail:ty),+ $(,)?) => { $crate::std::array::TyArray<$n, $crate::tyarray![$($tail),+]> };
}

// =============================================================================
// Layer 2: Capabilities (Verbs)
// =============================================================================

/// Array length
pub trait Len {
    type Output: Unsigned;
}

impl Len for TyNil {
    type Output = U0;
}

impl<Head, Tail: Cons + Len> Len for TyArray<Head, Tail>
where
    <Tail as Len>::Output: Add<B1>,
    <<Tail as Len>::Output as Add<B1>>::Output: Unsigned,
{
    type Output = <<Tail as Len>::Output as Add<B1>>::Output;
}

/// Head element of array
pub trait Head {
    type Output;
}

impl<H, T: Cons> Head for TyArray<H, T> {
    type Output = H;
}

/// Tail of array (everything except head)
pub trait Tail {
    type Output: Cons;
}

impl<H, T: Cons> Tail for TyArray<H, T> {
    type Output = T;
}

/// Check if array is empty
pub trait IsEmpty {
    type Output;
}

impl IsEmpty for TyNil {
    type Output = TyTrue;
}

impl<Head, Tail: Cons> IsEmpty for TyArray<Head, Tail> {
    type Output = TyFalse;
}

/// Index access
pub trait Get<Idx: Unsigned> {
    type Output;
}

impl<Head, Tail: Cons> Get<U0> for TyArray<Head, Tail> {
    type Output = Head;
}

impl<Head, Tail: Cons, N: Unsigned, B: typenum::Bit> Get<UInt<N, B>> for TyArray<Head, Tail>
where
    UInt<N, B>: Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    Tail: Get<Sub1<UInt<N, B>>>,
{
    type Output = <Tail as Get<Sub1<UInt<N, B>>>>::Output;
}

/// Index update (Set)
pub trait Set<Idx: Unsigned, Val> {
    type Output: Cons;
}

impl<Head, Tail: Cons, Val> Set<U0, Val> for TyArray<Head, Tail> {
    type Output = TyArray<Val, Tail>;
}

impl<Head, Tail: Cons, N: Unsigned, B: typenum::Bit, Val> Set<UInt<N, B>, Val>
    for TyArray<Head, Tail>
where
    UInt<N, B>: Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    Tail: Set<Sub1<UInt<N, B>>, Val>,
{
    type Output = TyArray<Head, <Tail as Set<Sub1<UInt<N, B>>, Val>>::Output>;
}

/// Array concatenation
pub trait Concat<Other: Cons> {
    type Output: Cons;
}

impl<Other: Cons> Concat<Other> for TyNil {
    type Output = Other;
}

impl<Head, Tail: Cons, Other: Cons> Concat<Other> for TyArray<Head, Tail>
where
    Tail: Concat<Other>,
{
    type Output = TyArray<Head, <Tail as Concat<Other>>::Output>;
}

// NOTE: Contains depends on Equality check.
use crate::std::cmp::IsEq;

/// Check if array contains element (const version)
pub trait Contains<Elem> {
    const VALUE: bool;
}

impl<Elem> Contains<Elem> for TyNil {
    const VALUE: bool = false;
}

impl<Head, Tail: Cons + Contains<Elem>, Elem> Contains<Elem> for TyArray<Head, Tail>
where
    Head: IsEq<Elem>,
{
    const VALUE: bool = <Head as IsEq<Elem>>::EQ || <Tail as Contains<Elem>>::VALUE;
}

// =============================================================================
// Layer 3: OpCodes (Instruction Markers)
// =============================================================================

/// Array length
pub struct FLen;
/// Head element
pub struct FHead;
/// Tail
pub struct FTail;
/// Is empty
pub struct FIsEmpty;
/// Index get
pub struct FGet;
/// Index set
pub struct FSet;
/// Concatenation
pub struct FConcat;
/// Append to end
pub struct FAppend;
/// Prepend to start
pub struct FPrepend;
/// Contains element
pub struct FContains;
/// Map: Apply function to each element
pub struct FMap;
/// Filter: Keep only elements matching condition
pub struct FFilter;
/// Fold: Left Fold
pub struct FFold;

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
impl Sealed for FMap {}
impl Sealed for FFilter {}
impl Sealed for FFold {}

// =============================================================================
// Layer 4: Backends (Helpers)
// =============================================================================

/// Helper for Map
#[doc(hidden)]
pub trait MapHelper<F> {
    type Output: Cons;
}

impl<F> MapHelper<F> for TyNil {
    type Output = TyNil;
}

impl<F, Head, Tail> MapHelper<F> for TyArray<Head, Tail>
where
    F: crate::std::traits::TyFn<Head>,
    Tail: Cons + MapHelper<F>,
    <Tail as MapHelper<F>>::Output: Cons,
{
    type Output =
        TyArray<<F as crate::std::traits::TyFn<Head>>::Output, <Tail as MapHelper<F>>::Output>;
}

/// Helper for Filter
#[doc(hidden)]
pub trait FilterHelper<Pred> {
    type Output: Cons;
}

impl<P> FilterHelper<P> for TyNil {
    type Output = TyNil;
}

impl<P, Head, Tail> FilterHelper<P> for TyArray<Head, Tail>
where
    P: crate::std::traits::TyFn<Head>,
    Tail: Cons + FilterHelper<P>,
    <Tail as FilterHelper<P>>::Output: Cons,
    // Check Predicate
    <P as crate::std::traits::TyFn<Head>>::Output: Eval,
    // EIf<Pred(Head), Cons<Head, Filter(Tail)>, Filter(Tail)>
    crate::eval::EIf<
        <P as crate::std::traits::TyFn<Head>>::Output,
        crate::eval::ELit<TyArray<Head, <Tail as FilterHelper<P>>::Output>>,
        crate::eval::ELit<<Tail as FilterHelper<P>>::Output>,
    >: Eval,
    Evaluate<
        crate::eval::EIf<
            <P as crate::std::traits::TyFn<Head>>::Output,
            crate::eval::ELit<TyArray<Head, <Tail as FilterHelper<P>>::Output>>,
            crate::eval::ELit<<Tail as FilterHelper<P>>::Output>,
        >,
    >: Cons,
{
    type Output = Evaluate<
        crate::eval::EIf<
            <P as crate::std::traits::TyFn<Head>>::Output,
            crate::eval::ELit<TyArray<Head, <Tail as FilterHelper<P>>::Output>>,
            crate::eval::ELit<<Tail as FilterHelper<P>>::Output>,
        >,
    >;
}

/// Helper for Fold
#[doc(hidden)]
pub trait FoldHelper<F, Acc> {
    type Output;
}

impl<F, Acc> FoldHelper<F, Acc> for TyNil {
    type Output = Acc;
}

impl<F, Acc, Head, Tail> FoldHelper<F, Acc> for TyArray<Head, Tail>
where
    F: crate::std::traits::TyFn<(Acc, Head)>,
    Tail: Cons + FoldHelper<F, <F as crate::std::traits::TyFn<(Acc, Head)>>::Output>,
{
    type Output =
        <Tail as FoldHelper<F, <F as crate::std::traits::TyFn<(Acc, Head)>>::Output>>::Output;
}

// =============================================================================
// Layer 5: Evaluator Integration
// =============================================================================

// --- Self Evaluation for Values ---
impl Eval for TyNil {
    type Output = TyNil;
}

impl<Head, Tail: Cons> Eval for TyArray<Head, Tail> {
    type Output = TyArray<Head, Tail>;
}

// --- FLen: Array length ---
impl<Array> Eval for EApply<FLen, Array>
where
    Array: Eval,
    Evaluate<Array>: Len,
{
    type Output = <Evaluate<Array> as Len>::Output;
}

// --- FHead: Head element ---
impl<Array> Eval for EApply<FHead, Array>
where
    Array: Eval,
    Evaluate<Array>: Head,
{
    type Output = <Evaluate<Array> as Head>::Output;
}

// --- FTail: Tail ---
impl<Array> Eval for EApply<FTail, Array>
where
    Array: Eval,
    Evaluate<Array>: Tail,
{
    type Output = <Evaluate<Array> as Tail>::Output;
}

// --- FIsEmpty: Is empty ---
impl<Array> Eval for EApply<FIsEmpty, Array>
where
    Array: Eval,
    Evaluate<Array>: IsEmpty,
{
    type Output = <Evaluate<Array> as IsEmpty>::Output;
}

// --- FGet: Index get ---
impl<Array, Idx> Eval for EApply2<FGet, Array, Idx>
where
    Array: Eval,
    Idx: Eval,
    Evaluate<Idx>: Unsigned,
    Evaluate<Array>: Get<Evaluate<Idx>>,
{
    type Output = <Evaluate<Array> as Get<Evaluate<Idx>>>::Output;
}

// --- FSet: Index set ---
impl<Array, Idx, Val> Eval for EApply3<FSet, Array, Idx, Val>
where
    Array: Eval,
    Idx: Eval,
    Val: Eval,
    Evaluate<Idx>: Unsigned,
    Evaluate<Array>: Set<Evaluate<Idx>, Evaluate<Val>>,
{
    type Output = <Evaluate<Array> as Set<Evaluate<Idx>, Evaluate<Val>>>::Output;
}

// --- FConcat: Concatenation ---
impl<Lhs, Rhs> Eval for EApply2<FConcat, Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Evaluate<Rhs>: Cons,
    Evaluate<Lhs>: Concat<Evaluate<Rhs>>,
{
    type Output = <Evaluate<Lhs> as Concat<Evaluate<Rhs>>>::Output;
}

// --- FAppend: Append to end ---
impl<Array, Elem> Eval for EApply2<FAppend, Array, Elem>
where
    Array: Eval,
    Elem: Eval,
    Evaluate<Array>: Concat<TyArray<Evaluate<Elem>, TyNil>>,
{
    type Output = <Evaluate<Array> as Concat<TyArray<Evaluate<Elem>, TyNil>>>::Output;
}

// --- FPrepend: Prepend to start ---
impl<Elem, Array> Eval for EApply2<FPrepend, Elem, Array>
where
    Elem: Eval,
    Array: Eval,
    Evaluate<Array>: Cons,
{
    type Output = TyArray<Evaluate<Elem>, Evaluate<Array>>;
}

// --- FContains: Contains element ---
impl<Array, Elem> Eval for EApply2<FContains, Array, Elem>
where
    Array: Eval,
    Elem: Eval,
    Evaluate<Array>: Contains<Evaluate<Elem>>,
    (): crate::std::reify::ReflectBool<{ <Evaluate<Array> as Contains<Evaluate<Elem>>>::VALUE }>,
{
    type Output = Evaluate<
        crate::std::bool::Assert<{ <Evaluate<Array> as Contains<Evaluate<Elem>>>::VALUE }>,
    >;
}

// --- FMap ---
impl<F, List> Eval for EApply2<FMap, F, List>
where
    List: Eval,
    Evaluate<List>: MapHelper<F>,
{
    type Output = <Evaluate<List> as MapHelper<F>>::Output;
}

// --- FFilter ---
impl<Pred, List> Eval for EApply2<FFilter, Pred, List>
where
    List: Eval,
    Evaluate<List>: FilterHelper<Pred>,
{
    type Output = <Evaluate<List> as FilterHelper<Pred>>::Output;
}

// --- FFold ---
impl<F, Init, List> Eval for EApply3<FFold, F, Init, List>
where
    Init: Eval,
    List: Eval,
    Evaluate<List>: FoldHelper<F, Evaluate<Init>>,
{
    type Output = <Evaluate<List> as FoldHelper<F, Evaluate<Init>>>::Output;
}

// =============================================================================
// Layer 6: Expressions (Aliases)
// =============================================================================

// Array (1 arg)
pub type ELen<Array> = EApply<FLen, Array>;
pub type EHead<Array> = EApply<FHead, Array>;
pub type ETail<Array> = EApply<FTail, Array>;
pub type EIsEmpty<Array> = EApply<FIsEmpty, Array>;

// Array (2 args)
pub type EGet<Array, Idx> = EApply2<FGet, Array, Idx>;
pub type EConcat<Lhs, Rhs> = EApply2<FConcat, Lhs, Rhs>;
pub type EAppend<Array, Elem> = EApply2<FAppend, Array, Elem>;
pub type EPrepend<Elem, Array> = EApply2<FPrepend, Elem, Array>;
pub type EContains<Array, Elem> = EApply2<FContains, Array, Elem>;
pub type EMap<F, Array> = EApply2<FMap, F, Array>;
pub type EFilter<Pred, Array> = EApply2<FFilter, Pred, Array>;

// Array (3 args)
pub type ESet<Array, Idx, Val> = EApply3<FSet, Array, Idx, Val>;
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
        assert_type_eq_all!(Evaluate<ELen<ELit<TyNil>>>, U0);
        assert_type_eq_all!(Evaluate<ELen<MyListExpr>>, U12);

        // EHead/ETail
        type List3Expr = ELit<tyarray![i32, f64, bool]>;
        assert_type_eq_all!(Evaluate<EHead<List3Expr>>, i32);
        assert_type_eq_all!(Evaluate<ETail<List3Expr>>, tyarray![f64, bool]);

        // EIsEmpty
        assert_type_eq_all!(Evaluate<EIsEmpty<ELit<TyNil>>>, TyTrue);
        assert_type_eq_all!(Evaluate<EIsEmpty<ELit<tyarray![i32]>>>, TyFalse);

        // EGet
        assert_type_eq_all!(Evaluate<EGet<MyListExpr, ELit<U0>>>, i32);
        assert_type_eq_all!(Evaluate<EGet<MyListExpr, ELit<U1>>>, String);
        assert_type_eq_all!(Evaluate<EGet<MyListExpr, ELit<U10>>>, (usize, usize));
    }

    #[test]
    fn test_composition() {
        type ListA = tyarray![i32, f64];
        type ListB = tyarray![bool, char];

        // EConcat
        type Concatenated = EConcat<ELit<ListA>, ELit<ListB>>;
        assert_type_eq_all!(Evaluate<Concatenated>, tyarray![i32, f64, bool, char]);

        // ELen<EConcat<...>> - 式のネスト！
        assert_type_eq_all!(Evaluate<ELen<Concatenated>>, typenum::U4);

        // EAppend
        type Appended = EAppend<ELit<ListA>, ELit<bool>>;
        assert_type_eq_all!(Evaluate<Appended>, tyarray![i32, f64, bool]);

        // EPrepend
        type Prepended = EPrepend<ELit<bool>, ELit<ListA>>;
        assert_type_eq_all!(Evaluate<Prepended>, tyarray![bool, i32, f64]);

        // EGet<EConcat<...>> - 式のネスト！
        assert_type_eq_all!(Evaluate<EGet<Concatenated, ELit<U2>>>, bool);
    }

    #[test]
    fn test_econtains() {
        type ListExpr = ELit<tyarray![i32, f64, bool, char]>;

        // 含まれる場合 → TyTrue
        assert_type_eq_all!(Evaluate<EContains<ListExpr, ELit<i32>>>, TyTrue);
        assert_type_eq_all!(Evaluate<EContains<ListExpr, ELit<char>>>, TyTrue);

        // 含まれない場合 → TyFalse
        assert_type_eq_all!(Evaluate<EContains<ListExpr, ELit<String>>>, TyFalse);
        assert_type_eq_all!(Evaluate<EContains<ListExpr, ELit<()>>>, TyFalse);

        // 合成したリストでテスト - 式のネスト！
        type Concatenated = EConcat<ELit<tyarray![i32]>, ELit<tyarray![f64]>>;
        assert_type_eq_all!(Evaluate<EContains<Concatenated, ELit<f64>>>, TyTrue);
        assert_type_eq_all!(Evaluate<EContains<Concatenated, ELit<bool>>>, TyFalse);
    }

    #[test]
    fn test_emap() {
        use typenum::{Add1, U1, U2, U3, U4};

        use crate::std::traits::TyFn;

        struct AddOne;
        impl<T> TyFn<T> for AddOne
        where
            T: std::ops::Add<typenum::B1>,
        {
            type Output = Add1<T>;
        }

        type List = tyarray![U1, U2, U3];
        type Mapped = EMap<AddOne, ELit<List>>;

        assert_type_eq_all!(Evaluate<Mapped>, tyarray![U2, U3, U4]);
    }

    #[test]
    fn test_efilter() {
        use typenum::{IsLess, U1, U2, U3, U4, U5};

        use crate::std::{bool::ToTyBoolOut, into::TyFrom, traits::TyFn};

        struct LessThan3;
        impl<T> TyFn<T> for LessThan3
        where
            T: IsLess<U3>,
            // T < 3 returns B1/B0. We want a boolean.
            // TyFrom checks: bool is target.
            bool: TyFrom<<T as IsLess<U3>>::Output>,
        {
            type Output = ToTyBoolOut<<T as IsLess<U3>>::Output>;
        }

        type List = tyarray![U1, U5, U2, U4, U3]; // [1, 5, 2, 4, 3]
        // Filter < 3 -> [1, 2]
        type Filtered = EFilter<LessThan3, ELit<List>>;

        assert_type_eq_all!(Evaluate<Filtered>, tyarray![U1, U2]);
    }

    #[test]
    fn test_efold() {
        use typenum::{U0, U1, U2, U3, U6};

        use crate::std::traits::TyFn;

        // Sum: (Acc, Elem) -> Acc + Elem
        struct Sum;
        impl<Acc, Elem> TyFn<(Acc, Elem)> for Sum
        where
            Acc: std::ops::Add<Elem>,
        {
            type Output = <Acc as std::ops::Add<Elem>>::Output;
        }

        type List = tyarray![U1, U2, U3];
        // Fold Sum 0 [1, 2, 3] -> 6
        type Summed = EFold<Sum, ELit<U0>, ELit<List>>;

        assert_type_eq_all!(Evaluate<Summed>, U6);
    }
}
