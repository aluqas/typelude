/// Trait representing a type-level function.
///
/// Represents a transformation from type to type, used in `EWhile` and higher
/// order operations.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a valid type-level function for argument `{Arg}`",
    label = "this type cannot be applied to `{Arg}`",
    note = "ensure `{Self}` implements `TyFn<{Arg}>`"
)]
pub trait TyFn<Arg> {
    type Output;
}
