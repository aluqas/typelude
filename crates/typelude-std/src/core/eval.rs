//! Evaluation boundary for type-level expressions.

/// Common interface for evaluating a type-level expression.
pub trait Eval {
    type Output;
}

/// Evaluates a type-level expression to its normal form.
pub type Evaluate<T> = <T as Eval>::Output;
