use core::marker::PhantomData;

use typelude_std::core::{Add, Div, Mul, Sub, Value};

pub struct Succ<T>(PhantomData<T>);
pub struct Zero;

impl<T> Value for Succ<T> {}
impl Value for Zero {}

pub trait Nat {
    const VAL: usize;
}

impl Nat for Zero {
    const VAL: usize = 0;
}

impl<T: Nat> Nat for Succ<T> {
    const VAL: usize = T::VAL + 1;
}


impl<Rhs: Nat> Add<Rhs> for Zero {
    type Output = Rhs;
}

impl<Lhs: Nat, Rhs: Nat> Add<Rhs> for Succ<Lhs>
where
    Lhs: Add<Rhs>,
{
    type Output = Succ<<Lhs as Add<Rhs>>::Output>;
}

impl<Rhs: Nat> Mul<Rhs> for Zero {
    type Output = Zero;
}

impl<Lhs: Nat, Rhs: Nat> Mul<Rhs> for Succ<Lhs>
where
    Lhs: Mul<Rhs>,
    Rhs: Add<<Lhs as Mul<Rhs>>::Output>,
{
    type Output = <Rhs as Add<<Lhs as Mul<Rhs>>::Output>>::Output;
}

impl<Rhs: Nat> Sub<Rhs> for Zero {
    type Output = Zero;
}

impl<Lhs: Nat> Sub<Zero> for Succ<Lhs> {
    type Output = Succ<Lhs>;
}

impl<Lhs: Nat, Rhs: Nat> Sub<Succ<Rhs>> for Succ<Lhs>
where
    Lhs: Sub<Rhs>,
{
    type Output = <Lhs as Sub<Rhs>>::Output;
}

#[doc(hidden)]
pub struct CanSubYes;
#[doc(hidden)]
pub struct CanSubNo;

#[doc(hidden)]
pub trait CanSub<Rhs> {
    type Output;
}

impl CanSub<Zero> for Zero {
    type Output = CanSubYes;
}

impl<Rhs: Nat> CanSub<Succ<Rhs>> for Zero {
    type Output = CanSubNo;
}

impl<Lhs: Nat> CanSub<Zero> for Succ<Lhs> {
    type Output = CanSubYes;
}

impl<Lhs: Nat, Rhs: Nat> CanSub<Succ<Rhs>> for Succ<Lhs>
where
    Lhs: CanSub<Rhs>,
{
    type Output = <Lhs as CanSub<Rhs>>::Output;
}

#[doc(hidden)]
pub trait DivStep<Divisor, Flag> {
    type Output;
}

impl<Dividend: Nat, Divisor: Nat> DivStep<Divisor, CanSubNo> for Dividend {
    type Output = Zero;
}

impl<Dividend: Nat, Divisor: Nat> DivStep<Divisor, CanSubYes> for Dividend
where
    Dividend: Sub<Succ<Divisor>>,
    <Dividend as Sub<Succ<Divisor>>>::Output: Div<Succ<Divisor>>,
{
    type Output = Succ<<<Dividend as Sub<Succ<Divisor>>>::Output as Div<Succ<Divisor>>>::Output>;
}

impl<Divisor: Nat> Div<Succ<Divisor>> for Zero {
    type Output = Zero;
}

impl<Dividend: Nat, Divisor: Nat> Div<Succ<Divisor>> for Succ<Dividend>
where
    Succ<Dividend>: CanSub<Succ<Divisor>>,
    Succ<Dividend>: DivStep<Divisor, <Succ<Dividend> as CanSub<Succ<Divisor>>>::Output>,
{
    type Output = <Succ<Dividend> as DivStep<
        Divisor,
        <Succ<Dividend> as CanSub<Succ<Divisor>>>::Output,
    >>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_std::core::{Apply, Evaluate, OpAdd, OpDiv, OpMul, OpSub};

    use super::*;

    type N0 = Zero;
    type N1 = Succ<Zero>;
    type N2 = Succ<Succ<Zero>>;
    type N3 = Succ<Succ<Succ<Zero>>>;
    type N4 = Succ<Succ<Succ<Succ<Zero>>>>;
    type N5 = Succ<N4>;
    type N6 = Succ<N5>;

    #[test]
    fn test_add() {
        assert_eq!(<N2 as Add<N0>>::Output::VAL, 2);
        assert_eq!(<N2 as Add<N3>>::Output::VAL, 5);
        assert_type_eq_all!(Evaluate<Apply<OpAdd, (N2, N3)>>, N5);
    }

    #[test]
    fn test_mul() {
        assert_eq!(<N2 as Mul<N0>>::Output::VAL, 0);
        assert_eq!(<N2 as Mul<N3>>::Output::VAL, 6);
        assert_type_eq_all!(Evaluate<Apply<OpMul, (N2, N3)>>, N6);
    }

    #[test]
    fn test_sub() {
        assert_eq!(<N3 as Sub<N0>>::Output::VAL, 3);
        assert_eq!(<N3 as Sub<N1>>::Output::VAL, 2);
        assert_eq!(<N3 as Sub<N3>>::Output::VAL, 0);
        assert_eq!(<N3 as Sub<N4>>::Output::VAL, 0);
        assert_type_eq_all!(Evaluate<Apply<OpSub, (N3, N1)>>, N2);
    }

    #[test]
    fn test_div() {
        assert_eq!(<N6 as Div<N2>>::Output::VAL, 3);
        assert_eq!(<N5 as Div<N2>>::Output::VAL, 2);
        assert_type_eq_all!(Evaluate<Apply<OpDiv, (N6, N2)>>, N3);
    }
}
