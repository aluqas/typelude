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
// Re-export traits at crate root for macro compatibility
pub use std::traits;

pub use paste;
pub mod core;
// pub use typelude_std::core::{self, Eval, Evaluate};
pub use core::{Apply, Eval, Evaluate};

pub use typenum;
