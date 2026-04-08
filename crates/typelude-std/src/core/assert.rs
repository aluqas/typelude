//! Common assertion and unwrapping helpers.

/// Unwrap a type-level container.
#[diagnostic::on_unimplemented(message = "`{Self}` cannot be unwrapped", label = "unwrap failed")]
pub trait Unwrap {
    type Output;
}

/// Unwrap a type-level container or fall back to `Default`.
pub trait UnwrapOr<Default> {
    type Output;
}

/// Compile-time type equality assertion.
#[diagnostic::on_unimplemented(
    message = "Unexpected types",
    label = "validation failed",
    note = "expected: {Check}"
)]
pub trait Assert<Check> {}

impl<T> Assert<T> for T {}
