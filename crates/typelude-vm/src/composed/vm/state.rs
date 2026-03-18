use core::marker::PhantomData;

use typelude_std::core::Eval;

#[derive(Debug)]
pub struct VmState<Stack, Locals, Memory, Frames>(pub PhantomData<(Stack, Locals, Memory, Frames)>);

impl<S, L, M, F> Eval for VmState<S, L, M, F> {
    type Output = Self;
}

#[derive(Debug)]
pub struct CallFrame<ReturnLocals, ReturnProgram>(pub PhantomData<(ReturnLocals, ReturnProgram)>);

impl<L, P> Eval for CallFrame<L, P> {
    type Output = Self;
}
