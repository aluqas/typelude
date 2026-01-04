pub mod arith;
pub mod cmp;
pub mod into;
pub mod logic;

pub use arith::*;
pub use cmp::*;
pub use into::*;
pub use logic::*;

#[cfg(test)]
mod def_op_tests;
