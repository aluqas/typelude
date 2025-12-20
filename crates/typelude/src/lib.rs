#![recursion_limit = "16384"]
//! # Typelude
//!
//! A library for type-level programming in Rust.

pub use typelude_core as core;
pub use typelude_vm as vm;
pub use typelude_macros::program;

pub use core::eval;
pub use core::kernel;
pub use core::lambda;
pub use core::std;
pub use vm::machine;

pub use typenum;
pub use core::{
    tyarray,
    eval::{Eval, Evaluate},
    kernel::traits::Apply,
};
