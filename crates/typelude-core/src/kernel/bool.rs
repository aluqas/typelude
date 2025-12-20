//! **Kernel: Boolean Types**
//!
//! Pure data structures for type-level booleans.

use crate::eval::{Eval, Sealed};

/// **Marker Trait**
///
/// Represents that a type is a Boolean.
pub trait Bool: Sealed {}

/// Type-level `true`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct TyTrue;

/// Type-level `false`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct TyFalse;

impl Sealed for TyTrue {}
impl Bool for TyTrue {}
impl Eval for TyTrue {
    type Output = Self;
}

impl Sealed for TyFalse {}
impl Bool for TyFalse {}
impl Eval for TyFalse {
    type Output = Self;
}
