//! **Kernel: Traits**
//!
//! Core traits for type application.

/// Trait representing a function application (High-Order Function Application).
///
/// Represents how a type "Operator" applies to an "Argument".
///
/// Formerly `TyFn`.
pub trait Apply<Arg> {
    type Output;
}
