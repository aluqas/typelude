//! **Type-Level Boolean Data**
//!
//! Pure data structures for type-level booleans.

use typelude_std::core::Eval;

/// **Marker Trait**
///
/// Represents that a type is a Boolean.
pub trait IsBool {}

/// Type-level `true`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct True;

/// Type-level `false`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct False;

impl IsBool for True {}
impl Eval for True {
    type Output = Self;
}

impl IsBool for False {}
impl Eval for False {
    type Output = Self;
}
