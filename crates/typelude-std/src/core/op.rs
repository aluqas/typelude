//! First-class operators and application AST.

use core::marker::PhantomData;

use super::{Eval, Evaluate};

/// Common interface for first-class operators that can be used by higher-order
/// control flow and state-machine constructs.
pub trait Op<Args> {
    type Output;
}

/// AST node representing operator application.
pub struct Apply<F, Args>(pub PhantomData<(F, Args)>);

impl<F, Args> Eval for Apply<F, Args>
where
    Args: Eval,
    F: Op<Evaluate<Args>>,
{
    type Output = <F as Op<Evaluate<Args>>>::Output;
}
