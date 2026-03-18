use core::marker::PhantomData;

use typelude_std::core::Eval;

#[derive(Debug)]
pub struct CallFrame<ReturnProgram, ReturnLocals, ReturnLabels>(
    pub PhantomData<(ReturnProgram, ReturnLocals, ReturnLabels)>,
);

impl<P, L, B> Eval for CallFrame<P, L, B> {
    type Output = Self;
}

#[derive(Debug)]
pub struct LabelFrame<Continuation, Arity>(pub PhantomData<(Continuation, Arity)>);

impl<C, A> Eval for LabelFrame<C, A> {
    type Output = Self;
}

#[derive(Debug)]
pub struct ReturnFrame<ReturnLocals, ReturnProgram>(
    pub PhantomData<(ReturnLocals, ReturnProgram)>,
);

impl<L, P> Eval for ReturnFrame<L, P> {
    type Output = Self;
}
