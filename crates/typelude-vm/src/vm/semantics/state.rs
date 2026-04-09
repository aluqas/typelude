//! Canonical VM state.

use core::marker::PhantomData;

use typelude_col::TTerm;
use typelude_std::core::Eval;

/// Full machine snapshot used by the pure semantics and the runtime state
/// effect.
///
/// The stack top is the array head and the remaining `Program` is interpreted
/// as the next instruction stream to execute.
#[derive(Debug)]
pub struct VmState<Stack, Locals, Memory, Frames, Program = TTerm>(
    pub PhantomData<(Stack, Locals, Memory, Frames, Program)>,
);

impl<S, L, M, F, P> Eval for VmState<S, L, M, F, P> {
    type Output = Self;
}
