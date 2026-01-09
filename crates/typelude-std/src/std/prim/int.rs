//! **Type-Level Integer**
//!
//! Integration with the `typenum` crate and integer arithmetic.

use std::ops::{Add, Div, Mul, Rem, Sub};

use typenum::Pow;

pub use crate::model::prim::int::*;
use crate::std::traits::{Nat, TAdd, TDiv, TMul, TPow, TRem, TSub};

// Eval implementation is now in typelude-kernel (via impls.rs)
// We just define the Traits and Adapter Logic here.

// Implement Nat for UTerm and UInt
impl Nat for typenum::UTerm {}
impl<U, B> Nat for typenum::UInt<U, B> {}

// Implement TAdd, etc. for any typenum type that implements the typenum traits
// Note: We use blanket implementations where possible, or specific ones if
// needed to avoid conflict. typenum implements Add for almost everything (UInt,
// Z0, PInt, NInt).

impl<L, R> TAdd<R> for L
where
    L: Add<R>,
{
    type Output = <L as Add<R>>::Output;
}

impl<L, R> TSub<R> for L
where
    L: Sub<R>,
{
    type Output = <L as Sub<R>>::Output;
}

impl<L, R> TMul<R> for L
where
    L: Mul<R>,
{
    type Output = <L as Mul<R>>::Output;
}

impl<L, R> TDiv<R> for L
where
    L: Div<R>,
{
    type Output = <L as Div<R>>::Output;
}

impl<L, R> TRem<R> for L
where
    L: Rem<R>,
{
    type Output = <L as Rem<R>>::Output;
}

impl<L, R> TPow<R> for L
where
    L: Pow<R>,
{
    type Output = <L as Pow<R>>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_core::{ELit, Evaluate};
    use typenum::{B0, B1, N2, P5, U1, U3};

    use crate::std::ops::EAdd;

    #[test]
    fn test_eval_lit() {
        assert_type_eq_all!(Evaluate<ELit<U1>>, U1);
        assert_type_eq_all!(Evaluate<ELit<P5>>, P5);
        assert_type_eq_all!(Evaluate<ELit<N2>>, N2);
        assert_type_eq_all!(Evaluate<ELit<B0>>, B0);
        assert_type_eq_all!(Evaluate<ELit<B1>>, B1);
    }

    #[test]
    fn test_add() {
        // EAdd uses TypeAdd, which is implemented for U1.
        // EAdd requires arguments to be Eval. ELit<U1> is Eval.
        // So this should work.
        assert_type_eq_all!(Evaluate<EAdd<ELit<U1>, ELit<typenum::U2>>>, U3);
    }
}
