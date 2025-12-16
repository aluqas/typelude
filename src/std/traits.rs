//! **Core Traits**
//!
//! Core trait definitions for `typelude`.

use crate::eval::ELit;

/// Trait representing a function
///
/// Represents a transformation from type to type, used in `EWhile` etc.
pub trait EFunction<A> {
    type Output;
}

// Automatically unwrap ELit
impl<F, T> EFunction<ELit<T>> for F
where
    F: EFunction<T>,
{
    type Output = <F as EFunction<T>>::Output;
}
