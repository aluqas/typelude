//! **Type-Level Boolean Data**
//!
//! Pure data structures for type-level booleans.

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

impl IsBool for False {}
