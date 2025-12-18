//! **Lambda Traits**
//!
//! Traits specific to pure functional programming context.

use crate::eval::Eval;

/// **Marker Trait for Pure Lambda Terms**
///
/// Types implementing this trait assert that they represent pure lambda calculus terms
/// or operations derived strictly from them.
///
/// While they utilize the `Eval` infrastructure for execution on the Rust type system,
/// semantically they belong to the pure functional world.
pub trait Lambda: Eval {}

// Automatically implement Lambda for things that are "Values" in Lambda Calculus?
// No, we should implement it manually for our combinators to be explicit.
