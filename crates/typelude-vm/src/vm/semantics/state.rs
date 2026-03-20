//! Canonical VM state.

use core::marker::PhantomData;

use typelude_std::{
    core::Eval,
    std::{col::array::Nil, debug::trace::Trace},
};

/// Full machine snapshot used by the pure semantics and the runtime state effect.
///
/// The stack top is the array head and the remaining `Program` is interpreted as the next
/// instruction stream to execute.
#[derive(Debug)]
pub struct VmState<Stack, Locals, Memory, Frames, Program = Nil>(
    pub PhantomData<(Stack, Locals, Memory, Frames, Program)>,
);

impl<S, L, M, F, P> Eval for VmState<S, L, M, F, P> {
    type Output = Self;
}

impl<Stack, Locals, Memory, Frames, Program> Trace
    for VmState<Stack, Locals, Memory, Frames, Program>
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
