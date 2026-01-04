use core::marker::PhantomData;

use typelude_std::{
    eval::{Eval, Sealed},
    std::trace::Trace,
};

#[derive(Debug)]
pub struct TracedMachineState<Stack, Locals, Memory, CallStack, Program, History>(
    PhantomData<(Stack, Locals, Memory, CallStack, Program, History)>,
);

impl<S, L, M, C, P, H> Sealed for TracedMachineState<S, L, M, C, P, H> {}

impl<S, L, M, C, P, H> Eval for TracedMachineState<S, L, M, C, P, H> {
    type Output = TracedMachineState<S, L, M, C, P, H>;
}

impl<S, L, M, C, P, H> Trace for TracedMachineState<S, L, M, C, P, H>
where
    S: Trace,
    L: Trace,
    M: Trace,
    H: Trace,
{
    fn fmt() -> String {
        format!(
            "MachineState {{\n  Stack: {}\n  Locals: {}\n  Memory: {}\n  History: {}\n}}",
            S::fmt(),
            L::fmt(),
            M::fmt(),
            H::fmt()
        )
    }
}
