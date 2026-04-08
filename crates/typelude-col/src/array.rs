use core::marker::PhantomData;

use typelude_num::peano::{Nat, Succ, Zero};
use typelude_std::core::{
    Append, Concat, Get, Head, Len, Prepend, Set, Tail as TlTail, Value,
};

pub struct TArr<Val, A>(PhantomData<(Val, A)>);
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

impl<Elem> Append<Elem> for TTerm {
    type Output = TArr<Elem, TTerm>;
}

impl<HeadVal, Tail, Elem> Append<Elem> for TArr<HeadVal, Tail>
where
    Tail: Append<Elem>,
{
    type Output = TArr<HeadVal, <Tail as Append<Elem>>::Output>;
}

impl<Elem> Prepend<Elem> for TTerm {
    type Output = TArr<Elem, TTerm>;
}

impl<HeadVal, Tail, Elem> Prepend<Elem> for TArr<HeadVal, Tail> {
    type Output = TArr<Elem, TArr<HeadVal, Tail>>;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_std::core::{
        Append, Apply, Concat, Evaluate, Get, Head, Len, OpConcat, OpGet, OpHead, OpLen, OpSet,
        Prepend, Set, Tail, Value,
    };

    use super::{TArr, TTerm};
    use typelude_num::peano::{Succ, Zero};

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
        assert_type_eq_all!(Evaluate<Apply<OpGet, (Arr, N1)>>, U16Ty);
    }

    #[test]
    fn test_set() {
        type NewArr = <Arr as Set<N1, I16Ty>>::Output;

        assert_type_eq_all!(<NewArr as Get<N0>>::Output, U8Ty);
        assert_type_eq_all!(<NewArr as Get<N1>>::Output, I16Ty);
        assert_type_eq_all!(<NewArr as Get<N2>>::Output, U32Ty);
        assert_type_eq_all!(Evaluate<Apply<OpSet, (Arr, N1, I16Ty)>>, NewArr);
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
}
