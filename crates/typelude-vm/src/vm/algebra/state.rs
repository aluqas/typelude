use core::marker::PhantomData;

use typelude_std::{
    core::Eval,
    std::{col::array::Nil, debug::trace::Trace},
};

#[derive(Debug)]
pub struct VmState<Stack, Locals, Memory, Frames, Program = Nil>(
    pub PhantomData<(Stack, Locals, Memory, Frames, Program)>,
);

impl<S, L, M, F, P> Eval for VmState<S, L, M, F, P> {
    type Output = Self;
}

impl<Stack, Locals, Memory, Frames, Program> Trace for VmState<Stack, Locals, Memory, Frames, Program>
where
    Stack: Trace,
    Locals: Trace,
    Memory: Trace,
{
    fn fmt() -> String {
        format!(
            "VmState {{\n  Stack: {}\n  Locals: {}\n  Memory: {}\n}}",
            Stack::fmt(),
            Locals::fmt(),
            Memory::fmt()
        )
    }
}
