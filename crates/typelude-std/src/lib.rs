// Unstable features required for type-level programming

// Use for NegativeEquality
#![cfg_attr(feature = "nightly", feature(specialization))]
// Use for Integration to value
#![cfg_attr(feature = "nightly", feature(generic_const_exprs))]
#![allow(incomplete_features)]
#![recursion_limit = "65536"]

extern crate self as typelude_std;

#[macro_use]
pub mod macros;
pub mod control;
pub mod core;
pub mod debug;
pub mod effect;

pub use core::{Apply, Eval, Evaluate, IntoValue, Lift, Op, Reflect, Reify, Value};

pub use control::{If, While};
