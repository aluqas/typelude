use core::marker::PhantomData;

use typelude_std::core::Eval;

#[derive(Debug)]
pub struct Continue<M>(pub PhantomData<M>);

impl<M> Eval for Continue<M> {
    type Output = Self;
}

#[derive(Debug)]
pub struct Halt<M>(pub PhantomData<M>);

impl<M> Eval for Halt<M> {
    type Output = Self;
}

#[derive(Debug)]
pub struct Trap<Reason, M>(pub PhantomData<(Reason, M)>);

impl<R, M> Eval for Trap<R, M> {
    type Output = Self;
}

#[derive(Debug)]
pub struct Suspend<Request, M>(pub PhantomData<(Request, M)>);

impl<R, M> Eval for Suspend<R, M> {
    type Output = Self;
}
