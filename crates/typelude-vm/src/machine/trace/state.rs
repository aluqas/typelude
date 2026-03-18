use crate::machine::{
    core::CoreState,
    effects::TraceEffects,
    machine::Machine,
    meta::{NoFuel, NoWorld, VmMeta},
};

pub type TracedMachineState<Stack, Locals, Memory, CallStack, Program, History> = Machine<
    CoreState<Stack, Locals, Memory, CallStack, typelude_std::std::col::array::Nil, Program>,
    VmMeta<History, NoFuel, NoWorld>,
    TraceEffects,
>;
