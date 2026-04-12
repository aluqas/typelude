//! 型レベル同一型配列と要素操作。
//!
//! TArr<Value, Tail> は再帰的に構成される型レベル配列。
//! TTerm は配列の終端を表す。tarr!マクロでリスト的記法が利用可能。
//!
//! ## 操作トレイト
//!
//! 手法 Len（長さ）、Head（先頭）、Tail（尾部）、Get（インデックスアクセス）、
//! Set（要素更新）、Concat（連結）が実装される。いずれもペアノ数無しの
//! 再帰的定義により、型レベルで計算されたあと出力型が確定する。

use core::marker::PhantomData;

use typelude_num::{
    B1, Sub1, UInt, UTerm, Unsigned,
    peano::{Nat, Succ, Zero},
};
use typelude_std::{
    core::{Append as ColAppend, Concat, Get, Head, Len, Prepend, Set, Tail as TlTail, Value},
    effect::Append as FxAppend,
};

/// 型レベル同一型配列。
///
/// `TArr<Val, A>` は先頭要素 `Val` と尾部 `A` からなる配列セル。
/// Val は任意の型（リテラル、計算結果等）、A は次のセル（通常は TArr か
/// TTerm）。
#[derive(Debug, Default)]
pub struct TArr<Val, A>(PhantomData<(Val, A)>);

/// 型レベル配列の終端マーカー。
///
/// 空配列または配列の末端を表す值が無い型。Len により Zero を返す。
#[derive(Debug, Default)]
pub struct TTerm;

impl<Val, A> Value for TArr<Val, A> {}
impl Value for TTerm {}

#[macro_export]
macro_rules! tarr {
    () => ( $crate::TTerm );
    ($n:ty) => ( $crate::TArr<$n, $crate::TTerm> );
    ($n:ty,) => ( $crate::TArr<$n, $crate::TTerm> );
    ($n:ty, $($tail:ty),+) => ( $crate::TArr<$n, tarr![$($tail),+]> );
    ($n:ty, $($tail:ty),+,) => ( $crate::TArr<$n, tarr![$($tail),+]> );
    ($n:ty | $rest:ty) => ( $crate::TArr<$n, $rest> );
    ($n:ty, $($tail:ty),+ | $rest:ty) => ( $crate::TArr<$n, tarr![$($tail),+ | $rest]> );
}

impl Len for TTerm {
    type Output = Zero;
}

impl<Val, A> Len for TArr<Val, A>
where
    A: Len,
{
    type Output = Succ<<A as Len>::Output>;
}

impl<Val, A> Head for TArr<Val, A> {
    type Output = Val;
}

impl<Val, A> TlTail for TArr<Val, A> {
    type Output = A;
}

impl<HeadVal, Tail> Get<Zero> for TArr<HeadVal, Tail> {
    type Output = HeadVal;
}

impl<HeadVal, Tail, Idx: Nat> Get<Succ<Idx>> for TArr<HeadVal, Tail>
where
    Tail: Get<Idx>,
{
    type Output = <Tail as Get<Idx>>::Output;
}

impl<HeadVal, Tail, Val> Set<Zero, Val> for TArr<HeadVal, Tail> {
    type Output = TArr<Val, Tail>;
}

impl<HeadVal, Tail, Val, Idx: Nat> Set<Succ<Idx>, Val> for TArr<HeadVal, Tail>
where
    Tail: Set<Idx, Val>,
{
    type Output = TArr<HeadVal, <Tail as Set<Idx, Val>>::Output>;
}

impl<Other> Concat<Other> for TTerm {
    type Output = Other;
}

impl<HeadVal, Tail, Other> Concat<Other> for TArr<HeadVal, Tail>
where
    Tail: Concat<Other>,
{
    type Output = TArr<HeadVal, <Tail as Concat<Other>>::Output>;
}

impl<Elem> ColAppend<Elem> for TTerm {
    type Output = TArr<Elem, TTerm>;
}

impl<HeadVal, Tail, Elem> ColAppend<Elem> for TArr<HeadVal, Tail>
where
    Tail: ColAppend<Elem>,
{
    type Output = TArr<HeadVal, <Tail as ColAppend<Elem>>::Output>;
}

impl<Elem> Prepend<Elem> for TTerm {
    type Output = TArr<Elem, TTerm>;
}

impl<HeadVal, Tail, Elem> Prepend<Elem> for TArr<HeadVal, Tail> {
    type Output = TArr<Elem, TArr<HeadVal, Tail>>;
}

impl<HeadVal, Tail> Get<UTerm> for TArr<HeadVal, Tail> {
    type Output = HeadVal;
}

impl<HeadVal, Tail, N, B> Get<UInt<N, B>> for TArr<HeadVal, Tail>
where
    UInt<N, B>: core::ops::Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    Tail: Get<Sub1<UInt<N, B>>>,
{
    type Output = <Tail as Get<Sub1<UInt<N, B>>>>::Output;
}

impl<HeadVal, Tail, Val> Set<UTerm, Val> for TArr<HeadVal, Tail> {
    type Output = TArr<Val, Tail>;
}

impl<HeadVal, Tail, Val, N, B> Set<UInt<N, B>, Val> for TArr<HeadVal, Tail>
where
    UInt<N, B>: core::ops::Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    Tail: Set<Sub1<UInt<N, B>>, Val>,
{
    type Output = TArr<HeadVal, <Tail as Set<Sub1<UInt<N, B>>, Val>>::Output>;
}

impl<Rhs> FxAppend<Rhs> for TTerm {
    type Output = Rhs;
}

impl<HeadVal, Tail, Rhs> FxAppend<Rhs> for TArr<HeadVal, Tail>
where
    Tail: FxAppend<Rhs>,
{
    type Output = TArr<HeadVal, <Tail as FxAppend<Rhs>>::Output>;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_num::{
        U0, U1,
        peano::{Succ, Zero},
    };
    use typelude_std::{
        core::{
            Append, Apply, Concat, Evaluate, Get, Head, Len, OpConcat, OpGet, OpHead, OpLen,
            OpSet, Prepend, Set, Tail, Value,
        },
        effect::Append as FxAppend,
    };

    use super::{TArr, TTerm};

    struct U8Ty;
    struct U16Ty;
    struct U32Ty;
    struct I8Ty;
    struct I16Ty;
    struct I64Ty;

    impl Value for U8Ty {}
    impl Value for U16Ty {}
    impl Value for U32Ty {}
    impl Value for I8Ty {}
    impl Value for I16Ty {}
    impl Value for I64Ty {}

    type Arr = TArr<U8Ty, TArr<U16Ty, TArr<U32Ty, TTerm>>>;
    type Arr2 = TArr<I8Ty, TArr<I16Ty, TTerm>>;
    type N0 = Zero;
    type N1 = Succ<Zero>;
    type N2 = Succ<N1>;
    type N3 = Succ<N2>;

    #[test]
    fn test_len() {
        assert_type_eq_all!(<Arr as Len>::Output, N3);
        assert_type_eq_all!(Evaluate<Apply<OpLen, Arr>>, N3);
    }

    #[test]
    fn test_head_and_tail() {
        assert_type_eq_all!(<Arr as Head>::Output, U8Ty);
        assert_type_eq_all!(<Arr as Tail>::Output, TArr<U16Ty, TArr<U32Ty, TTerm>>);
        assert_type_eq_all!(Evaluate<Apply<OpHead, Arr>>, U8Ty);
    }

    #[test]
    fn test_get() {
        assert_type_eq_all!(<Arr as Get<N0>>::Output, U8Ty);
        assert_type_eq_all!(<Arr as Get<N1>>::Output, U16Ty);
        assert_type_eq_all!(<Arr as Get<N2>>::Output, U32Ty);
        assert_type_eq_all!(<Arr as Get<U0>>::Output, U8Ty);
        assert_type_eq_all!(<Arr as Get<U1>>::Output, U16Ty);
        assert_type_eq_all!(Evaluate<Apply<OpGet, (Arr, N1)>>, U16Ty);
        // assert_type_eq_all!(Evaluate<Apply<OpGet, (Arr, U1)>>, U16Ty);
    }

    #[test]
    fn test_set() {
        type NewArr = <Arr as Set<N1, I16Ty>>::Output;

        assert_type_eq_all!(<NewArr as Get<N0>>::Output, U8Ty);
        assert_type_eq_all!(<NewArr as Get<N1>>::Output, I16Ty);
        assert_type_eq_all!(<NewArr as Get<N2>>::Output, U32Ty);
        assert_type_eq_all!(Evaluate<Apply<OpSet, (Arr, N1, I16Ty)>>, NewArr);
        assert_type_eq_all!(<Arr as Set<U1, I16Ty>>::Output, NewArr);
        // assert_type_eq_all!(Evaluate<Apply<OpSet, (Arr, U1, I16Ty)>>,
        // NewArr);
    }

    #[test]
    fn test_concat() {
        type Joined = TArr<U8Ty, TArr<U16Ty, TArr<U32Ty, TArr<I8Ty, TArr<I16Ty, TTerm>>>>>;

        assert_type_eq_all!(<Arr as Concat<Arr2>>::Output, Joined);
        assert_type_eq_all!(Evaluate<Apply<OpConcat, (Arr, Arr2)>>, Joined);
    }

    #[test]
    fn test_append_and_prepend() {
        type Appended = TArr<U8Ty, TArr<U16Ty, TArr<U32Ty, TArr<I64Ty, TTerm>>>>;
        type Prepended = TArr<I64Ty, Arr>;

        assert_type_eq_all!(<Arr as Append<I64Ty>>::Output, Appended);
        assert_type_eq_all!(<Arr as Prepend<I64Ty>>::Output, Prepended);
    }

    #[test]
    fn test_effect_append_concat() {
        type Joined = <Arr as FxAppend<Arr2>>::Output;
        type Expected = TArr<U8Ty, TArr<U16Ty, TArr<U32Ty, TArr<I8Ty, TArr<I16Ty, TTerm>>>>>;

        assert_type_eq_all!(Joined, Expected);
    }
}
