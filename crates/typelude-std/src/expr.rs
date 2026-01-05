//! **Expression Types (Deprecated)**
//!
//! These types have been moved to `std::control` as part of RFC-0002.
//! Please update your imports.

#[deprecated(since = "0.2.0", note = "Use `typelude_std::std::control::EIf` instead")]
pub use crate::std::control::EIf;
#[deprecated(since = "0.2.0", note = "Use `typelude_std::std::control::EWhile` instead")]
pub use crate::std::control::EWhile;
