//! **Type-Level Integer**
//!
//! Integration with the `typenum` crate and integer arithmetic.

use std::ops::{Add, Div, Mul, Rem, Sub};

use typenum::{B0, B1, NInt, PInt, Pow, UInt, UTerm, Unsigned, Z0};

use crate::{
    eval::{Eval, Evaluate, Sealed},
    kernel::traits::Apply,
};

//
// Eval Implementation for typenum Types
//

// --- Unsigned Integers ---

impl Eval for UTerm {
    type Output = Self;
}
impl<U, B> Eval for UInt<U, B> {
    type Output = Self;
}

// --- Signed Integers ---

impl Eval for Z0 {
    type Output = Self;
}
impl<U: Unsigned + typenum::NonZero> Eval for PInt<U> {
    type Output = Self;
}
impl<U: Unsigned + typenum::NonZero> Eval for NInt<U> {
    type Output = Self;
}

// --- Bits ---

impl Eval for B0 {
    type Output = Self;
}
impl Eval for B1 {
    type Output = Self;
}

//
// Arithmetic Functions
//

use std::marker::PhantomData;

use paste::paste;

macro_rules! define_arith_op {
    ($op_name:ident, $trait:path, $doc:literal) => {
        paste! {
            // Operator Symbol
            #[doc = $doc]
            pub struct [<Op $op_name>];
            impl Sealed for [<Op $op_name>] {}

            // Expression Struct
            pub struct [<E $op_name>]<Lhs, Rhs>(PhantomData<(Lhs, Rhs)>);

            impl<Lhs, Rhs> Eval for [<E $op_name>]<Lhs, Rhs>
            where
                Lhs: Eval,
                Rhs: Eval,
                Evaluate<Lhs>: $trait<Evaluate<Rhs>>,
                <Evaluate<Lhs> as $trait<Evaluate<Rhs>>>::Output: Eval, // Ensure result is well-formed (usually is)
            {
                type Output = <Evaluate<Lhs> as $trait<Evaluate<Rhs>>>::Output;
            }

            // Apply Implementation
            impl<Lhs, Rhs> Apply<(Lhs, Rhs)> for [<Op $op_name>] {
                type Output = [<E $op_name>]<Lhs, Rhs>;
            }
        }
    };
}

define_arith_op!(Add, Add, "Addition: A + B");
define_arith_op!(Sub, Sub, "Subtraction: A - B");
define_arith_op!(Mul, Mul, "Multiplication: A * B");
define_arith_op!(Div, Div, "Division: A / B");
define_arith_op!(Rem, Rem, "Remainder: A % B");
define_arith_op!(Pow, Pow, "Exponentiation: A ^ B");

//
// Tests
//

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typenum::{N1, N2, P1, P4, P5, U1, U2, U3, U5, U8, U9, U10, U20};

    use super::*;

    #[test]
    fn test_eval_typenum() {
        assert_type_eq_all!(Evaluate<U1>, U1);
        assert_type_eq_all!(Evaluate<P5>, P5);
        assert_type_eq_all!(Evaluate<N2>, N2);
        assert_type_eq_all!(Evaluate<B0>, B0);
        assert_type_eq_all!(Evaluate<B1>, B1);
    }

    #[test]
    fn test_add() {
        assert_type_eq_all!(Evaluate<EAdd<U1, U2>>, U3);
        assert_type_eq_all!(Evaluate<EAdd<P1, N2>>, N1);
    }

    #[test]
    fn test_sub() {
        assert_type_eq_all!(Evaluate<ESub<U10, U2>>, U8);
        assert_type_eq_all!(Evaluate<ESub<N1, P1>>, N2);
    }

    #[test]
    fn test_mul() {
        assert_type_eq_all!(Evaluate<EMul<U2, U10>>, U20);
        assert_type_eq_all!(Evaluate<EMul<N2, N2>>, P4);
    }

    #[test]
    fn test_div() {
        assert_type_eq_all!(Evaluate<EDiv<U10, U2>>, U5);
    }

    #[test]
    fn test_rem() {
        assert_type_eq_all!(Evaluate<ERem<U10, U3>>, U1);
    }

    #[test]
    fn test_pow() {
        assert_type_eq_all!(Evaluate<EPow<U2, U3>>, U8);
    }

    #[test]
    fn test_composition() {
        // (1 + 2) * 3 = 9
        assert_type_eq_all!(Evaluate<EMul<EAdd<U1, U2>, U3>>, U9);
    }
}
