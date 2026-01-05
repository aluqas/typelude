//! **Logical Operations**
//!
//! Generic logical operations (And, Or, Not, etc.).

use crate::std::traits::Bool;

// Define Unary: OpNot
define_unary_logic_op!(Not, Bool, Not, "Logical NOT");

// Define Binary: OpAnd, OpOr, etc.
define_logic_op!(And, Bool, And, "Logical AND");
define_logic_op!(Or, Bool, Or, "Logical OR");
define_logic_op!(Nand, Bool, Nand, "Logical NAND");
define_logic_op!(Nor, Bool, Nor, "Logical NOR");
define_logic_op!(Xor, Bool, Xor, "Logical XOR");
define_logic_op!(Xnor, Bool, Xnor, "Logical XNOR");
