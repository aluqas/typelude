//! # Standard Library
//!
//! Standard library for `typelude`.
//! Provides data type definitions and operations (functions) for them.

pub mod col;
pub mod control;
pub mod debug;
pub mod ops;
pub mod prim;

pub mod reify;
pub mod testing; // New testing module
pub mod traits;

// Flatten primitives
pub use col::array;
pub use debug::trace;
pub use prim::{bool, int, str};
