// Unstable features required for type-level programming
#![cfg_attr(feature = "nightly", feature(specialization))]
#![cfg_attr(feature = "nightly", feature(generic_const_exprs))]
#![allow(incomplete_features)]
#![recursion_limit = "16384"]

#[macro_use]
pub mod macros;

pub mod eval;
mod kernel; // Private
pub mod lambda;
pub mod std;

// Export typenum for macros
pub use typenum;
pub use paste;

pub use crate::{
    eval::{Eval, Evaluate},
    kernel::traits::Apply,
};
