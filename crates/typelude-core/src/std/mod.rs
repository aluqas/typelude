//! # Standard Library
//!
//! Standard library for `typelude`.
//! Provides data type definitions and operations (functions) for them.

pub mod debug;
pub mod ops;
pub mod primitives;

pub mod reify;
pub mod traits;

pub use debug::trace;
pub use ops::{arith, cmp, into, logic};
pub use primitives::{array, bool, int, str};
