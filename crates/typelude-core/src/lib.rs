// Unstable features required for type-level programming
#![feature(specialization)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
#![recursion_limit = "16384"]

pub mod eval;
pub mod kernel;
pub mod lambda;
pub mod std;

pub mod macros;

// Export typenum for macros
pub use typenum;

pub use crate::{
    eval::{Eval, Evaluate},
    kernel::traits::Apply,
};
