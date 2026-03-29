//! **Type-Level Array**
//!
//! Type-level array (cons list) and its operations.

use std::ops::{Add, Sub};

use typelude_std::core::{ECall, ECall2, ELit, Eval, Evaluate};
use typenum::{B1, Sub1, U0, UInt, Unsigned};

pub use crate::model::col::array::IsList;
// Re-export kernel types for convenience/compatibility if mostly used from here
pub use crate::model::col::array::{Array, Nil};
/// Re-export List trait for public use
pub use crate::std::traits::{Foldable, List, ToArray};
use crate::{
    model::prim::bool::{False, True},
    std::control::EIf,
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

impl ToArray for Nil {
    type Output = Nil;
}

impl<Head, Tail: IsList> ToArray for Array<Head, Tail> {
    type Output = Array<Head, Tail>;
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

macro_rules! define_array_unary_fn {
    ($name:ident<$arg:ident> where [$($bound:tt)*] => $($out:tt)+) => {
        crate::typelude_macros::ty_fn! {
            pub struct $name<$arg>
            where [$($bound)*]
            {
                type Output = $($out)+;
            }
        }
    };
}

macro_rules! define_array_binary_fn {
    ($name:ident<$lhs:ident, $rhs:ident> where [$($bound:tt)*] => $($out:tt)+) => {
        crate::paste::paste! {
            crate::typelude_macros::ty_fn! {
                pub struct $name<$lhs>
                {
                    type Output = [<$name Captured>]<$lhs>;
                }
            }

            crate::typelude_macros::ty_fn! {
                pub struct [<$name Captured>]<$lhs, $rhs>
                where [$($bound)*]
                {
                    type Output = $($out)+;
                }
            }
        }
    };
}

macro_rules! define_array_ternary_fn {
    ($name:ident<$a:ident, $b:ident, $c:ident> where [$($bound:tt)*] => $($out:tt)+) => {
        crate::paste::paste! {
            crate::typelude_macros::ty_fn! {
                pub struct $name<$a>
                {
                    type Output = [<$name Captured1>]<$a>;
                }
            }

            crate::typelude_macros::ty_fn! {
                pub struct [<$name Captured1>]<$a, $b>
                {
                    type Output = [<$name Captured2>]<$a, $b>;
                }
            }

            crate::typelude_macros::ty_fn! {
                pub struct [<$name Captured2>]<$a, $b, $c>
                where [$($bound)*]
                {
                    type Output = $($out)+;
                }
            }
        }
    };
}

// NOTE: Contains depends on explicit equality decisions.
use crate::std::ops::EqDecide;

/// Check if array contains element.
pub trait Contains<Elem> {
    type Output;
}

impl<Elem> Contains<Elem> for Nil {
    type Output = False;
}

crate::helper_if! {
    #[doc(hidden)]
    pub trait ContainsHelper<Elem, Tail>;
    on True => True;
    on False where [Tail: IsList + Contains<Elem>] => <Tail as Contains<Elem>>::Output;
}

impl<Head, Tail: IsList + Contains<Elem>, Elem> Contains<Elem> for Array<Head, Tail>
where
    Head: EqDecide<Elem>,
    <Head as EqDecide<Elem>>::Output: ContainsHelper<Elem, Tail>,
{
    type Output = <<Head as EqDecide<Elem>>::Output as ContainsHelper<Elem, Tail>>::Output;
}

define_array_unary_fn!(FLen<Arr> where [Arr: Len] => <Arr as Len>::Output);
define_array_unary_fn!(FHead<Arr> where [Arr: List] => <Arr as List>::Head);
define_array_unary_fn!(FTail<Arr> where [Arr: List] => <Arr as List>::Tail);
define_array_unary_fn!(FIsEmpty<Arr> where [Arr: IsEmpty] => <Arr as IsEmpty>::Output);
define_array_binary_fn!(FGet<Arr, Idx> where [Arr: Get<Idx>, Idx: Unsigned] => <Arr as Get<Idx>>::Output);
define_array_ternary_fn!(FSet<Arr, Idx, Val> where [Arr: Set<Idx, Val>, Idx: Unsigned] => <Arr as Set<Idx, Val>>::Output);
define_array_binary_fn!(FConcat<Lhs, Rhs> where [Lhs: Concat<Rhs>, Rhs: IsList] => <Lhs as Concat<Rhs>>::Output);
define_array_binary_fn!(FAppend<Arr, Elem> where [Arr: Concat<Array<Elem, Nil>>] => <Arr as Concat<Array<Elem, Nil>>>::Output);
define_array_binary_fn!(FPrepend<Elem, Arr> where [Arr: List] => <Arr as List>::Cons<Elem>);
define_array_binary_fn!(FContains<Arr, Elem> where [Arr: Contains<Elem>] => <Arr as Contains<Elem>>::Output);
define_array_binary_fn!(FMap<Op, Arr> where [Arr: MapHelper<Op>] => <Arr as MapHelper<Op>>::Output);
define_array_binary_fn!(FFilter<Pred, Arr> where [Arr: FilterHelper<Pred>] => <Arr as FilterHelper<Pred>>::Output);
define_array_ternary_fn!(FFold<Op, Init, Arr> where [Arr: FoldHelper<Op, Init>] => <Arr as FoldHelper<Op, Init>>::Output);
define_array_unary_fn!(FReverse<Arr> where [Arr: Reverse] => <Arr as Reverse>::Output);
define_array_binary_fn!(FTake<N, Arr> where [Arr: Take<N>] => <Arr as Take<N>>::Output);
define_array_binary_fn!(FDrop<N, Arr> where [Arr: Drop<N>] => <Arr as Drop<N>>::Output);
define_array_unary_fn!(FLast<Arr> where [Arr: Last] => <Arr as Last>::Output);
define_array_binary_fn!(FZip<Lhs, Rhs> where [Lhs: Zip<Rhs>] => <Lhs as Zip<Rhs>>::Output);
define_array_binary_fn!(FFind<Pred, Arr> where [Arr: Find<Pred>] => <Arr as Find<Pred>>::Output);
define_array_binary_fn!(FAny<Pred, Arr> where [Arr: Any<Pred>] => <Arr as Any<Pred>>::Output);
define_array_binary_fn!(FAll<Pred, Arr> where [Arr: All<Pred>] => <Arr as All<Pred>>::Output);
// --- Basic Array Operations ---

/// Get the length of an array.
pub type ELen<Arr> = crate::core::ECall<FLen, Arr>;

/// Get the head (first element) of an array.
pub type EHead<Arr> = crate::core::ECall<FHead, Arr>;

/// Get the tail (all but first) of an array.
pub type ETail<Arr> = crate::core::ECall<FTail, Arr>;

/// Check if an array is empty.
pub type EIsEmpty<Arr> = crate::core::ECall<FIsEmpty, Arr>;

// --- Index Operations ---

/// Get element at index.
pub type EGet<Arr, Idx> = crate::core::ECall2<FGet, Arr, Idx>;

/// Set element at index.
pub type ESet<Arr, Idx, Val> = crate::core::ECall3<FSet, Arr, Idx, Val>;

// --- Concatenation Operations ---

/// Concatenate two arrays.
pub type EConcat<Lhs, Rhs> = crate::core::ECall2<FConcat, Lhs, Rhs>;

/// Append element to end of array.
pub type EAppend<Arr, Elem> = crate::core::ECall2<FAppend, Arr, Elem>;

/// Prepend element to start of array.
pub type EPrepend<Elem, Arr> = crate::core::ECall2<FPrepend, Elem, Arr>;

// --- Higher-Order Operations (defined separately due to complex bounds) ---

/// Map a first-class operator over an array.
pub type EMap<Op, List> = crate::core::ECall2<FMap, ELit<Op>, List>;

/// Filter an array with a predicate operator.
pub type EFilter<Pred, List> = crate::core::ECall2<FFilter, ELit<Pred>, List>;

/// Left fold over an array with a curried binary operator.
pub type EFold<Op, Init, List> = crate::core::ECall3<FFold, ELit<Op>, Init, List>;

/// Check whether an array contains an element.
pub type EContains<Arr, Elem> = crate::core::ECall2<FContains, Arr, Elem>;

// --- Higher-Order Operations ---
// Helper for Map.
crate::helper_list! {
    #[doc(hidden)]
    #[diagnostic::on_unimplemented(
        message = "Internal `MapHelper` not implemented for `{Self}`",
        label = "Map not implemented",
        note = "ensure `{Self}` is a List and `Op` is a valid function"
    )]
    pub trait MapHelper<Op>;
    base Nil => Nil;
    step <Head, Tail> Array<Head, Tail>
        where [
            ECall<Op, ELit<Head>>: Eval,
            Tail: IsList + MapHelper<Op>,
            <Tail as MapHelper<Op>>::Output: IsList
        ]
        => Array<Evaluate<ECall<Op, ELit<Head>>>, <Tail as MapHelper<Op>>::Output>;
}

// Helper for Filter.
crate::helper_list! {
    #[doc(hidden)]
    #[diagnostic::on_unimplemented(
        message = "Internal `FilterHelper` not implemented for `{Self}`",
        label = "Filter not implemented",
        note = "ensure `{Self}` is a List and `Pred` is a valid function"
    )]
    pub trait FilterHelper<Pred>;
    base Nil => Nil;
    step <Head, Tail> Array<Head, Tail>
        where [
            ECall<Pred, ELit<Head>>: Eval,
            Tail: IsList + FilterHelper<Pred>,
            <Tail as FilterHelper<Pred>>::Output: IsList,
            EIf<
                ECall<Pred, ELit<Head>>,
                ELit<Array<Head, <Tail as FilterHelper<Pred>>::Output>>,
                ELit<<Tail as FilterHelper<Pred>>::Output>,
            >: Eval,
            Evaluate<
                EIf<
                    ECall<Pred, ELit<Head>>,
                    ELit<Array<Head, <Tail as FilterHelper<Pred>>::Output>>,
                    ELit<<Tail as FilterHelper<Pred>>::Output>,
                >,
            >: IsList
        ]
        => Evaluate<
            EIf<
                ECall<Pred, ELit<Head>>,
                ELit<Array<Head, <Tail as FilterHelper<Pred>>::Output>>,
                ELit<<Tail as FilterHelper<Pred>>::Output>,
            >,
        >;
}

// Helper for Fold.
crate::helper_list! {
    #[doc(hidden)]
    #[diagnostic::on_unimplemented(
        message = "Internal `FoldHelper` not implemented for `{Self}`",
        label = "Fold not implemented",
        note = "ensure `{Self}` is a Cons list and `Op` is a valid function"
    )]
    pub trait FoldHelper<Op, Acc>;
    base Nil => Acc;
    step <Head, Tail> Array<Head, Tail>
        where [
            ECall2<Op, ELit<Acc>, ELit<Head>>: Eval,
            Tail: IsList
                + FoldHelper<
                    Op,
                    Evaluate<ECall2<Op, ELit<Acc>, ELit<Head>>>,
                >
        ]
        => <Tail as FoldHelper<
            Op,
            Evaluate<ECall2<Op, ELit<Acc>, ELit<Head>>>,
        >>::Output;
}
use crate::std::prim::option::{None, Some};

impl<Op, Init> Foldable<Op, Init> for Nil {
    type Output = Init;
}

impl<Head, Tail: IsList, Op, Init> Foldable<Op, Init> for Array<Head, Tail>
where
    Array<Head, Tail>: FoldHelper<Op, Init>,
{
    type Output = <Array<Head, Tail> as FoldHelper<Op, Init>>::Output;
}

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

// Helper for Find dispatch.
crate::helper_if! {
    #[doc(hidden)]
    pub trait FindHelper<Pred, Head, Tail>;
    on True where [Tail: IsList] => Some<Head>;
    on False where [Tail: IsList + Find<Pred>] => <Tail as Find<Pred>>::Output;
}

impl<Pred, Head, Tail: IsList> Find<Pred> for Array<Head, Tail>
where
    ECall<Pred, ELit<Head>>: Eval,
    Evaluate<ECall<Pred, ELit<Head>>>: FindHelper<Pred, Head, Tail>,
{
    type Output = <Evaluate<ECall<Pred, ELit<Head>>> as FindHelper<Pred, Head, Tail>>::Output;
}

// --- Any<Pred> ---

/// Any trait: true if any element satisfies predicate
pub trait Any<Pred> {
    type Output; // True or False
}

impl<Pred> Any<Pred> for Nil {
    type Output = False;
}

// Helper for Any dispatch.
crate::helper_if! {
    #[doc(hidden)]
    pub trait AnyHelper<Pred, Tail>;
    on True where [Tail: IsList] => True;
    on False where [Tail: IsList + Any<Pred>] => <Tail as Any<Pred>>::Output;
}

impl<Pred, Head, Tail: IsList> Any<Pred> for Array<Head, Tail>
where
    ECall<Pred, ELit<Head>>: Eval,
    Evaluate<ECall<Pred, ELit<Head>>>: AnyHelper<Pred, Tail>,
{
    type Output = <Evaluate<ECall<Pred, ELit<Head>>> as AnyHelper<Pred, Tail>>::Output;
}

// --- All<Pred> ---

/// All trait: true if all elements satisfy predicate
pub trait All<Pred> {
    type Output; // True or False
}

impl<Pred> All<Pred> for Nil {
    type Output = True;
}

// Helper for All dispatch.
crate::helper_if! {
    #[doc(hidden)]
    pub trait AllHelper<Pred, Tail>;
    on True where [Tail: IsList + All<Pred>] => <Tail as All<Pred>>::Output;
    on False where [Tail: IsList] => False;
}

impl<Pred, Head, Tail: IsList> All<Pred> for Array<Head, Tail>
where
    ECall<Pred, ELit<Head>>: Eval,
    Evaluate<ECall<Pred, ELit<Head>>>: AllHelper<Pred, Tail>,
{
    type Output = <Evaluate<ECall<Pred, ELit<Head>>> as AllHelper<Pred, Tail>>::Output;
}

// --- Additional list expressions ---

/// Reverse a list.
pub type EReverse<List> = crate::core::ECall<FReverse, List>;

/// Take first N elements from a list.
pub type ETake<N, List> = crate::core::ECall2<FTake, N, List>;

/// Drop first N elements from a list.
pub type EDrop<N, List> = crate::core::ECall2<FDrop, N, List>;

/// Get the last element of a list.
pub type ELast<List> = crate::core::ECall<FLast, List>;

/// Zip two lists together pairwise.
pub type EZip<L1, L2> = crate::core::ECall2<FZip, L1, L2>;

/// Find the first element matching a predicate.
pub type EFind<Pred, List> = crate::core::ECall2<FFind, ELit<Pred>, List>;

/// True if any element matches a predicate.
pub type EAny<Pred, List> = crate::core::ECall2<FAny, ELit<Pred>, List>;

/// True if all elements match a predicate.
pub type EAll<Pred, List> = crate::core::ECall2<FAll, ELit<Pred>, List>;

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_std::core::{ELit, TyFn};
    use typenum::{U0, U1, U2, U4, U10, U12};

    use super::*;
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
    fn test_econtains() {
        use typenum::{U1, U2, U3, U4, U9};

        type ListExpr = ELit<tyarray![U1, U2, U3, U4]>;

        assert_type_eq_all!(Evaluate<EContains<ListExpr, ELit<U1>>>, True);
        assert_type_eq_all!(Evaluate<EContains<ListExpr, ELit<U4>>>, True);
        assert_type_eq_all!(Evaluate<EContains<ListExpr, ELit<U9>>>, False);

        type Concatenated = EConcat<ELit<tyarray![U1]>, ELit<tyarray![U2]>>;
        assert_type_eq_all!(Evaluate<EContains<Concatenated, ELit<U2>>>, True);
        assert_type_eq_all!(Evaluate<EContains<Concatenated, ELit<U9>>>, False);
    }

    #[test]
    fn test_emap() {
        use typenum::{Add1, U1, U2, U3, U4};

        struct FnAddOne;
        impl<T> TyFn<T> for FnAddOne
        where
            T: std::ops::Add<typenum::B1>,
        {
            type Output = Add1<T>;
        }

        type List = tyarray![U1, U2, U3];
        type Mapped = EMap<FnAddOne, ELit<List>>;

        assert_type_eq_all!(Evaluate<Mapped>, tyarray![U2, U3, U4]);
    }

    #[test]
    fn test_efilter() {
        use typenum::{IsLess, U1, U2, U3, U4, U5};

        struct PredLessThan3;
        impl<T> TyFn<T> for PredLessThan3
        where
            T: IsLess<U3>,
            bool: From<<T as IsLess<U3>>::Output>,
        {
            type Output = ToBoolOut<<T as IsLess<U3>>::Output>;
        }

        type List = tyarray![U1, U5, U2, U4, U3]; // [1, 5, 2, 4, 3]
        // Filter < 3 -> [1, 2]
        type Filtered = EFilter<PredLessThan3, ELit<List>>;

        assert_type_eq_all!(Evaluate<Filtered>, tyarray![U1, U2]);
    }

    #[test]
    fn test_efold() {
        use typenum::{U0, U1, U2, U3, U6};

        crate::typelude_macros::ty_fn! {
            struct FnSum<Acc> {
                type Output = FnSumCaptured<Acc>;
            }
        }

        crate::typelude_macros::ty_fn! {
            struct FnSumCaptured<Acc, Elem>
            where [Acc: std::ops::Add<Elem>]
            {
                type Output = <Acc as std::ops::Add<Elem>>::Output;
            }
        }

        type List = tyarray![U1, U2, U3];
        // Fold Sum 0 [1, 2, 3] -> 6
        type Summed = EFold<FnSum, ELit<U0>, ELit<List>>;

        assert_type_eq_all!(Evaluate<Summed>, U6);
    }

    #[test]
    fn test_first_class_ops_and_foldable() {
        use typenum::{U0, U1, U2, U3, U6};

        type List = tyarray![U1, U2, U3];

        assert_type_eq_all!(
            Evaluate<crate::core::ECall2<crate::std::ops::FAdd, ELit<U1>, ELit<U2>>>,
            U3
        );
        assert_type_eq_all!(
            Evaluate<crate::core::ECall<crate::std::ops::FNot, ELit<True>>>,
            False
        );
        assert_type_eq_all!(Evaluate<crate::core::ECall2<FGet, ELit<List>, ELit<U1>>>, U2);
        assert_type_eq_all!(<List as Foldable<crate::std::ops::FAdd, U0>>::Output, U6);
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
