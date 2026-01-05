//! **Arithmetic Operations**
//!
//! Generic arithmetic operations (Add, Sub, etc.) defined via
//! `define_arith_op!`.

use crate::std::traits::{TAdd, TDiv, TMul, TPow, TRem, TSub};

// Note: Pass the *Type* trait here, not the typenum trait
define_arith_op!(Add, TAdd, "Addition: A + B");
define_arith_op!(Sub, TSub, "Subtraction: A - B");
define_arith_op!(Mul, TMul, "Multiplication: A * B");
define_arith_op!(Div, TDiv, "Division: A / B");
define_arith_op!(Rem, TRem, "Remainder: A % B");
define_arith_op!(Pow, TPow, "Exponentiation: A ^ B");
