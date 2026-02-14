// Unstable features required for type-level programming
#![cfg_attr(feature = "nightly", feature(specialization))]
#![cfg_attr(feature = "nightly", feature(generic_const_exprs))]
#![allow(incomplete_features)]
#![recursion_limit = "65536"]

extern crate self as typelude_std;

#[macro_use]
pub mod macros;

pub mod expr;
pub mod lambda;
pub mod model;
pub mod std;

// Export typenum for macros
pub use paste;
// Re-export macros crate for use in macros.rs
pub use typelude_macros;

pub mod core;
// pub use typelude_std::core::{self, Eval, Evaluate};
pub use core::{Eval, Evaluate, TyFn};

pub use typenum;
