//! **Core Traits**
//!
//! Core trait definitions for `typelude`.

use crate::eval::ELit;

/// Trait representing a function
///
/// Represents a transformation from type to type, used in `EWhile` etc.
pub trait TyFn<Arg> {
    type Output;
}

// Automatically unwrap ELit
impl<F, Arg> TyFn<ELit<Arg>> for F
where
    F: TyFn<Arg>,
{
    type Output = <F as TyFn<Arg>>::Output;
}
