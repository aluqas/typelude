//! **Type-Level Array**
//!
//! Type-level array (cons list) and its operations.

use std::{
    marker::PhantomData,
    ops::{Add, Sub},
};

use typelude_core::{EApp, ELit, Eval, Evaluate};
use typenum::{B1, Sub1, U0, UInt, Unsigned};

pub use crate::model::col::array::IsList;
// Re-export kernel types for convenience/compatibility if mostly used from here
pub use crate::model::col::array::{Array, Nil};
/// Re-export List trait for public use
pub use crate::std::traits::List;
use crate::{
    expr::EIf,
    model::prim::bool::{False, True},
};
impl List for Nil {
    type Cons<NewHead> = Array<NewHead, Nil>;
    // Head/Tail for Nil are usually undefined or Unit/Nil
    type Head = (); // Or a custom Error Type
    type Tail = Nil;
}

impl<Head, Tail: IsList> List for Array<Head, Tail> {
    type Cons<NewHead> = Array<NewHead, Self>;
    type Head = Head;
    type Tail = Tail;
}
/// Array length
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a type with a length",
    label = "Len not implemented",
    note = "ensure `{Self}` implements `Len`"
)]
pub trait Len {
    type Output: Unsigned;
}

impl Len for Nil {
    type Output = U0;
}

impl<Head, Tail: IsList + Len> Len for Array<Head, Tail>
where
    <Tail as Len>::Output: Add<B1>,
    <<Tail as Len>::Output as Add<B1>>::Output: Unsigned,
{
    type Output = <<Tail as Len>::Output as Add<B1>>::Output;
}

/// Head element of array
#[diagnostic::on_unimplemented(
    message = "`{Self}` does not have a Head element",
    label = "Head not implemented",
    note = "ensure `{Self}` is a non-empty list"
)]
pub trait Head {
    type Output;
}

impl<H, T: IsList> Head for Array<H, T> {
    type Output = H;
}

/// Tail of array (everything except head)
#[diagnostic::on_unimplemented(
    message = "`{Self}` does not have a Tail",
    label = "Tail not implemented",
    note = "ensure `{Self}` is a non-empty list"
)]
pub trait Tail {
    type Output: IsList;
}

impl<H, T: IsList> Tail for Array<H, T> {
    type Output = T;
}

/// Check if array is empty
pub trait IsEmpty {
    type Output;
}

impl IsEmpty for Nil {
    type Output = True;
}

impl<Head, Tail: IsList> IsEmpty for Array<Head, Tail> {
    type Output = False;
}

/// Index access
#[diagnostic::on_unimplemented(
    message = "Cannot access index `{Idx}` in `{Self}`",
    label = "index access failed",
    note = "index might be out of bounds or `{Self}` is not a list"
)]
pub trait Get<Idx: Unsigned> {
    type Output;
}

impl<Head, Tail: IsList> Get<U0> for Array<Head, Tail> {
    type Output = Head;
}

impl<Head, Tail: IsList, N: Unsigned, B: typenum::Bit> Get<UInt<N, B>> for Array<Head, Tail>
where
    UInt<N, B>: Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    Tail: Get<Sub1<UInt<N, B>>>,
{
    type Output = <Tail as Get<Sub1<UInt<N, B>>>>::Output;
}

/// Index update (Set)
#[diagnostic::on_unimplemented(
    message = "Cannot set index `{Idx}` in `{Self}` to `{Val}`",
    label = "index update failed",
    note = "index might be out of bounds or `{Self}` is not a list"
)]
pub trait Set<Idx: Unsigned, Val> {
    type Output: IsList;
}

impl<Head, Tail: IsList, Val> Set<U0, Val> for Array<Head, Tail> {
    type Output = Array<Val, Tail>;
}

impl<Head, Tail: IsList, N: Unsigned, B: typenum::Bit, Val> Set<UInt<N, B>, Val>
    for Array<Head, Tail>
where
    UInt<N, B>: Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    Tail: Set<Sub1<UInt<N, B>>, Val>,
{
    type Output = Array<Head, <Tail as Set<Sub1<UInt<N, B>>, Val>>::Output>;
}

/// Array concatenation
pub trait Concat<Other: IsList> {
    type Output: IsList;
}

impl<Other: IsList> Concat<Other> for Nil {
    type Output = Other;
}

impl<Head, Tail: IsList, Other: IsList> Concat<Other> for Array<Head, Tail>
where
    Tail: Concat<Other>,
{
    type Output = Array<Head, <Tail as Concat<Other>>::Output>;
}

// NOTE: Contains depends on Equality check.
use crate::std::ops::IsEq;

/// Check if array contains element (const version)
pub trait Contains<Elem> {
    const VALUE: bool;
}

impl<Elem> Contains<Elem> for Nil {
    const VALUE: bool = false;
}

impl<Head, Tail: IsList + Contains<Elem>, Elem> Contains<Elem> for Array<Head, Tail>
where
    Head: IsEq<Elem>,
{
    const VALUE: bool = <Head as IsEq<Elem>>::EQ || <Tail as Contains<Elem>>::VALUE;
}
use typelude_macros::def_op;

// --- Basic Array Operations ---

def_op! {
    /// Get the length of an array
    name: DefLen,
    args: (Arr),
    ast: ELen {
        where: [
            Evaluate<Arr>: Len
        ],
        type Output = <Evaluate<Arr> as Len>::Output
    }
}

def_op! {
    /// Get the head (first element) of an array
    name: DefHead,
    args: (Arr),
    ast: EHead {
        where: [
            Evaluate<Arr>: List
        ],
        type Output = <Evaluate<Arr> as List>::Head
    }
}

def_op! {
    /// Get the tail (all but first) of an array
    name: DefTail,
    args: (Arr),
    ast: ETail {
        where: [
            Evaluate<Arr>: List,
            <Evaluate<Arr> as List>::Tail: Eval
        ],
        type Output = <Evaluate<Arr> as List>::Tail
    }
}

def_op! {
    /// Check if an array is empty
    name: DefIsEmpty,
    args: (Arr),
    ast: EIsEmpty {
        where: [
            Evaluate<Arr>: IsEmpty
        ],
        type Output = <Evaluate<Arr> as IsEmpty>::Output
    }
}

// --- Index Operations ---

def_op! {
    /// Get element at index
    name: DefGet,
    args: (Arr, Idx),
    ast: EGet {
        where: [
            Evaluate<Idx>: Unsigned,
            Evaluate<Arr>: Get<Evaluate<Idx>>
        ],
        type Output = <Evaluate<Arr> as Get<Evaluate<Idx>>>::Output
    }
}

def_op! {
    /// Set element at index
    name: DefSet,
    args: (Arr, Idx, Val),
    ast: ESet {
        where: [
            Evaluate<Idx>: Unsigned,
            Evaluate<Arr>: Set<Evaluate<Idx>, Evaluate<Val>>
        ],
        type Output = <Evaluate<Arr> as Set<Evaluate<Idx>, Evaluate<Val>>>::Output
    }
}

// --- Concatenation Operations ---

def_op! {
    /// Concatenate two arrays
    name: DefConcat,
    args: (Lhs, Rhs),
    ast: EConcat {
        where: [
            Evaluate<Rhs>: IsList,
            Evaluate<Lhs>: Concat<Evaluate<Rhs>>
        ],
        type Output = <Evaluate<Lhs> as Concat<Evaluate<Rhs>>>::Output
    }
}

def_op! {
    /// Append element to end of array
    name: DefAppend,
    args: (Arr, Elem),
    ast: EAppend {
        where: [
            Evaluate<Arr>: Concat<Array<Evaluate<Elem>, Nil>>
        ],
        type Output = <Evaluate<Arr> as Concat<Array<Evaluate<Elem>, Nil>>>::Output
    }
}

def_op! {
    /// Prepend element to start of array
    name: DefPrepend,
    args: (Elem, Arr),
    ast: EPrepend {
        where: [
            Evaluate<Arr>: List
        ],
        type Output = <Evaluate<Arr> as List>::Cons<Evaluate<Elem>>
    }
}

// --- Higher-Order Operations (defined separately due to complex bounds) ---

// --- EMap ---
pub struct EMap<Op, List>(PhantomData<(Op, List)>);

impl<Op, List> Eval for EMap<Op, List>
where
    List: Eval,
    Evaluate<List>: MapHelper<Op>,
{
    type Output = <Evaluate<List> as MapHelper<Op>>::Output;
}

// --- EFilter ---
pub struct EFilter<Pred, List>(PhantomData<(Pred, List)>);

impl<Pred, List> Eval for EFilter<Pred, List>
where
    List: Eval,
    Evaluate<List>: FilterHelper<Pred>,
{
    type Output = <Evaluate<List> as FilterHelper<Pred>>::Output;
}

// --- EFold ---
pub struct EFold<Op, Init, List>(PhantomData<(Op, Init, List)>);

impl<Op, Init, List> Eval for EFold<Op, Init, List>
where
    Init: Eval,
    List: Eval,
    Evaluate<List>: FoldHelper<Op, Evaluate<Init>>,
{
    type Output = <Evaluate<List> as FoldHelper<Op, Evaluate<Init>>>::Output;
}

/// Expression to check if an array contains an element.
///
/// # Examples
///
/// ```ignore
/// type Res = Evaluate<EContains<ELit<tyarray![U1, U2]>, ELit<U1>>>; // True
/// ```
#[cfg(feature = "nightly")]
pub struct EContains<Array, Elem>(PhantomData<(Array, Elem)>);

#[cfg(feature = "nightly")]
impl<Array, Elem> Eval for EContains<Array, Elem>
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

// --- Higher-Order Operations ---
/// Helper for Map
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "Internal `MapHelper` not implemented for `{Self}`",
    label = "Map not implemented",
    note = "ensure `{Self}` is a List and `Op` is a valid function"
)]
pub trait MapHelper<Op> {
    type Output: IsList;
}

impl<Op> MapHelper<Op> for Nil {
    type Output = Nil;
}

impl<Op, Head, Tail> MapHelper<Op> for Array<Head, Tail>
where
    EApp<ELit<Op>, ELit<Head>>: Eval,
    Evaluate<EApp<ELit<Op>, ELit<Head>>>: Eval,
    Tail: IsList + MapHelper<Op>,
    <Tail as MapHelper<Op>>::Output: IsList,
{
    // Evaluate application result for strict map
    type Output =
        Array<Evaluate<Evaluate<EApp<ELit<Op>, ELit<Head>>>>, <Tail as MapHelper<Op>>::Output>;
}

/// Helper for Filter
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "Internal `FilterHelper` not implemented for `{Self}`",
    label = "Filter not implemented",
    note = "ensure `{Self}` is a List and `Pred` is a valid function"
)]
pub trait FilterHelper<Pred> {
    type Output: IsList;
}

impl<P> FilterHelper<P> for Nil {
    type Output = Nil;
}

impl<P, Head, Tail> FilterHelper<P> for Array<Head, Tail>
where
    EApp<ELit<P>, ELit<Head>>: Eval,
    Tail: IsList + FilterHelper<P>,
    <Tail as FilterHelper<P>>::Output: IsList,
    // EIf<Pred(Head), Array<Head, Filter(Tail)>, Filter(Tail)>
    EIf<
        Evaluate<EApp<ELit<P>, ELit<Head>>>, // Predicate Result Expr
        ELit<Array<Head, <Tail as FilterHelper<P>>::Output>>,
        ELit<<Tail as FilterHelper<P>>::Output>,
    >: Eval,
    Evaluate<
        EIf<
            Evaluate<EApp<ELit<P>, ELit<Head>>>,
            ELit<Array<Head, <Tail as FilterHelper<P>>::Output>>,
            ELit<<Tail as FilterHelper<P>>::Output>,
        >,
    >: IsList,
{
    type Output = Evaluate<
        EIf<
            Evaluate<EApp<ELit<P>, ELit<Head>>>,
            ELit<Array<Head, <Tail as FilterHelper<P>>::Output>>,
            ELit<<Tail as FilterHelper<P>>::Output>,
        >,
    >;
}

/// Helper for Fold
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "Internal `FoldHelper` not implemented for `{Self}`",
    label = "Fold not implemented",
    note = "ensure `{Self}` is a Cons list and `Op` is a valid function"
)]
pub trait FoldHelper<Op, Acc> {
    type Output;
}

impl<Op, Acc> FoldHelper<Op, Acc> for Nil {
    type Output = Acc;
}

impl<Op, Acc, Head, Tail> FoldHelper<Op, Acc> for Array<Head, Tail>
where
    EApp<ELit<Op>, ELit<typelude_core::ECons<Acc, typelude_core::ECons<Head, typelude_core::ENil>>>>:
        Eval,
    Evaluate<
        EApp<
            ELit<Op>,
            ELit<typelude_core::ECons<Acc, typelude_core::ECons<Head, typelude_core::ENil>>>,
        >,
    >: Eval,
    Tail: IsList
        + FoldHelper<
            Op,
            Evaluate<
                Evaluate<
                    EApp<
                        ELit<Op>,
                        ELit<typelude_core::ECons<Acc, typelude_core::ECons<Head, typelude_core::ENil>>>,
                    >,
                >,
            >,
        >,
{
    // Strict Fold
    type Output = <Tail as FoldHelper<
        Op,
        Evaluate<
            Evaluate<
                EApp<
                    ELit<Op>,
                    ELit<typelude_core::ECons<Acc, typelude_core::ECons<Head, typelude_core::ENil>>>,
                >,
            >,
        >,
    >>::Output;
}
use crate::std::prim::option::{None, Some};

// --- Reverse ---

/// Reverse trait: reverses the order of elements
pub trait Reverse {
    type Output;
}

impl Reverse for Nil {
    type Output = Nil;
}

impl<Head, Tail: IsList> Reverse for Array<Head, Tail>
where
    Tail: Reverse,
    <Tail as Reverse>::Output: Concat<Array<Head, Nil>>,
{
    type Output = <<Tail as Reverse>::Output as Concat<Array<Head, Nil>>>::Output;
}

// --- Take<N> ---

/// Take trait: take first N elements
pub trait Take<N> {
    type Output;
}

impl<N> Take<N> for Nil {
    type Output = Nil;
}

impl<Head, Tail: IsList> Take<U0> for Array<Head, Tail> {
    type Output = Nil;
}

impl<Head, Tail: IsList, N: Unsigned, B: typenum::Bit> Take<UInt<N, B>> for Array<Head, Tail>
where
    UInt<N, B>: Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    Tail: Take<Sub1<UInt<N, B>>>,
    <Tail as Take<Sub1<UInt<N, B>>>>::Output: IsList,
{
    type Output = Array<Head, <Tail as Take<Sub1<UInt<N, B>>>>::Output>;
}

// --- Drop<N> ---

/// Drop trait: skip first N elements
pub trait Drop<N> {
    type Output;
}

impl<N> Drop<N> for Nil {
    type Output = Nil;
}

impl<Head, Tail: IsList> Drop<U0> for Array<Head, Tail> {
    type Output = Array<Head, Tail>;
}

impl<Head, Tail: IsList, N: Unsigned, B: typenum::Bit> Drop<UInt<N, B>> for Array<Head, Tail>
where
    UInt<N, B>: Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    Tail: Drop<Sub1<UInt<N, B>>>,
{
    type Output = <Tail as Drop<Sub1<UInt<N, B>>>>::Output;
}

// --- Last ---

/// Last trait: get the last element
#[diagnostic::on_unimplemented(
    message = "`{Self}` does not have a Last element",
    label = "Last not implemented",
    note = "ensure `{Self}` is a non-empty list"
)]
pub trait Last {
    type Output;
}

impl<T> Last for Array<T, Nil> {
    type Output = T;
}

impl<Head, T, Rest: IsList> Last for Array<Head, Array<T, Rest>>
where
    Array<T, Rest>: Last,
{
    type Output = <Array<T, Rest> as Last>::Output;
}

// --- Zip ---

/// Zip trait: combine two lists pairwise
pub trait Zip<Other> {
    type Output;
}

impl<Other> Zip<Other> for Nil {
    type Output = Nil;
}

impl<H, T: IsList> Zip<Nil> for Array<H, T> {
    type Output = Nil;
}

impl<H1, T1: IsList, H2, T2: IsList> Zip<Array<H2, T2>> for Array<H1, T1>
where
    T1: Zip<T2>,
    <T1 as Zip<T2>>::Output: IsList,
{
    type Output = Array<(H1, H2), <T1 as Zip<T2>>::Output>;
}

// --- Find<Pred> ---

/// Find trait: find first element matching predicate, returns Some/None
pub trait Find<Pred> {
    type Output; // Some<T> or None
}

impl<Pred> Find<Pred> for Nil {
    type Output = None;
}

/// Helper for Find dispatch
#[doc(hidden)]
pub trait FindHelper<Pred, Head, Tail> {
    type Output;
}

impl<Pred, Head, Tail: IsList> FindHelper<Pred, Head, Tail> for True {
    type Output = Some<Head>;
}

impl<Pred, Head, Tail: IsList + Find<Pred>> FindHelper<Pred, Head, Tail> for False {
    type Output = <Tail as Find<Pred>>::Output;
}

impl<Pred, Head, Tail: IsList> Find<Pred> for Array<Head, Tail>
where
    EApp<ELit<Pred>, ELit<Head>>: Eval,
    Evaluate<EApp<ELit<Pred>, ELit<Head>>>: Eval,
    Evaluate<Evaluate<EApp<ELit<Pred>, ELit<Head>>>>: FindHelper<Pred, Head, Tail>,
{
    type Output =
        <Evaluate<Evaluate<EApp<ELit<Pred>, ELit<Head>>>> as FindHelper<Pred, Head, Tail>>::Output;
}

// --- Any<Pred> ---

/// Any trait: true if any element satisfies predicate
pub trait Any<Pred> {
    type Output; // True or False
}

impl<Pred> Any<Pred> for Nil {
    type Output = False;
}

/// Helper for Any dispatch
#[doc(hidden)]
pub trait AnyHelper<Pred, Tail> {
    type Output;
}

impl<Pred, Tail: IsList> AnyHelper<Pred, Tail> for True {
    type Output = True;
}

impl<Pred, Tail: IsList + Any<Pred>> AnyHelper<Pred, Tail> for False {
    type Output = <Tail as Any<Pred>>::Output;
}

impl<Pred, Head, Tail: IsList> Any<Pred> for Array<Head, Tail>
where
    EApp<ELit<Pred>, ELit<Head>>: Eval,
    Evaluate<EApp<ELit<Pred>, ELit<Head>>>: Eval,
    Evaluate<Evaluate<EApp<ELit<Pred>, ELit<Head>>>>: AnyHelper<Pred, Tail>,
{
    type Output =
        <Evaluate<Evaluate<EApp<ELit<Pred>, ELit<Head>>>> as AnyHelper<Pred, Tail>>::Output;
}

// --- All<Pred> ---

/// All trait: true if all elements satisfy predicate
pub trait All<Pred> {
    type Output; // True or False
}

impl<Pred> All<Pred> for Nil {
    type Output = True;
}

/// Helper for All dispatch
#[doc(hidden)]
pub trait AllHelper<Pred, Tail> {
    type Output;
}

impl<Pred, Tail: IsList + All<Pred>> AllHelper<Pred, Tail> for True {
    type Output = <Tail as All<Pred>>::Output;
}

impl<Pred, Tail: IsList> AllHelper<Pred, Tail> for False {
    type Output = False;
}

impl<Pred, Head, Tail: IsList> All<Pred> for Array<Head, Tail>
where
    EApp<ELit<Pred>, ELit<Head>>: Eval,
    Evaluate<EApp<ELit<Pred>, ELit<Head>>>: Eval,
    Evaluate<Evaluate<EApp<ELit<Pred>, ELit<Head>>>>: AllHelper<Pred, Tail>,
{
    type Output =
        <Evaluate<Evaluate<EApp<ELit<Pred>, ELit<Head>>>> as AllHelper<Pred, Tail>>::Output;
}

// --- RFC-0001 Phase 1: Expression definitions via def_op! ---

def_op! {
    /// Reverse a list
    name: DefReverse,
    args: (List),
    ast: EReverse {
        where: [
            Evaluate<List>: Reverse
        ],
        type Output = <Evaluate<List> as Reverse>::Output
    }
}

def_op! {
    /// Take first N elements from a list
    name: DefTake,
    args: (N, List),
    ast: ETake {
        where: [
            Evaluate<List>: Take<Evaluate<N>>
        ],
        type Output = <Evaluate<List> as Take<Evaluate<N>>>::Output
    }
}

def_op! {
    /// Drop first N elements from a list
    name: DefDrop,
    args: (N, List),
    ast: EDrop {
        where: [
            Evaluate<List>: Drop<Evaluate<N>>
        ],
        type Output = <Evaluate<List> as Drop<Evaluate<N>>>::Output
    }
}

def_op! {
    /// Get the last element of a list
    name: DefLast,
    args: (List),
    ast: ELast {
        where: [
            Evaluate<List>: Last
        ],
        type Output = <Evaluate<List> as Last>::Output
    }
}

def_op! {
    /// Zip two lists together pairwise
    name: DefZip,
    args: (L1, L2),
    ast: EZip {
        where: [
            Evaluate<L1>: Zip<Evaluate<L2>>
        ],
        type Output = <Evaluate<L1> as Zip<Evaluate<L2>>>::Output
    }
}

def_op! {
    /// Find first element matching predicate
    name: DefFind,
    args: (Pred, List),
    ast: EFind {
        where: [
            Evaluate<List>: Find<Evaluate<Pred>>
        ],
        type Output = <Evaluate<List> as Find<Evaluate<Pred>>>::Output
    }
}

def_op! {
    /// True if any element matches predicate
    name: DefAny,
    args: (Pred, List),
    ast: EAny {
        where: [
            Evaluate<List>: Any<Evaluate<Pred>>
        ],
        type Output = <Evaluate<List> as Any<Evaluate<Pred>>>::Output
    }
}

def_op! {
    /// True if all elements match predicate
    name: DefAll,
    args: (Pred, List),
    ast: EAll {
        where: [
            Evaluate<List>: All<Evaluate<Pred>>
        ],
        type Output = <Evaluate<List> as All<Evaluate<Pred>>>::Output
    }
}
#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_core::ELit;
    use typenum::{U0, U1, U2, U4, U10, U12};

    use super::*;
    use typelude_core::Apply;
    use crate::{
        std::{
            ops::From,
            prim::bool::{False, ToBoolOut, True},
        },
        tyarray,
    };

    type MyList =
        tyarray![i32, String, bool, f64, char, (), (), (), (), (), (usize, usize), [True; 100]];
    type MyListExpr = ELit<MyList>;

    #[test]
    fn test_simple_evals() {
        // ELen
        assert_type_eq_all!(Evaluate<ELen<ELit<Nil>>>, U0);
        assert_type_eq_all!(Evaluate<ELen<MyListExpr>>, U12);

        // EHead/ETail
        type List3Expr = ELit<tyarray![i32, f64, bool]>;
        assert_type_eq_all!(Evaluate<EHead<List3Expr>>, i32);
        assert_type_eq_all!(Evaluate<ETail<List3Expr>>, tyarray![f64, bool]);

        // EIsEmpty
        assert_type_eq_all!(Evaluate<EIsEmpty<ELit<Nil>>>, True);
        assert_type_eq_all!(Evaluate<EIsEmpty<ELit<tyarray![i32]>>>, False);

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

        // ELen<EConcat<...>>
        assert_type_eq_all!(Evaluate<ELen<Concatenated>>, U4);

        // EAppend
        type Appended = EAppend<ELit<ListA>, ELit<bool>>;
        assert_type_eq_all!(Evaluate<Appended>, tyarray![i32, f64, bool]);

        // EPrepend
        type Prepended = EPrepend<ELit<bool>, ELit<ListA>>;
        assert_type_eq_all!(Evaluate<Prepended>, tyarray![bool, i32, f64]);

        // EGet<EConcat<...>>
        assert_type_eq_all!(Evaluate<EGet<Concatenated, ELit<U2>>>, bool);
    }

    #[test]
    #[cfg(feature = "nightly")]
    fn test_econtains() {
        type ListExpr = ELit<tyarray![i32, f64, bool, char]>;

        // 含まれる場合 → True
        assert_type_eq_all!(Evaluate<EContains<ListExpr, ELit<i32>>>, True);
        assert_type_eq_all!(Evaluate<EContains<ListExpr, ELit<char>>>, True);

        // 含まれない場合 → False
        assert_type_eq_all!(Evaluate<EContains<ListExpr, ELit<String>>>, False);
        assert_type_eq_all!(Evaluate<EContains<ListExpr, ELit<()>>>, False);

        // 合成したリストでテスト - 式のネスト！
        type Concatenated = EConcat<ELit<tyarray![i32]>, ELit<tyarray![f64]>>;
        assert_type_eq_all!(Evaluate<EContains<Concatenated, ELit<f64>>>, True);
        assert_type_eq_all!(Evaluate<EContains<Concatenated, ELit<bool>>>, False);
    }

    #[test]
    fn test_emap() {
        use typenum::{Add1, U1, U2, U3, U4};

        struct FnAddOne;
        impl<T> Apply<T> for FnAddOne
        where
            T: std::ops::Add<typenum::B1>,
        {
            type Output = ELit<Add1<T>>;
        }

        type List = tyarray![U1, U2, U3];
        type Mapped = EMap<FnAddOne, ELit<List>>;

        assert_type_eq_all!(Evaluate<Mapped>, tyarray![U2, U3, U4]);
    }

    #[test]
    fn test_efilter() {
        use typenum::{IsLess, U1, U2, U3, U4, U5};

        struct PredLessThan3;
        impl<T> Apply<T> for PredLessThan3
        where
            T: IsLess<U3>,
            bool: From<<T as IsLess<U3>>::Output>,
        {
            type Output = ELit<ToBoolOut<<T as IsLess<U3>>::Output>>;
        }

        type List = tyarray![U1, U5, U2, U4, U3]; // [1, 5, 2, 4, 3]
        // Filter < 3 -> [1, 2]
        type Filtered = EFilter<PredLessThan3, ELit<List>>;

        assert_type_eq_all!(Evaluate<Filtered>, tyarray![U1, U2]);
    }

    #[test]
    fn test_efold() {
        use typenum::{U0, U1, U2, U3, U6};

        // Sum: (Acc, Elem) -> Acc + Elem
        struct FnSum;
        impl<Acc, Elem>
            Apply<typelude_core::ECons<Acc, typelude_core::ECons<Elem, typelude_core::ENil>>>
            for FnSum
        where
            Acc: std::ops::Add<Elem>,
        {
            type Output = ELit<<Acc as std::ops::Add<Elem>>::Output>;
        }

        type List = tyarray![U1, U2, U3];
        // Fold Sum 0 [1, 2, 3] -> 6
        type Summed = EFold<FnSum, ELit<U0>, ELit<List>>;

        assert_type_eq_all!(Evaluate<Summed>, U6);
    }
    #[test]
    fn test_reverse() {
        use typenum::{U1, U2, U3};

        type List = tyarray![U1, U2, U3];
        type Reversed = <List as Reverse>::Output;
        assert_type_eq_all!(Reversed, tyarray![U3, U2, U1]);

        // Empty list
        type EmptyReversed = <Nil as Reverse>::Output;
        assert_type_eq_all!(EmptyReversed, Nil);
    }

    #[test]
    fn test_take_drop() {
        use typenum::{U1, U2, U3, U4};

        type List = tyarray![U1, U2, U3, U4];

        // Take<2> -> [1, 2]
        type Taken = <List as Take<U2>>::Output;
        assert_type_eq_all!(Taken, tyarray![U1, U2]);

        // Drop<2> -> [3, 4]
        type Dropped = <List as Drop<U2>>::Output;
        assert_type_eq_all!(Dropped, tyarray![U3, U4]);

        // Take<0> -> []
        type TakeZero = <List as Take<U0>>::Output;
        assert_type_eq_all!(TakeZero, Nil);

        // Drop<0> -> [1, 2, 3, 4]
        type DropZero = <List as Drop<U0>>::Output;
        assert_type_eq_all!(DropZero, List);
    }

    #[test]
    fn test_last() {
        use typenum::{U1, U2, U3};

        type List = tyarray![U1, U2, U3];
        type LastElem = <List as Last>::Output;
        assert_type_eq_all!(LastElem, U3);

        // Single element
        type Single = tyarray![U1];
        type SingleLast = <Single as Last>::Output;
        assert_type_eq_all!(SingleLast, U1);
    }

    #[test]
    fn test_zip() {
        use typenum::{U1, U2, U3};

        type ListA = tyarray![U1, U2, U3];
        type ListB = tyarray![i32, f64, bool];
        type Zipped = <ListA as Zip<ListB>>::Output;

        assert_type_eq_all!(Zipped, tyarray![(U1, i32), (U2, f64), (U3, bool)]);

        // Different lengths - shorter wins
        type Short = tyarray![U1];
        type ZipShort = <ListA as Zip<Short>>::Output;
        assert_type_eq_all!(ZipShort, tyarray![(U1, U1)]);
    }
}
