// Unstable features required for type-level programming
#![cfg_attr(feature = "nightly", feature(specialization))]
#![cfg_attr(feature = "nightly", feature(generic_const_exprs))]
#![allow(incomplete_features)]
#![recursion_limit = "16384"]

#[macro_use]
pub mod macros;

pub mod eval;
pub mod lambda;
pub mod model;
pub mod std;

// Export typenum for macros
pub use paste;
pub use typenum;

pub use crate::eval::{Eval, Evaluate};
