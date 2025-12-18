//! Type-Level Monad Traits
//!
//! Defines the standard Monad operations:
//! - `Bind`: `m >>= f`

/// Bind Trait: m >>= f
///
/// `Self` is the Monad (e.g., `Option<T>`).
/// `F` is the Binder function (`T -> Option<U>`).
/// `Output` is the result Monad (`Option<U>`).
pub trait Bind<F> {
    type Output;
}

// Note: `Pure` is usually specific to the Monad type constructor itself
// (e.g. `Id<T>`, `State<S, A>`), so we might not need a universal trait for it
// unless we want generic code over Monads.
// For now, we focus on `Bind`.
