//! Conditional expression AST.

use core::marker::PhantomData;

/// Conditional expression AST.
///
/// Semantics may be provided either directly with `Eval` impls or via sugar
/// over `Apply<OpIf, ...>`.
pub struct If<Cond, Then, Else>(pub PhantomData<(Cond, Then, Else)>);
