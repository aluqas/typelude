//! **Arithmetic Operations**
//!
//! Generic arithmetic operations (Add, Sub, etc.) defined via `define_arith_op!`.

use crate::std::traits::{TypeAdd, TypeDiv, TypeMul, TypePow, TypeRem, TypeSub};

// Note: Pass the *Type* trait here, not the typenum trait
define_arith_op!(Add, TypeAdd, "Addition: A + B");
define_arith_op!(Sub, TypeSub, "Subtraction: A - B");
define_arith_op!(Mul, TypeMul, "Multiplication: A * B");
define_arith_op!(Div, TypeDiv, "Division: A / B");
define_arith_op!(Rem, TypeRem, "Remainder: A % B");
define_arith_op!(Pow, TypePow, "Exponentiation: A ^ B");
