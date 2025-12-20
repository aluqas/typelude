//! **Unified Application AST**
//!
//! This module defines `App<Op, Args>`, the canonical AST node for generic function application.
//! It serves as the fallback/generic alternative to specific ASTs like `EAdd`.

use std::marker::PhantomData;

use crate::{
    eval::{Eval, Evaluate},
    kernel::traits::Apply,
};

/// **Generic Function Application**: `App<Op, Args>`
///
/// Represents the application of an operator `Op` to arguments `Args`.
///
/// - **Op**: The operator (e.g., `OpAdd`, `MyFunc`).
/// - **Args**: The arguments (e.g., `(U1, U2)` or `U1`).
///
/// # Evaluation Strategy
///
/// When `App<Op, Args>` is evaluated:
/// 1. It calls `Op::Apply(Args)` (via the `Apply` trait) to produce an underlying expression (AST).
/// 2. It evaluates that underlying expression.
///
/// This allows `App` to wrap *any* operation that implements `Apply`,
/// regardless of whether it returns a specific AST (`EAdd`) or another `App`.
pub struct App<Op, Args>(PhantomData<(Op, Args)>);

impl<Op, Args> Eval for App<Op, Args>
where
    Op: Apply<Args>,
    Op::Output: Eval,
{
    type Output = Evaluate<Op::Output>;
}
