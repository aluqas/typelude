// Unstable features required for type-level programming
#![cfg_attr(feature = "nightly", feature(specialization))]
#![cfg_attr(feature = "nightly", feature(generic_const_exprs))]
#![allow(incomplete_features)]
#![recursion_limit = "16384"]

#[macro_use]
pub mod macros;

pub mod data;
pub mod expr;
pub mod lambda;
pub mod std;
pub mod traits;

// Export typenum for macros
pub use paste;
pub use typelude_core::{self, Eval, Evaluate};
pub use typenum;
