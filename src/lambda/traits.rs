//! **Lambda Traits**
//!
//! Traits specific to pure functional programming context.

use crate::eval::Eval;

/// **Pure Lambda Term Trait**
///
/// Types implementing this trait represent pure lambda calculus terms
/// or operations derived strictly from them.
///
/// Implementing `Lambda` automatically provides `Eval` via blanket impl.
pub trait Lambda {
    type Output;
}

/// Blanket impl: Lambda types automatically implement Eval
impl<T: Lambda> Eval for T {
    type Output = <T as Lambda>::Output;
}
