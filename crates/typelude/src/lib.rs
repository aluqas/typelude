#![recursion_limit = "16384"]
//! # Typelude
//!
//! A library for type-level programming in Rust.

pub use core::{
    eval,
    eval::{Eval, Evaluate},
    kernel,
    kernel::traits::Apply,
    lambda, std, tyarray,
};

pub use typelude_core as core;
pub use typelude_macros::program;
pub use typelude_vm as vm;
pub use typenum;
pub use vm::machine;
