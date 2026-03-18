use core::marker::PhantomData;

use typelude_std::core::Eval;

#[derive(Debug)]
pub struct CoreState<Stack, Locals, Memory, Frames, Labels, Program>(
    pub PhantomData<(Stack, Locals, Memory, Frames, Labels, Program)>,
);

impl<S, L, M, F, B, P> Eval for CoreState<S, L, M, F, B, P> {
    type Output = Self;
}

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
