//! Generic adapters from shared capability traits to canonical `Op*` markers.

use super::{
    Add, And, Concat, Div, Eq, Get, Gt, Head, Le, Len, Lt, Mul, Nand, Neq, Not, Op, OpAdd, OpAnd,
    OpConcat, OpDiv, OpEq, OpGet, OpGt, OpHead, OpLe, OpLen, OpLt, OpMul, OpNand, OpNeq, OpNot,
    OpOr, OpSet, OpSub, OpTail, OpXor, Or, Set, Sub, Tail, Xor,
};

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

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use crate::core::{
        Add, And, Apply, Concat, Eq, Evaluate, Get, Gt, Len, Lt, Not, OpAdd, OpAnd, OpConcat,
        OpEq, OpGet, OpGt, OpLen, OpLt, OpNot, OpSet, Set, Value,
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

    impl Value for A {}
    impl Value for B {}
    impl Value for C {}
    impl Value for D {}
    impl Value for Idx {}
    impl Value for Val {}
    impl Value for Col {}
    impl Value for Col2 {}
    impl Value for Col3 {}

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
    }
}
