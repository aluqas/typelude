use typelude_std::std::col::array::Nil;

use crate::machine::{
    core::CoreState,
    effects::PureEffects,
    machine::Machine,
    meta::DefaultMeta,
};

pub type MachineState<Stack, Locals, Memory, CallStack, Program> =
    Machine<CoreState<Stack, Locals, Memory, CallStack, Nil, Program>, DefaultMeta, PureEffects>;

pub trait GetStack {
    type Output;
}

impl<S, L, M, C, B, P, Meta, Fx> GetStack for Machine<CoreState<S, L, M, C, B, P>, Meta, Fx> {
    type Output = S;
}
