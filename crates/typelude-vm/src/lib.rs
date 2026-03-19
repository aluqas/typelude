#![cfg_attr(feature = "nightly", feature(generic_const_exprs))]
#![allow(incomplete_features)]
#![recursion_limit = "65536"]

pub mod core;
pub mod opcode;
pub mod shared;
pub mod vm;
