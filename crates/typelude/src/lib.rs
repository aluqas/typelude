#![recursion_limit = "16384"]
//! # Typelude
//!
//! A comprehensive toolkit for type-level programming in Rust, providing:
//! - **Core**: Foundational AST for type expressions and evaluation.
//! - **Std**: A standard library of type-level collections (Array, Map, Tree)
//!   and primitives.
//! - **VM**: A type-level stack machine for complex logic.
//! - **Macros**: DSL for intuitive type-level programming.

pub mod core {
    pub use tstr;
    pub use typelude_std::core::*;
    pub use typenum;
}
pub use tstr;
pub use typelude_macros::{twat};
pub use typelude_std::{Eval, Evaluate};
pub use typelude_vm as vm;
pub use typelude_wasm as wasm;
pub use typenum;
