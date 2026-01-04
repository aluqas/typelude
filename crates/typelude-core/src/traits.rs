//! **Kernel: Traits**
//!
//! Core traits for type application.

/// Trait representing a function application (High-Order Function Application).
///
/// Represents how a type "Operator" applies to an "Argument".
///
/// Formerly `TyFn`.
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be applied to argument `{Arg}`",
    label = "Apply not implemented",
    note = "ensure `{Self}` implements `Apply<{Arg}>`"
)]
pub trait Apply<Arg> {
    type Output;
}
