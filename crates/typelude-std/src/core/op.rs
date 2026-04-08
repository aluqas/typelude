//! First-class operators and application AST.
//!
//! This module is the canonical higher-order ABI for `typelude`.
//! Primitive crates such as `typelude-bool`, `typelude-num`, and
//! `typelude-col` provide values and capability traits, while shared
//! higher-order APIs should speak in terms of `Op<Args>` and `Apply<F, Args>`.

use core::marker::PhantomData;

use super::{Eval, Evaluate};

/// Common interface for first-class operators that can be used by higher-order
/// control flow and state-machine constructs.
///
/// Concrete semantics for canonical operators such as `OpAdd` or `OpGet` are
/// intentionally owned by `typelude-std`.
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
