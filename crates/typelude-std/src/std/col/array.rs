//! **Type-Level Array**
//!
//! Type-level array (cons list) and its operations.

use std::ops::{Add, Sub};

use typelude_std::core::{EApp, ELit, Eval, Evaluate};
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
// --- Basic Array Operations ---

crate::def_expr_via_trait!(
    /// Get the length of an array
    pub ELen<Arr>
    args [Arr]
    where [~Arr: Len]
    => <~Arr as Len>::Output
);

crate::def_expr_via_trait!(
    /// Get the head (first element) of an array
    pub EHead<Arr>
    args [Arr]
    where [~Arr: List]
    => <~Arr as List>::Head
);

crate::def_expr_via_trait!(
    /// Get the tail (all but first) of an array
    pub ETail<Arr>
    args [Arr]
    where [~Arr: List, <~Arr as List>::Tail: Eval]
    => <~Arr as List>::Tail
);

crate::def_expr_via_trait!(
    /// Check if an array is empty
    pub EIsEmpty<Arr>
    args [Arr]
    where [~Arr: IsEmpty]
    => <~Arr as IsEmpty>::Output
);

// --- Index Operations ---

crate::def_expr_via_trait!(
    /// Get element at index
    pub EGet<Arr, Idx>
    args [Idx, Arr]
    where [~Idx: Unsigned, ~Arr: Get<~Idx>]
    => <~Arr as Get<~Idx>>::Output
);

crate::def_expr_via_trait!(
    /// Set element at index
    pub ESet<Arr, Idx, Val>
    args [Idx, Arr, Val]
    where [~Idx: Unsigned, ~Arr: Set<~Idx, ~Val>]
    => <~Arr as Set<~Idx, ~Val>>::Output
);

// --- Concatenation Operations ---

crate::def_expr_via_trait!(
    /// Concatenate two arrays
    pub EConcat<Lhs, Rhs>
    args [Rhs, Lhs]
    where [~Rhs: IsList, ~Lhs: Concat<~Rhs>]
    => <~Lhs as Concat<~Rhs>>::Output
);

crate::def_expr_via_trait!(
    /// Append element to end of array
    pub EAppend<Arr, Elem>
    args [Arr, Elem]
    where [~Arr: Concat<Array<~Elem, Nil>>]
    => <~Arr as Concat<Array<~Elem, Nil>>>::Output
);

crate::def_expr_via_trait!(
    /// Prepend element to start of array
    pub EPrepend<Elem, Arr>
    args [Arr, Elem]
    where [~Arr: List]
    => <~Arr as List>::Cons<~Elem>
);

// --- Higher-Order Operations (defined separately due to complex bounds) ---

// --- EMap ---
crate::def_expr_via_trait!(
    pub EMap<Op, List>
    args [List]
    where [~List: MapHelper<Op>]
    => <~List as MapHelper<Op>>::Output
);

// --- EFilter ---
crate::def_expr_via_trait!(
    pub EFilter<Pred, List>
    args [List]
    where [~List: FilterHelper<Pred>]
    => <~List as FilterHelper<Pred>>::Output
);

// --- EFold ---
crate::def_expr_via_trait!(
    pub EFold<Op, Init, List>
    args [Init, List]
    where [~List: FoldHelper<Op, ~Init>]
    => <~List as FoldHelper<Op, ~Init>>::Output
);

// Expression to check if an array contains an element.
#[cfg(feature = "nightly")]
crate::def_expr_via_trait!(
    pub EContains<Array, Elem>
    args [Array, Elem]
    where [
        ~Array: Contains<~Elem>,
        (): crate::std::reify::ReflectBool<{ <Evaluate<Array> as Contains<Evaluate<Elem>>>::VALUE }>
    ]
    => ~crate::std::prim::bool::Assert<{ <Evaluate<Array> as Contains<Evaluate<Elem>>>::VALUE }>
);

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
            EApp<ELit<Op>, ELit<Head>>: Eval,
            crate::eval_once!(EApp<ELit<Op>, ELit<Head>>): Eval,
            Tail: IsList + MapHelper<Op>,
            <Tail as MapHelper<Op>>::Output: IsList
        ]
        => Array<crate::eval_twice!(EApp<ELit<Op>, ELit<Head>>), <Tail as MapHelper<Op>>::Output>;
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
            EApp<ELit<Pred>, ELit<Head>>: Eval,
            Tail: IsList + FilterHelper<Pred>,
            <Tail as FilterHelper<Pred>>::Output: IsList,
            EIf<
                crate::eval_once!(EApp<ELit<Pred>, ELit<Head>>),
                ELit<Array<Head, <Tail as FilterHelper<Pred>>::Output>>,
                ELit<<Tail as FilterHelper<Pred>>::Output>,
            >: Eval,
            Evaluate<
                EIf<
                    crate::eval_once!(EApp<ELit<Pred>, ELit<Head>>),
                    ELit<Array<Head, <Tail as FilterHelper<Pred>>::Output>>,
                    ELit<<Tail as FilterHelper<Pred>>::Output>,
                >,
            >: IsList
        ]
        => Evaluate<
            EIf<
                crate::eval_once!(EApp<ELit<Pred>, ELit<Head>>),
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
            EApp<
                ELit<Op>,
                ELit<
                    typelude_std::core::ECons<
                        Acc,
                        typelude_std::core::ECons<Head, typelude_std::core::ENil>,
                    >,
                >,
            >: Eval,
            Evaluate<
                EApp<
                    ELit<Op>,
                    ELit<
                        typelude_std::core::ECons<
                            Acc,
                            typelude_std::core::ECons<Head, typelude_std::core::ENil>,
                        >,
                    >,
                >,
            >: Eval,
            Tail: IsList
                + FoldHelper<
                    Op,
                    crate::eval_twice!(
                        EApp<
                            ELit<Op>,
                            ELit<
                                typelude_std::core::ECons<
                                    Acc,
                                    typelude_std::core::ECons<Head, typelude_std::core::ENil>,
                                >,
                            >,
                        >
                    ),
                >
        ]
        => <Tail as FoldHelper<
            Op,
            crate::eval_twice!(
                EApp<
                    ELit<Op>,
                    ELit<
                        typelude_std::core::ECons<
                            Acc,
                            typelude_std::core::ECons<Head, typelude_std::core::ENil>,
                        >,
                    >,
                >
            ),
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

// Helper for Find dispatch.
crate::helper_if! {
    #[doc(hidden)]
    pub trait FindHelper<Pred, Head, Tail>;
    on True where [Tail: IsList] => Some<Head>;
    on False where [Tail: IsList + Find<Pred>] => <Tail as Find<Pred>>::Output;
}

impl<Pred, Head, Tail: IsList> Find<Pred> for Array<Head, Tail>
where
    EApp<ELit<Pred>, ELit<Head>>: Eval,
    crate::eval_once!(EApp<ELit<Pred>, ELit<Head>>): Eval,
    crate::eval_twice!(EApp<ELit<Pred>, ELit<Head>>): FindHelper<Pred, Head, Tail>,
{
    type Output =
        <crate::eval_twice!(EApp<ELit<Pred>, ELit<Head>>) as FindHelper<Pred, Head, Tail>>::Output;
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
    EApp<ELit<Pred>, ELit<Head>>: Eval,
    crate::eval_once!(EApp<ELit<Pred>, ELit<Head>>): Eval,
    crate::eval_twice!(EApp<ELit<Pred>, ELit<Head>>): AnyHelper<Pred, Tail>,
{
    type Output =
        <crate::eval_twice!(EApp<ELit<Pred>, ELit<Head>>) as AnyHelper<Pred, Tail>>::Output;
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
    EApp<ELit<Pred>, ELit<Head>>: Eval,
    crate::eval_once!(EApp<ELit<Pred>, ELit<Head>>): Eval,
    crate::eval_twice!(EApp<ELit<Pred>, ELit<Head>>): AllHelper<Pred, Tail>,
{
    type Output =
        <crate::eval_twice!(EApp<ELit<Pred>, ELit<Head>>) as AllHelper<Pred, Tail>>::Output;
}

// --- Additional list expressions ---

crate::def_expr_via_trait!(
    /// Reverse a list
    pub EReverse<List>
    args [List]
    where [~List: Reverse]
    => <~List as Reverse>::Output
);

crate::def_expr_via_trait!(
    /// Take first N elements from a list
    pub ETake<N, List>
    args [N, List]
    where [~List: Take<~N>]
    => <~List as Take<~N>>::Output
);

crate::def_expr_via_trait!(
    /// Drop first N elements from a list
    pub EDrop<N, List>
    args [N, List]
    where [~List: Drop<~N>]
    => <~List as Drop<~N>>::Output
);

crate::def_expr_via_trait!(
    /// Get the last element of a list
    pub ELast<List>
    args [List]
    where [~List: Last]
    => <~List as Last>::Output
);

crate::def_expr_via_trait!(
    /// Zip two lists together pairwise
    pub EZip<L1, L2>
    args [L1, L2]
    where [~L1: Zip<~L2>]
    => <~L1 as Zip<~L2>>::Output
);

crate::def_expr_via_trait!(
    /// Find first element matching predicate
    pub EFind<Pred, List>
    args [Pred, List]
    where [~List: Find<~Pred>]
    => <~List as Find<~Pred>>::Output
);

crate::def_expr_via_trait!(
    /// True if any element matches predicate
    pub EAny<Pred, List>
    args [Pred, List]
    where [~List: Any<~Pred>]
    => <~List as Any<~Pred>>::Output
);

crate::def_expr_via_trait!(
    /// True if all elements match predicate
    pub EAll<Pred, List>
    args [Pred, List]
    where [~List: All<~Pred>]
    => <~List as All<~Pred>>::Output
);

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
        impl<T> TyFn<T> for FnAddOne
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
        impl<T> TyFn<T> for PredLessThan3
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
            TyFn<
                typelude_std::core::ECons<
                    Acc,
                    typelude_std::core::ECons<Elem, typelude_std::core::ENil>,
                >,
            > for FnSum
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
