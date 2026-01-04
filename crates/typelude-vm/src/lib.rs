#![cfg_attr(feature = "nightly", feature(generic_const_exprs))]
#![allow(incomplete_features)]
#![recursion_limit = "65536"]

pub mod machine;

// Re-export typelude-core for internal use consistency if needed
pub use typelude_core as core;
