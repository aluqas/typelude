use ::std::marker::PhantomData;

use crate::{Add, Div, Mul, Sub};

struct Succ<T>(PhantomData<T>);
struct Zero;

trait Nat {
    const VAL: usize;
}

impl Nat for Zero {
    const VAL: usize = 0;
}

impl<T: Nat> Nat for Succ<T> {
    const VAL: usize = T::VAL + 1;
}

//
// Add
//

impl<M: Nat> Add<M, Zero> for () {
    type Output = M;
}

impl<M: Nat, N: Nat> Add<M, Succ<N>> for ()
where
    (): Add<M, N>,
{
    type Output = Succ<<() as Add<M, N>>::Output>;
}

//
// Mul
//

impl<M: Nat> Mul<M, Zero> for () {
    type Output = Zero;
}

impl<M: Nat, N: Nat> Mul<M, Succ<N>> for ()
where
    (): Mul<M, N> + Add<M, <() as Mul<M, N>>::Output>,
{
    type Output = <() as Add<M, <() as Mul<M, N>>::Output>>::Output;
}

//
// Pred
//

trait Pred<N> {
    type Output;
}

impl Pred<Zero> for () {
    type Output = Zero;
}

impl<N: Nat> Pred<Succ<N>> for () {
    type Output = N;
}

//
// Sub: 切り捨て
//

impl<M: Nat> Sub<M, Zero> for () {
    type Output = M;
}

impl<M: Nat, Rhs: Nat> Sub<M, Succ<Rhs>> for ()
where
    (): Sub<M, Rhs> + Pred<<() as Sub<M, Rhs>>::Output>,
{
    type Output = <() as Pred<<() as Sub<M, Rhs>>::Output>>::Output;
}

//
// Pow
//

trait Pow<M, N> {
    type Output;
}

impl<M: Nat> Pow<M, Zero> for () {
    type Output = Succ<Zero>;
}

impl<M: Nat, N: Nat> Pow<M, Succ<N>> for ()
where
    (): Pow<M, N> + Mul<M, <() as Pow<M, N>>::Output>,
{
    type Output = <() as Mul<M, <() as Pow<M, N>>::Output>>::Output;
}

#[cfg(test)]
mod tests {
    use super::*;

    type N0 = Zero;
    type N1 = Succ<Zero>;
    type N2 = Succ<Succ<Zero>>;
    type N3 = Succ<Succ<Succ<Zero>>>;
    type N4 = Succ<Succ<Succ<Succ<Zero>>>>;

    #[test]
    fn test_add() {
        assert_eq!(<() as Add<N2, N0>>::Output::VAL, 2);
        assert_eq!(<() as Add<N2, N3>>::Output::VAL, 5);
    }

    #[test]
    fn test_mul() {
        assert_eq!(<() as Mul<N2, N0>>::Output::VAL, 0);
        assert_eq!(<() as Mul<N2, N3>>::Output::VAL, 6);
    }

    #[test]
    fn test_sub() {
        assert_eq!(<() as Sub<N3, N0>>::Output::VAL, 3);
        assert_eq!(<() as Sub<N3, N1>>::Output::VAL, 2);
        assert_eq!(<() as Sub<N3, N3>>::Output::VAL, 0);
        assert_eq!(<() as Sub<N3, N4>>::Output::VAL, 0);
    }

    #[test]
    fn test_pow() {
        assert_eq!(<() as Pow<N2, N0>>::Output::VAL, 1);
        assert_eq!(<() as Pow<N2, N3>>::Output::VAL, 8);
        assert_eq!(<() as Pow<N4, N4>>::Output::VAL, 256);
    }
}
