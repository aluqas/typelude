//! Type-level collection utilities.
//!
//! This crate owns collection primitive values and collection capability
//! implementations. Shared higher-order APIs should use the canonical operator
//! names from `typelude_std::core`.
//!
//! Canonical operator mapping:
//! - `Len` -> `OpLen`
//! - `Head` -> `OpHead`
//! - `Tail` -> `OpTail`
//! - `Get` -> `OpGet`
//! - `Set` -> `OpSet`
//! - `Concat` -> `OpConcat`
//! - `Map` -> `OpMap`
//! - `Fold` -> `OpFold`

mod array;

pub use array::{TArr, TTerm};
pub use typelude_std::core::{Append, Concat, Get, Head, Len, Prepend, Set, Tail};
