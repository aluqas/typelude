#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
#![recursion_limit = "16384"]

pub mod machine;

// Re-export typelude-core for internal use consistency if needed
pub use typelude_core as core;
