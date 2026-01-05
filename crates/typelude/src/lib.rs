#![recursion_limit = "16384"]
//! # Typelude
//!
//! A library for type-level programming in Rust.

pub mod core {
    pub use typelude_core::*;
    pub use typelude_std::{
        lambda,
        std, // Just in case
        tyarray,
    };
    pub use typenum;
}
pub use typelude_core::{Apply, Eval, Evaluate};
pub use typelude_macros::program;
pub use typelude_std::{lambda, std, tyarray};
pub use typelude_vm as vm;
pub use typenum;
pub use vm::machine;
