pub mod col;
pub mod expr;

pub use col::*;
pub use expr::*;

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

// pub type Apply<Fn, Args> = <Fn<Args> as Eval>::Output;
