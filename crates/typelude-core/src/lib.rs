pub mod expr;
pub mod list;

pub use expr::*;
pub use list::*;

#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be applied to argument `{Arg}`",
    label = "Apply not implemented",
    note = "ensure `{Self}` implements `Apply<{Arg}>`"
)]
pub trait Apply<Arg> {
    type Output;
}

/// Type alias for function application result
pub type App<F, A> = <F as Apply<A>>::Output;

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
