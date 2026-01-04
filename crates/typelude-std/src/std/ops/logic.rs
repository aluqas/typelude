//! **Logical Operations**
//!
//! Generic logical operations (And, Or, Not, etc.).

use crate::std::traits::TypeBool;

// Define Unary: OpNot
define_unary_logic_op!(Not, TypeBool, Not, "Logical NOT");

// Define Binary: OpAnd, OpOr, etc.
define_logic_op!(And, TypeBool, And, "Logical AND");
define_logic_op!(Or, TypeBool, Or, "Logical OR");
define_logic_op!(Nand, TypeBool, Nand, "Logical NAND");
define_logic_op!(Nor, TypeBool, Nor, "Logical NOR");
define_logic_op!(Xor, TypeBool, Xor, "Logical XOR");
define_logic_op!(Xnor, TypeBool, Xnor, "Logical XNOR");
