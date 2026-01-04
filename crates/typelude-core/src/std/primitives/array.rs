//! **Type-Level Array**
//!
//! Type-level array (cons list) and its operations.

use std::{
    marker::PhantomData,
    ops::{Add, Sub},
};

use typenum::{B1, Sub1, U0, UInt, Unsigned};

// =============================================================================
// Layer 1: Values (Data Structure)
// =============================================================================

// Re-export kernel types for convenience/compatibility if mostly used from here
pub use crate::kernel::array::{Cons, TyArray, TyNil};
use crate::{
    eval::{EIf, ELit, Eval, Evaluate, Sealed},
    std::traits::TypeList,
};
use crate::kernel::{
    bool::{TyFalse, TyTrue},
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
// Layer 3: Expression Structs (Direct Style)
// =============================================================================

// --- ELen ---
pub struct ELen<Array>(PhantomData<Array>);

impl<Array> Eval for ELen<Array>
where
    Array: Eval,
    Evaluate<Array>: Len, // Keeping Len for now as TypeList doesn't enforce "Unsigned" output for Length directly yet
{
    type Output = <Evaluate<Array> as Len>::Output;
}

// --- EHead ---
pub struct EHead<Array>(PhantomData<Array>);

impl<Array> Eval for EHead<Array>
where
    Array: Eval,
    Evaluate<Array>: TypeList,
{
    type Output = <Evaluate<Array> as TypeList>::Head;
}

// --- ETail ---
pub struct ETail<Array>(PhantomData<Array>);

impl<Array> Eval for ETail<Array>
where
    Array: Eval,
    Evaluate<Array>: TypeList,
    <Evaluate<Array> as TypeList>::Tail: Eval,
{
    type Output = <Evaluate<Array> as TypeList>::Tail;
}

// --- EIsEmpty ---
pub struct EIsEmpty<Array>(PhantomData<Array>);

impl<Array> Eval for EIsEmpty<Array>
where
    Array: Eval,
    Evaluate<Array>: IsEmpty, // Keep IsEmpty trait for now as logic is specific
{
    type Output = <Evaluate<Array> as IsEmpty>::Output;
}

// --- EGet ---
pub struct EGet<Array, Idx>(PhantomData<(Array, Idx)>);

impl<Array, Idx> Eval for EGet<Array, Idx>
where
    Array: Eval,
    Idx: Eval,
    Evaluate<Idx>: Unsigned,
    Evaluate<Array>: Get<Evaluate<Idx>>,
{
    type Output = <Evaluate<Array> as Get<Evaluate<Idx>>>::Output;
}

// --- ESet ---
pub struct ESet<Array, Idx, Val>(PhantomData<(Array, Idx, Val)>);

impl<Array, Idx, Val> Eval for ESet<Array, Idx, Val>
where
    Array: Eval,
    Idx: Eval,
    Val: Eval,
    Evaluate<Idx>: Unsigned,
    Evaluate<Array>: Set<Evaluate<Idx>, Evaluate<Val>>,
{
    type Output = <Evaluate<Array> as Set<Evaluate<Idx>, Evaluate<Val>>>::Output;
}

// --- EConcat ---
pub struct EConcat<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

impl<Lhs, Rhs> Eval for EConcat<Lhs, Rhs>
where
    Lhs: Eval,
    Rhs: Eval,
    Evaluate<Rhs>: Cons,
    Evaluate<Lhs>: Concat<Evaluate<Rhs>>,
{
    type Output = <Evaluate<Lhs> as Concat<Evaluate<Rhs>>>::Output;
}

// --- EAppend ---
pub struct EAppend<Array, Elem>(PhantomData<(Array, Elem)>);

impl<Array, Elem> Eval for EAppend<Array, Elem>
where
    Array: Eval,
    Elem: Eval,
    Evaluate<Array>: Concat<TyArray<Evaluate<Elem>, TyNil>>,
{
    type Output = <Evaluate<Array> as Concat<TyArray<Evaluate<Elem>, TyNil>>>::Output;
}

// --- EPrepend ---
pub struct EPrepend<Elem, Array>(PhantomData<(Elem, Array)>);

impl<Elem, Array> Eval for EPrepend<Elem, Array>
where
    Elem: Eval,
    Array: Eval,
    Evaluate<Array>: TypeList, // Use TypeList::Cons
{
    type Output = <Evaluate<Array> as TypeList>::Cons<Evaluate<Elem>>;
}

// --- EContains ---
pub struct EContains<Array, Elem>(PhantomData<(Array, Elem)>);

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

// =============================================================================
// Layer 4: Backends (Helpers)
// =============================================================================

/// Helper for Map
#[doc(hidden)]
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
// Layer 5: Operator Symbols (Apply Impls)
// =============================================================================

pub struct OpLen;
pub struct OpHead;
pub struct OpTail;
pub struct OpIsEmpty;
pub struct OpGet;
pub struct OpSet;
pub struct OpConcat;
pub struct OpAppend;
pub struct OpPrepend;
pub struct OpContains;
pub struct OpMap;
pub struct OpFilter;
pub struct OpFold;

impl Sealed for OpLen {}
impl Sealed for OpHead {}
impl Sealed for OpTail {}
impl Sealed for OpIsEmpty {}
impl Sealed for OpGet {}
impl Sealed for OpSet {}
impl Sealed for OpConcat {}
impl Sealed for OpAppend {}
impl Sealed for OpPrepend {}
impl Sealed for OpContains {}
impl Sealed for OpMap {}
impl Sealed for OpFilter {}
impl Sealed for OpFold {}

impl<Array> Apply<Array> for OpLen {
    type Output = ELen<Array>;
}
impl<Array> Apply<Array> for OpHead {
    type Output = EHead<Array>;
}
impl<Array> Apply<Array> for OpTail {
    type Output = ETail<Array>;
}
impl<Array> Apply<Array> for OpIsEmpty {
    type Output = EIsEmpty<Array>;
}

impl<Array, Idx> Apply<(Array, Idx)> for OpGet {
    type Output = EGet<Array, Idx>;
}
impl<Array, Idx, Val> Apply<(Array, Idx, Val)> for OpSet {
    type Output = ESet<Array, Idx, Val>;
}

impl<Lhs, Rhs> Apply<(Lhs, Rhs)> for OpConcat {
    type Output = EConcat<Lhs, Rhs>;
}
impl<Array, Elem> Apply<(Array, Elem)> for OpAppend {
    type Output = EAppend<Array, Elem>;
}
impl<Elem, Array> Apply<(Elem, Array)> for OpPrepend {
    type Output = EPrepend<Elem, Array>;
}
impl<Array, Elem> Apply<(Array, Elem)> for OpContains {
    type Output = EContains<Array, Elem>;
}

impl<Op, List> Apply<(Op, List)> for OpMap {
    type Output = EMap<Op, List>;
}
impl<Pred, List> Apply<(Pred, List)> for OpFilter {
    type Output = EFilter<Pred, List>;
}
impl<Op, Init, List> Apply<(Op, Init, List)> for OpFold {
    type Output = EFold<Op, Init, List>;
}

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
        std::{
            bool::{ToTyBoolOut, TyFalse, TyTrue},
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
}
