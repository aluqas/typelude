//! Generic adapters from shared capability traits to canonical `Op*` markers.

use super::{
    Add, And, Append, Concat, Div, Eq, Evaluate, Fold, Get, Gt, Head, Into, Le, Len, Lt, Map, Mul,
    Nand, Neq, Not, Op, OpAdd, OpAnd, OpAppend, OpConcat, OpDiv, OpEq, OpFold, OpGet, OpGt,
    OpHead, OpIf, OpInto, OpLe, OpLen, OpLt, OpMap, OpMul, OpNand, OpNeq, OpNot, OpOr, OpPow,
    OpPrepend, OpRem, OpSet, OpSub, OpTail, OpWhile, OpXor, Or, Pow, Prepend, Rem, Set, Sub, Tail,
    Xor,
};
use crate::control::{If, While};

impl<Arg> Op<Arg> for OpNot
where
    Arg: Not,
{
    type Output = <Arg as Not>::Output;
}

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpAnd
where
    Lhs: And<Rhs>,
{
    type Output = <Lhs as And<Rhs>>::Output;
}

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpOr
where
    Lhs: Or<Rhs>,
{
    type Output = <Lhs as Or<Rhs>>::Output;
}

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpXor
where
    Lhs: Xor<Rhs>,
{
    type Output = <Lhs as Xor<Rhs>>::Output;
}

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpNand
where
    Lhs: Nand<Rhs>,
{
    type Output = <Lhs as Nand<Rhs>>::Output;
}

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpAdd
where
    Lhs: Add<Rhs>,
{
    type Output = <Lhs as Add<Rhs>>::Output;
}

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpSub
where
    Lhs: Sub<Rhs>,
{
    type Output = <Lhs as Sub<Rhs>>::Output;
}

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpMul
where
    Lhs: Mul<Rhs>,
{
    type Output = <Lhs as Mul<Rhs>>::Output;
}

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpDiv
where
    Lhs: Div<Rhs>,
{
    type Output = <Lhs as Div<Rhs>>::Output;
}

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpRem
where
    Lhs: Rem<Rhs>,
{
    type Output = <Lhs as Rem<Rhs>>::Output;
}

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpPow
where
    Lhs: Pow<Rhs>,
{
    type Output = <Lhs as Pow<Rhs>>::Output;
}

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpEq
where
    Lhs: Eq<Rhs>,
{
    type Output = <Lhs as Eq<Rhs>>::Output;
}

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpNeq
where
    Lhs: Neq<Rhs>,
{
    type Output = <Lhs as Neq<Rhs>>::Output;
}

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpLt
where
    Lhs: Lt<Rhs>,
{
    type Output = <Lhs as Lt<Rhs>>::Output;
}

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpLe
where
    Lhs: Le<Rhs>,
{
    type Output = <Lhs as Le<Rhs>>::Output;
}

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpGt
where
    Lhs: Gt<Rhs>,
{
    type Output = <Lhs as Gt<Rhs>>::Output;
}

impl<Col> Op<Col> for OpLen
where
    Col: Len,
{
    type Output = <Col as Len>::Output;
}

impl<Col> Op<Col> for OpHead
where
    Col: Head,
{
    type Output = <Col as Head>::Output;
}

impl<Col> Op<Col> for OpTail
where
    Col: Tail,
{
    type Output = <Col as Tail>::Output;
}

impl<Col, Idx> Op<(Col, Idx)> for OpGet
where
    Col: Get<Idx>,
{
    type Output = <Col as Get<Idx>>::Output;
}

impl<Col, Idx, Val> Op<(Col, Idx, Val)> for OpSet
where
    Col: Set<Idx, Val>,
{
    type Output = <Col as Set<Idx, Val>>::Output;
}

impl<Lhs, Rhs> Op<(Lhs, Rhs)> for OpConcat
where
    Lhs: Concat<Rhs>,
{
    type Output = <Lhs as Concat<Rhs>>::Output;
}

impl<Col, Elem> Op<(Col, Elem)> for OpAppend
where
    Col: Append<Elem>,
{
    type Output = <Col as Append<Elem>>::Output;
}

impl<Elem, Col> Op<(Elem, Col)> for OpPrepend
where
    Col: Prepend<Elem>,
{
    type Output = <Col as Prepend<Elem>>::Output;
}

impl<Func, Col> Op<(Func, Col)> for OpMap
where
    Col: Map<Func>,
{
    type Output = <Col as Map<Func>>::Output;
}

impl<Func, Init, Col> Op<(Func, Init, Col)> for OpFold
where
    Col: Fold<Func, Init>,
{
    type Output = <Col as Fold<Func, Init>>::Output;
}

impl<Source, Target> Op<(Source, Target)> for OpInto
where
    Source: Into<Target>,
{
    type Output = <Source as Into<Target>>::Output;
}

impl<Cond, Then, Else> Op<(Cond, Then, Else)> for OpIf
where
    If<Cond, Then, Else>: super::Eval,
{
    type Output = Evaluate<If<Cond, Then, Else>>;
}

impl<Pred, Step, State> Op<(Pred, Step, State)> for OpWhile
where
    While<Pred, Step, State>: super::Eval,
{
    type Output = Evaluate<While<Pred, Step, State>>;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use crate::core::{
        Add, And, Append, Apply, Concat, Eq, Evaluate, Fold, From, Get, Gt, Len, Lt, Map, Not,
        OpAdd, OpAnd, OpAppend, OpConcat, OpEq, OpFold, OpGet, OpGt, OpInto, OpLen, OpLt, OpMap,
        OpNot, OpSet, Prepend, Set, Value,
    };

    struct A;
    struct B;
    struct C;
    struct D;
    struct Idx;
    struct Val;
    struct Col;
    struct Col2;
    struct Col3;
    struct Col4;
    struct Out;

    impl Value for A {}
    impl Value for B {}
    impl Value for C {}
    impl Value for D {}
    impl Value for Idx {}
    impl Value for Val {}
    impl Value for Col {}
    impl Value for Col2 {}
    impl Value for Col3 {}
    impl Value for Col4 {}
    impl Value for Out {}

    impl Not for A {
        type Output = B;
    }

    impl And<B> for A {
        type Output = C;
    }

    impl Add<B> for A {
        type Output = D;
    }

    impl Len for Col {
        type Output = A;
    }

    impl Eq<B> for A {
        type Output = C;
    }

    impl Lt<B> for A {
        type Output = D;
    }

    impl Gt<B> for A {
        type Output = Col;
    }

    impl Get<Idx> for Col {
        type Output = Val;
    }

    impl Set<Idx, Val> for Col {
        type Output = Col2;
    }

    impl Concat<Col2> for Col {
        type Output = Col3;
    }

    impl Append<Val> for Col {
        type Output = Col4;
    }

    impl Prepend<Idx> for Col {
        type Output = Col4;
    }

    impl Map<A> for Col {
        type Output = Col2;
    }

    impl Fold<A, B> for Col {
        type Output = D;
    }

    impl From<Col> for Out {
        type Output = Col3;
    }

    #[test]
    fn adapters_work_without_primitive_crate_dependencies() {
        assert_type_eq_all!(Evaluate<Apply<OpNot, A>>, B);
        assert_type_eq_all!(Evaluate<Apply<OpAnd, (A, B)>>, C);
        assert_type_eq_all!(Evaluate<Apply<OpAdd, (A, B)>>, D);
        assert_type_eq_all!(Evaluate<Apply<OpEq, (A, B)>>, C);
        assert_type_eq_all!(Evaluate<Apply<OpLt, (A, B)>>, D);
        assert_type_eq_all!(Evaluate<Apply<OpGt, (A, B)>>, Col);
        assert_type_eq_all!(Evaluate<Apply<OpLen, Col>>, A);
        assert_type_eq_all!(Evaluate<Apply<OpGet, (Col, Idx)>>, Val);
        assert_type_eq_all!(Evaluate<Apply<OpSet, (Col, Idx, Val)>>, Col2);
        assert_type_eq_all!(Evaluate<Apply<OpConcat, (Col, Col2)>>, Col3);
        assert_type_eq_all!(Evaluate<Apply<OpAppend, (Col, Val)>>, Col4);
        assert_type_eq_all!(Evaluate<Apply<OpMap, (A, Col)>>, Col2);
        assert_type_eq_all!(Evaluate<Apply<OpFold, (A, B, Col)>>, D);
        assert_type_eq_all!(Evaluate<Apply<OpInto, (Col, Out)>>, Col3);
    }
}
