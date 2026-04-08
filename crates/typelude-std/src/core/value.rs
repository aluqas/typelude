//! Self-evaluating values and tuple argument packs.

use super::{Eval, Evaluate};

/// Marker trait for values that evaluate to themselves.
pub trait Value {}

impl<T> Eval for T
where
    T: Value,
{
    type Output = Self;
}

impl Eval for () {
    type Output = ();
}

impl<A> Eval for (A,)
where
    A: Eval,
{
    type Output = (Evaluate<A>,);
}

impl<A, B> Eval for (A, B)
where
    A: Eval,
    B: Eval,
{
    type Output = (Evaluate<A>, Evaluate<B>);
}

impl<A, B, C> Eval for (A, B, C)
where
    A: Eval,
    B: Eval,
    C: Eval,
{
    type Output = (Evaluate<A>, Evaluate<B>, Evaluate<C>);
}

impl<A, B, C, D> Eval for (A, B, C, D)
where
    A: Eval,
    B: Eval,
    C: Eval,
    D: Eval,
{
    type Output = (Evaluate<A>, Evaluate<B>, Evaluate<C>, Evaluate<D>);
}
