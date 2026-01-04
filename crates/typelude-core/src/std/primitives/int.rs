//! **Type-Level Integer**
//!
//! Integration with the `typenum` crate and integer arithmetic.

use std::{
    ops::{Add, Div, Mul, Rem, Sub},
};

use typenum::{Pow};

use crate::{
    std::traits::{TypeAdd, TypeDiv, TypeMul, TypeNat, TypePow, TypeRem, TypeSub},
};

// Eval implementation is now in typelude-kernel (via impls.rs)
// We just define the Traits and Adapter Logic here.

// =============================================================================
// Adapter Implementation: TypeNat / TypeInt for typenum
// =============================================================================

// Implement TypeNat for UTerm and UInt
impl TypeNat for typenum::UTerm {}
impl<U, B> TypeNat for typenum::UInt<U, B> {}

// Implement TypeAdd, etc. for any typenum type that implements the typenum traits
// Note: We use blanket implementations where possible, or specific ones if needed to avoid conflict.
// typenum implements Add for almost everything (UInt, Z0, PInt, NInt).

impl<L, R> TypeAdd<R> for L
where
    L: Add<R>,
{
    type Output = <L as Add<R>>::Output;
}

impl<L, R> TypeSub<R> for L
where
    L: Sub<R>,
{
    type Output = <L as Sub<R>>::Output;
}

impl<L, R> TypeMul<R> for L
where
    L: Mul<R>,
{
    type Output = <L as Mul<R>>::Output;
}

impl<L, R> TypeDiv<R> for L
where
    L: Div<R>,
{
    type Output = <L as Div<R>>::Output;
}

impl<L, R> TypeRem<R> for L
where
    L: Rem<R>,
{
    type Output = <L as Rem<R>>::Output;
}

impl<L, R> TypePow<R> for L
where
    L: Pow<R>,
{
    type Output = <L as Pow<R>>::Output;
}

//
// Tests (Only for implementation validity)
//

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typenum::{N2, P5, U1, U3, B0, B1};

    use crate::{eval::{Evaluate, ELit}, std::ops::EAdd};

    #[test]
    fn test_eval_typenum() {
        // Since typenum types are implemented in kernel, they should evaluate to themselves.
        assert_type_eq_all!(Evaluate<U1>, U1);
        assert_type_eq_all!(Evaluate<P5>, P5);
        assert_type_eq_all!(Evaluate<N2>, N2);
        assert_type_eq_all!(Evaluate<B0>, B0);
        assert_type_eq_all!(Evaluate<B1>, B1);
    }

    #[test]
    fn test_add() {
        // EAdd uses TypeAdd, which is implemented for U1.
        // EAdd requires arguments to be Eval. U1 is Eval.
        // So this should work.
        assert_type_eq_all!(Evaluate<EAdd<ELit<U1>, ELit<typenum::U2>>>, U3);
        // Also direct if U1 is Eval?
        assert_type_eq_all!(Evaluate<EAdd<U1, typenum::U2>>, U3);
    }
}
