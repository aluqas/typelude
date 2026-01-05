pub mod app;
pub mod bridge;

pub mod apply; // Typenum support

pub use app::*;
pub use apply::*;
pub use bridge::*;

/// Sealed trait pattern
#[doc(hidden)]
pub trait Sealed {}

pub mod impls;

#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be evaluated",
    label = "Eval not implemented",
    note = "ensure `{Self}` implements `Eval` or is a valid expression"
)]
pub trait Eval {
    type Output;
}

/// Type alias to obtain evaluation results of expressions
pub type Evaluate<T> = <T as Eval>::Output;

/// Evaluation Barrier for Literals
///
/// Wraps a type `T` to prevent further evaluation or to simply treat it as a value.
/// `ELit<T>` evaluates to `T`.
pub struct ELit<T>(pub std::marker::PhantomData<T>);

impl<T> Eval for ELit<T> {
    type Output = T;
}
