//! Terminal runtime outcomes.

use core::marker::PhantomData;

use typelude_std::core::Eval;

/// Successful completion with the committed end state and both trace streams.
#[derive(Debug)]
pub struct Done<A, State, Trace>(pub PhantomData<(A, State, Trace)>);

/// Trapped execution with the pre-step state of the failing instruction and
/// both trace streams.
#[derive(Debug)]
pub struct Raised<Trap, State, Trace>(pub PhantomData<(Trap, State, Trace)>);

/// Suspended execution with the advanced state committed before yielding and
/// both trace streams.
#[derive(Debug)]
pub struct Suspended<Request, State, Trace>(pub PhantomData<(Request, State, Trace)>);

impl<A, S, T> Eval for Done<A, S, T> {
    type Output = Self;
}

impl<E, S, T> Eval for Raised<E, S, T> {
    type Output = Self;
}

impl<R, S, T> Eval for Suspended<R, S, T> {
    type Output = Self;
}
