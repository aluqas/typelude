//! # Standard Library
//!
//! Standard library for `typelude`.
//! Provides data type definitions and operations (functions) for them.

pub mod debug;
pub mod ops;
mod primitives; // Hidden

pub mod reify;
pub mod traits;
pub mod testing; // New testing module

pub use debug::trace;
// Flatten primitives
pub use primitives::{array, bool, int, str};
