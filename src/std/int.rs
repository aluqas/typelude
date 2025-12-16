//! **Type-Level Integer**
//!
//! Integration with the `typenum` crate and integer arithmetic.

use std::ops::{Add, Div, Mul, Rem, Sub};

use typenum::{B0, B1, NInt, PInt, Pow, UInt, UTerm, Unsigned, Z0};

use crate::eval::{EApply2, Evaluable, Evaluator, Sealed};

//
// Evaluable Implementation for typenum Types
//

// --- Unsigned Integers ---

impl Evaluable for UTerm {
    type Output = Self;
}

impl<U, B> Evaluable for UInt<U, B> {
    type Output = Self;
}

// --- Signed Integers ---

impl Evaluable for Z0 {
    type Output = Self;
}

impl<U: Unsigned + typenum::NonZero> Evaluable for PInt<U> {
    type Output = Self;
}

impl<U: Unsigned + typenum::NonZero> Evaluable for NInt<U> {
    type Output = Self;
}

// --- Bits ---

impl Evaluable for B0 {
    type Output = Self;
}

impl Evaluable for B1 {
    type Output = Self;
}

//
// Arithmetic Functions
//

use paste::paste;

//
// Arithmetic Functions
//

macro_rules! define_arith_op {
    ($op_name:ident, $trait:path, $doc:literal) => {
        paste! {
            #[doc = $doc]
            pub struct [<F $op_name>];
            impl Sealed for [<F $op_name>] {}

            impl<Lhs, Rhs> Evaluable for EApply2<[<F $op_name>], Lhs, Rhs>
            where
                Lhs: Evaluable,
                Rhs: Evaluable,
                Evaluator<Lhs>: $trait<Evaluator<Rhs>>,
                <Evaluator<Lhs> as $trait<Evaluator<Rhs>>>::Output: Evaluable,
            {
                type Output = <Evaluator<Lhs> as $trait<Evaluator<Rhs>>>::Output;
            }

            // Aliases
            pub type [<E $op_name>]<Lhs, Rhs> = EApply2<[<F $op_name>], Lhs, Rhs>;
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
        assert_type_eq_all!(Evaluator<U1>, U1);
        assert_type_eq_all!(Evaluator<P5>, P5);
        assert_type_eq_all!(Evaluator<N2>, N2);
        assert_type_eq_all!(Evaluator<B0>, B0);
        assert_type_eq_all!(Evaluator<B1>, B1);
    }

    #[test]
    fn test_add() {
        assert_type_eq_all!(Evaluator<EAdd<U1, U2>>, U3);
        assert_type_eq_all!(Evaluator<EAdd<P1, N2>>, N1);
    }

    #[test]
    fn test_sub() {
        assert_type_eq_all!(Evaluator<ESub<U10, U2>>, U8);
        assert_type_eq_all!(Evaluator<ESub<N1, P1>>, N2);
    }

    #[test]
    fn test_mul() {
        assert_type_eq_all!(Evaluator<EMul<U2, U10>>, U20);
        assert_type_eq_all!(Evaluator<EMul<N2, N2>>, P4);
    }

    #[test]
    fn test_div() {
        assert_type_eq_all!(Evaluator<EDiv<U10, U2>>, U5);
    }

    #[test]
    fn test_rem() {
        assert_type_eq_all!(Evaluator<ERem<U10, U3>>, U1);
    }

    #[test]
    fn test_pow() {
        assert_type_eq_all!(Evaluator<EPow<U2, U3>>, U8);
    }

    #[test]
    fn test_composition() {
        // (1 + 2) * 3 = 9
        assert_type_eq_all!(Evaluator<EMul<EAdd<U1, U2>, U3>>, U9);
    }
}
