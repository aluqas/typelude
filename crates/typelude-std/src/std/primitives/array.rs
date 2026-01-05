//! **Type-Level Array**
//!
//! Type-level array (cons list) and its operations.

use std::{
    marker::PhantomData,
    ops::{Add, Sub},
};

use typelude_core::{ELit, Eval, Evaluate};
use typenum::{B1, Sub1, U0, UInt, Unsigned};

// =============================================================================
// Layer 1: Values (Data Structure)
// =============================================================================

// Re-export kernel types for convenience/compatibility if mostly used from here
pub use crate::data::collections::array::{Cons, TyArray, TyNil};
use crate::{
    data::primitives::bool::{TyFalse, TyTrue},
    expr::EIf,
    std::traits::TypeList,
    traits::Apply,
};

// =============================================================================
// Adapter Implementation: TypeList for TyArray/TyNil
// =============================================================================

impl TypeList for TyNil {
    type Cons<NewHead> = TyArray<NewHead, TyNil>;
    // Head/Tail for Nil are usually undefined or Unit/Nil
    type Head = (); // Or a custom Error Type
    type Tail = TyNil;
}

impl<Head, Tail: Cons> TypeList for TyArray<Head, Tail> {
    type Cons<NewHead> = TyArray<NewHead, Self>;
    type Head = Head;
    type Tail = Tail;
}

// =============================================================================
// Layer 2: Capabilities (Verbs) - Deprecated/Wrapped by TypeList, but kept for logic
// =============================================================================

/// Array length
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a type with a length",
    label = "Len not implemented",
    note = "ensure `{Self}` implements `Len`"
)]
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
#[diagnostic::on_unimplemented(
    message = "`{Self}` does not have a Head element",
    label = "Head not implemented",
    note = "ensure `{Self}` is a non-empty list"
)]
pub trait Head {
    type Output;
}

impl<H, T: Cons> Head for TyArray<H, T> {
    type Output = H;
}

/// Tail of array (everything except head)
#[diagnostic::on_unimplemented(
    message = "`{Self}` does not have a Tail",
    label = "Tail not implemented",
    note = "ensure `{Self}` is a non-empty list"
)]
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
#[diagnostic::on_unimplemented(
    message = "Cannot access index `{Idx}` in `{Self}`",
    label = "index access failed",
    note = "index might be out of bounds or `{Self}` is not a list"
)]
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
#[diagnostic::on_unimplemented(
    message = "Cannot set index `{Idx}` in `{Self}` to `{Val}`",
    label = "index update failed",
    note = "index might be out of bounds or `{Self}` is not a list"
)]
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
use crate::std::ops::IsEq;

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
// Layer 3 & 5: Operations via def_op! Macro
// =============================================================================

use typelude_macros::def_op;

// --- Basic Array Operations ---

def_op! {
    /// Get the length of an array
    name: OpLen,
    args: (Array),
    ast: ELen {
        where: [
            Evaluate<Array>: Len
        ],
        type Output = <Evaluate<Array> as Len>::Output
    }
}

def_op! {
    /// Get the head (first element) of an array
    name: OpHead,
    args: (Array),
    ast: EHead {
        where: [
            Evaluate<Array>: TypeList
        ],
        type Output = <Evaluate<Array> as TypeList>::Head
    }
}

def_op! {
    /// Get the tail (all but first) of an array
    name: OpTail,
    args: (Array),
    ast: ETail {
        where: [
            Evaluate<Array>: TypeList,
            <Evaluate<Array> as TypeList>::Tail: Eval
        ],
        type Output = <Evaluate<Array> as TypeList>::Tail
    }
}

def_op! {
    /// Check if an array is empty
    name: OpIsEmpty,
    args: (Array),
    ast: EIsEmpty {
        where: [
            Evaluate<Array>: IsEmpty
        ],
        type Output = <Evaluate<Array> as IsEmpty>::Output
    }
}

// --- Index Operations ---

def_op! {
    /// Get element at index
    name: OpGet,
    args: (Array, Idx),
    ast: EGet {
        where: [
            Evaluate<Idx>: Unsigned,
            Evaluate<Array>: Get<Evaluate<Idx>>
        ],
        type Output = <Evaluate<Array> as Get<Evaluate<Idx>>>::Output
    }
}

def_op! {
    /// Set element at index
    name: OpSet,
    args: (Array, Idx, Val),
    ast: ESet {
        where: [
            Evaluate<Idx>: Unsigned,
            Evaluate<Array>: Set<Evaluate<Idx>, Evaluate<Val>>
        ],
        type Output = <Evaluate<Array> as Set<Evaluate<Idx>, Evaluate<Val>>>::Output
    }
}

// --- Concatenation Operations ---

def_op! {
    /// Concatenate two arrays
    name: OpConcat,
    args: (Lhs, Rhs),
    ast: EConcat {
        where: [
            Evaluate<Rhs>: Cons,
            Evaluate<Lhs>: Concat<Evaluate<Rhs>>
        ],
        type Output = <Evaluate<Lhs> as Concat<Evaluate<Rhs>>>::Output
    }
}

def_op! {
    /// Append element to end of array
    name: OpAppend,
    args: (Array, Elem),
    ast: EAppend {
        where: [
            Evaluate<Array>: Concat<TyArray<Evaluate<Elem>, TyNil>>
        ],
        type Output = <Evaluate<Array> as Concat<TyArray<Evaluate<Elem>, TyNil>>>::Output
    }
}

def_op! {
    /// Prepend element to start of array
    name: OpPrepend,
    args: (Elem, Array),
    ast: EPrepend {
        where: [
            Evaluate<Array>: TypeList
        ],
        type Output = <Evaluate<Array> as TypeList>::Cons<Evaluate<Elem>>
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

// --- EContains ---
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

// --- Operator Symbols for Higher-Order Operations ---

def_op! {
    /// Map a function over an array
    name: OpMap,
    args: (Op, List),
    alias: EMap<Op, List>
}

def_op! {
    /// Filter array elements by predicate
    name: OpFilter,
    args: (Pred, List),
    alias: EFilter<Pred, List>
}

def_op! {
    /// Fold/reduce an array
    name: OpFold,
    args: (Op, Init, List),
    alias: EFold<Op, Init, List>
}

// OpContains is nightly-only
#[cfg(feature = "nightly")]
pub struct OpContains;
#[cfg(feature = "nightly")]
impl<Array, Elem> Apply<(Array, Elem)> for OpContains {
    type Output = EContains<Array, Elem>;
}

// =============================================================================
// Layer 4: Backends (Helpers)
// =============================================================================

/// Helper for Map
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "Internal `MapHelper` not implemented for `{Self}`",
    label = "Map not implemented",
    note = "ensure `{Self}` is a Cons list and `Op` is valid"
)]
pub trait MapHelper<Op> {
    type Output: Cons;
}

impl<Op> MapHelper<Op> for TyNil {
    type Output = TyNil;
}

impl<Op, Head, Tail> MapHelper<Op> for TyArray<Head, Tail>
where
    Op: Apply<Head>,
    Op::Output: Eval, // Evaluate the result of application
    Tail: Cons + MapHelper<Op>,
    <Tail as MapHelper<Op>>::Output: Cons,
{
    // Evaluate application result for strict map
    type Output = TyArray<Evaluate<Op::Output>, <Tail as MapHelper<Op>>::Output>;
}

/// Helper for Filter
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "Internal `FilterHelper` not implemented for `{Self}`",
    label = "Filter not implemented",
    note = "ensure `{Self}` is a Cons list and `Pred` is valid"
)]
pub trait FilterHelper<Pred> {
    type Output: Cons;
}

impl<P> FilterHelper<P> for TyNil {
    type Output = TyNil;
}

impl<P, Head, Tail> FilterHelper<P> for TyArray<Head, Tail>
where
    P: Apply<Head>,
    P::Output: Eval,
    Tail: Cons + FilterHelper<P>,
    <Tail as FilterHelper<P>>::Output: Cons,
    // EIf<Pred(Head), Cons<Head, Filter(Tail)>, Filter(Tail)>
    EIf<
        P::Output, // Predicate Result Expr
        ELit<TyArray<Head, <Tail as FilterHelper<P>>::Output>>,
        ELit<<Tail as FilterHelper<P>>::Output>,
    >: Eval,
    Evaluate<
        EIf<
            P::Output,
            ELit<TyArray<Head, <Tail as FilterHelper<P>>::Output>>,
            ELit<<Tail as FilterHelper<P>>::Output>,
        >,
    >: Cons,
{
    type Output = Evaluate<
        EIf<
            P::Output,
            ELit<TyArray<Head, <Tail as FilterHelper<P>>::Output>>,
            ELit<<Tail as FilterHelper<P>>::Output>,
        >,
    >;
}

/// Helper for Fold
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "Internal `FoldHelper` not implemented for `{Self}`",
    label = "Fold not implemented",
    note = "ensure `{Self}` is a Cons list and `Op` is valid"
)]
pub trait FoldHelper<Op, Acc> {
    type Output;
}

impl<Op, Acc> FoldHelper<Op, Acc> for TyNil {
    type Output = Acc;
}

impl<Op, Acc, Head, Tail> FoldHelper<Op, Acc> for TyArray<Head, Tail>
where
    Op: Apply<(Acc, Head)>,
    Op::Output: Eval, // Evaluate acc+head
    Tail: Cons + FoldHelper<Op, Evaluate<Op::Output>>,
{
    // Strict Fold
    type Output = <Tail as FoldHelper<Op, Evaluate<Op::Output>>>::Output;
}

// =============================================================================
// RFC-0001 Phase 1: New Array Operations
// =============================================================================

use crate::std::primitives::option::{TyNone, TySome};

// --- Reverse ---

/// Reverse trait: reverses the order of elements
pub trait Reverse {
    type Output;
}

impl Reverse for TyNil {
    type Output = TyNil;
}

impl<Head, Tail: Cons> Reverse for TyArray<Head, Tail>
where
    Tail: Reverse,
    <Tail as Reverse>::Output: Concat<TyArray<Head, TyNil>>,
{
    type Output = <<Tail as Reverse>::Output as Concat<TyArray<Head, TyNil>>>::Output;
}

// --- Take<N> ---

/// Take trait: take first N elements
pub trait Take<N> {
    type Output;
}

impl<N> Take<N> for TyNil {
    type Output = TyNil;
}

impl<Head, Tail: Cons> Take<U0> for TyArray<Head, Tail> {
    type Output = TyNil;
}

impl<Head, Tail: Cons, N: Unsigned, B: typenum::Bit> Take<UInt<N, B>> for TyArray<Head, Tail>
where
    UInt<N, B>: Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    Tail: Take<Sub1<UInt<N, B>>>,
    <Tail as Take<Sub1<UInt<N, B>>>>::Output: Cons,
{
    type Output = TyArray<Head, <Tail as Take<Sub1<UInt<N, B>>>>::Output>;
}

// --- Drop<N> ---

/// Drop trait: skip first N elements
pub trait Drop<N> {
    type Output;
}

impl<N> Drop<N> for TyNil {
    type Output = TyNil;
}

impl<Head, Tail: Cons> Drop<U0> for TyArray<Head, Tail> {
    type Output = TyArray<Head, Tail>;
}

impl<Head, Tail: Cons, N: Unsigned, B: typenum::Bit> Drop<UInt<N, B>> for TyArray<Head, Tail>
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

impl<T> Last for TyArray<T, TyNil> {
    type Output = T;
}

impl<Head, T, Rest: Cons> Last for TyArray<Head, TyArray<T, Rest>>
where
    TyArray<T, Rest>: Last,
{
    type Output = <TyArray<T, Rest> as Last>::Output;
}

// --- Zip ---

/// Zip trait: combine two lists pairwise
pub trait Zip<Other> {
    type Output;
}

impl<Other> Zip<Other> for TyNil {
    type Output = TyNil;
}

impl<H, T: Cons> Zip<TyNil> for TyArray<H, T> {
    type Output = TyNil;
}

impl<H1, T1: Cons, H2, T2: Cons> Zip<TyArray<H2, T2>> for TyArray<H1, T1>
where
    T1: Zip<T2>,
    <T1 as Zip<T2>>::Output: Cons,
{
    type Output = TyArray<(H1, H2), <T1 as Zip<T2>>::Output>;
}

// --- Find<Pred> ---

/// Find trait: find first element matching predicate, returns TySome/TyNone
pub trait Find<Pred> {
    type Output; // TySome<T> or TyNone
}

impl<Pred> Find<Pred> for TyNil {
    type Output = TyNone;
}

/// Helper for Find dispatch
#[doc(hidden)]
pub trait FindHelper<Pred, Head, Tail> {
    type Output;
}

impl<Pred, Head, Tail: Cons> FindHelper<Pred, Head, Tail> for TyTrue {
    type Output = TySome<Head>;
}

impl<Pred, Head, Tail: Cons + Find<Pred>> FindHelper<Pred, Head, Tail> for TyFalse {
    type Output = <Tail as Find<Pred>>::Output;
}

impl<Pred, Head, Tail: Cons> Find<Pred> for TyArray<Head, Tail>
where
    Pred: Apply<Head>,
    <Pred as Apply<Head>>::Output: Eval,
    Evaluate<<Pred as Apply<Head>>::Output>: FindHelper<Pred, Head, Tail>,
{
    type Output =
        <Evaluate<<Pred as Apply<Head>>::Output> as FindHelper<Pred, Head, Tail>>::Output;
}

// --- Any<Pred> ---

/// Any trait: true if any element satisfies predicate
pub trait Any<Pred> {
    type Output; // TyTrue or TyFalse
}

impl<Pred> Any<Pred> for TyNil {
    type Output = TyFalse;
}

/// Helper for Any dispatch
#[doc(hidden)]
pub trait AnyHelper<Pred, Tail> {
    type Output;
}

impl<Pred, Tail: Cons> AnyHelper<Pred, Tail> for TyTrue {
    type Output = TyTrue;
}

impl<Pred, Tail: Cons + Any<Pred>> AnyHelper<Pred, Tail> for TyFalse {
    type Output = <Tail as Any<Pred>>::Output;
}

impl<Pred, Head, Tail: Cons> Any<Pred> for TyArray<Head, Tail>
where
    Pred: Apply<Head>,
    <Pred as Apply<Head>>::Output: Eval,
    Evaluate<<Pred as Apply<Head>>::Output>: AnyHelper<Pred, Tail>,
{
    type Output = <Evaluate<<Pred as Apply<Head>>::Output> as AnyHelper<Pred, Tail>>::Output;
}

// --- All<Pred> ---

/// All trait: true if all elements satisfy predicate
pub trait All<Pred> {
    type Output; // TyTrue or TyFalse
}

impl<Pred> All<Pred> for TyNil {
    type Output = TyTrue;
}

/// Helper for All dispatch
#[doc(hidden)]
pub trait AllHelper<Pred, Tail> {
    type Output;
}

impl<Pred, Tail: Cons + All<Pred>> AllHelper<Pred, Tail> for TyTrue {
    type Output = <Tail as All<Pred>>::Output;
}

impl<Pred, Tail: Cons> AllHelper<Pred, Tail> for TyFalse {
    type Output = TyFalse;
}

impl<Pred, Head, Tail: Cons> All<Pred> for TyArray<Head, Tail>
where
    Pred: Apply<Head>,
    <Pred as Apply<Head>>::Output: Eval,
    Evaluate<<Pred as Apply<Head>>::Output>: AllHelper<Pred, Tail>,
{
    type Output = <Evaluate<<Pred as Apply<Head>>::Output> as AllHelper<Pred, Tail>>::Output;
}

// --- RFC-0001 Phase 1: Expression + Op definitions via def_op! ---

def_op! {
    /// Reverse a list
    name: OpReverse,
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
    name: OpTake,
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
    name: OpDrop,
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
    name: OpLast,
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
    name: OpZip,
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
    name: OpFind,
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
    name: OpAny,
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
    name: OpAll,
    args: (Pred, List),
    ast: EAll {
        where: [
            Evaluate<List>: All<Evaluate<Pred>>
        ],
        type Output = <Evaluate<List> as All<Evaluate<Pred>>>::Output
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_core::ELit;
    use typenum::{U0, U1, U2, U10, U12};

    use super::*;
    use crate::{
        std::{
            bool::{ToTyBoolOut, TyFalse, TyTrue},
            control::EIf, // Updated from expr::EIf
            ops::TyFrom,
        },
        tyarray,
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

        // ELen<EConcat<...>>
        assert_type_eq_all!(Evaluate<ELen<Concatenated>>, typenum::U4);

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

        struct OpAddOne;
        impl<T> Apply<T> for OpAddOne
        where
            T: std::ops::Add<typenum::B1>,
        {
            type Output = ELit<Add1<T>>;
        }

        type List = tyarray![U1, U2, U3];
        type Mapped = EMap<OpAddOne, ELit<List>>;

        assert_type_eq_all!(Evaluate<Mapped>, tyarray![U2, U3, U4]);
    }

    #[test]
    fn test_efilter() {
        use typenum::{IsLess, U1, U2, U3, U4, U5};

        struct OpLessThan3;
        impl<T> Apply<T> for OpLessThan3
        where
            T: IsLess<U3>,
            bool: TyFrom<<T as IsLess<U3>>::Output>,
        {
            type Output = ELit<ToTyBoolOut<<T as IsLess<U3>>::Output>>;
        }

        type List = tyarray![U1, U5, U2, U4, U3]; // [1, 5, 2, 4, 3]
        // Filter < 3 -> [1, 2]
        type Filtered = EFilter<OpLessThan3, ELit<List>>;

        assert_type_eq_all!(Evaluate<Filtered>, tyarray![U1, U2]);
    }

    #[test]
    fn test_efold() {
        use typenum::{U0, U1, U2, U3, U6};

        // Sum: (Acc, Elem) -> Acc + Elem
        struct OpSum;
        impl<Acc, Elem> Apply<(Acc, Elem)> for OpSum
        where
            Acc: std::ops::Add<Elem>,
        {
            type Output = ELit<<Acc as std::ops::Add<Elem>>::Output>;
        }

        type List = tyarray![U1, U2, U3];
        // Fold Sum 0 [1, 2, 3] -> 6
        type Summed = EFold<OpSum, ELit<U0>, ELit<List>>;

        assert_type_eq_all!(Evaluate<Summed>, U6);
    }

    // ==========================================================================
    // RFC-0001 Phase 1: New Operations Tests
    // ==========================================================================

    #[test]
    fn test_reverse() {
        use typenum::{U1, U2, U3};

        type List = tyarray![U1, U2, U3];
        type Reversed = <List as Reverse>::Output;
        assert_type_eq_all!(Reversed, tyarray![U3, U2, U1]);

        // Empty list
        type EmptyReversed = <TyNil as Reverse>::Output;
        assert_type_eq_all!(EmptyReversed, TyNil);
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
        assert_type_eq_all!(TakeZero, TyNil);

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
