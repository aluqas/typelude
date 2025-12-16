//! **Core Traits**
//!
//! Core trait definitions for `typelude`.

use crate::eval::ELit;

/// Trait representing a function
///
/// Represents a transformation from type to type, used in `EWhile` etc.
pub trait EFunction<Arg> {
    type Output;
}

// Automatically unwrap ELit
impl<F, Arg> EFunction<ELit<Arg>> for F
where
    F: EFunction<Arg>,
{
    type Output = <F as EFunction<Arg>>::Output;
}
