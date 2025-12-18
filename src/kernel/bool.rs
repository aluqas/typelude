//! **Kernel: Boolean Types**
//!
//! Pure data structures for type-level booleans.

use crate::eval::Sealed;

/// Marker Trait for Kernel Booleans
pub trait TyBool: Sealed {
    const BOOL: bool;
}

/// Type representing True (Value)
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, PartialOrd, Ord)]
pub struct TyTrue;

/// Type representing False (Value)
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Default, PartialOrd, Ord)]
pub struct TyFalse;

impl Sealed for TyTrue {}
impl Sealed for TyFalse {}

impl TyBool for TyTrue {
    const BOOL: bool = true;
}

impl TyBool for TyFalse {
    const BOOL: bool = false;
}
