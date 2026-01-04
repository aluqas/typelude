// Unstable features required for type-level programming
#![feature(specialization)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
#![recursion_limit = "16384"]

#[macro_use]
pub mod macros;

pub mod eval;
pub mod kernel;
pub mod lambda;
pub mod std;

// Export typenum for macros
pub use typenum;
pub use paste;

pub use crate::{
    eval::{Eval, Evaluate},
    kernel::traits::Apply,
};
