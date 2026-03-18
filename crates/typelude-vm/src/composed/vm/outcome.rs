use core::marker::PhantomData;

use typelude_std::core::Eval;

#[derive(Debug)]
pub struct Done<A, State, Trace>(pub PhantomData<(A, State, Trace)>);

#[derive(Debug)]
pub struct Raised<Trap, State, Trace>(pub PhantomData<(Trap, State, Trace)>);

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
