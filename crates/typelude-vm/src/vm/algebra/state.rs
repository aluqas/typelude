use core::marker::PhantomData;

use typelude_std::core::Eval;
pub type CallFrame<ReturnLocals, ReturnProgram> =
    crate::shared::frame::ReturnFrame<ReturnLocals, ReturnProgram>;

#[derive(Debug)]
pub struct VmState<Stack, Locals, Memory, Frames>(
    pub PhantomData<(Stack, Locals, Memory, Frames)>,
);

impl<S, L, M, F> Eval for VmState<S, L, M, F> {
    type Output = Self;
}
