//! **Lambda Traits**
//!
//! Traits specific to pure functional programming context.
//!
//! - `Lambda`: Pure lambda term trait with automatic `Eval` implementation
//! - `Bind`: Monadic bind operation (`m >>= f`)

use crate::eval::Eval;

// =========================================================================
// Lambda Trait
// =========================================================================

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

// =========================================================================
// Bind Trait (Monad)
// =========================================================================

/// Bind Trait: `m >>= f`
///
/// `Self` is the Monad (e.g., `Option<T>`).
/// `F` is the Binder function (`T -> Option<U>`).
/// `Output` is the result Monad (`Option<U>`).
pub trait LBind<F> {
    type Output;
}

// Note: `Pure` is usually specific to the Monad type constructor itself
// (e.g. `Id<T>`, `State<S, A>`), so we might not need a universal trait for it
// unless we want generic code over Monads.
// =========================================================================
// Kind System (Marker Traits)
// =========================================================================

/// Base trait for all Lambda Terms.
pub trait LTerm {}

/// Marker for Church Booleans.
pub trait LBool: LTerm {}

/// Marker for Church Numerals.
pub trait LNat: LTerm {}

/// Marker for Church Lists.
pub trait LList: LTerm {}
