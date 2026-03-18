use core::marker::PhantomData;

use typelude_std::{
    core::{Eval, Evaluate},
    std::col::array::{Array, IsList, Nil},
};

use crate::machine::{
    core::CoreState,
    effects::TraceEffects,
    machine::Machine,
    meta::{NoFuel, NoWorld, VmMeta},
    run,
    step::{ExtractContinue, Step},
};

pub trait TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History> {
    type OutputState;
}

type CompatMachine<Stack, Locals, Memory, CallStack, RestProg, History, Inst> = Machine<
    CoreState<Stack, Locals, Memory, CallStack, Nil, Array<Inst, RestProg>>,
    VmMeta<History, NoFuel, NoWorld>,
    TraceEffects,
>;

impl<Inst, Stack, Locals, Memory, CallStack, RestProg, History>
    TracedExecute<Stack, Locals, Memory, CallStack, RestProg, History> for Inst
where
    RestProg: IsList,
    Inst: Step<CompatMachine<Stack, Locals, Memory, CallStack, RestProg, History, Inst>>,
    <Inst as Step<CompatMachine<Stack, Locals, Memory, CallStack, RestProg, History, Inst>>>::Output:
        ExtractContinue,
{
    type OutputState = <<Inst as Step<
        CompatMachine<Stack, Locals, Memory, CallStack, RestProg, History, Inst>,
    >>::Output as ExtractContinue>::Output;
}

pub struct ETracedRun<S>(pub PhantomData<S>);

impl<S> Eval for ETracedRun<S>
where
    S: Eval,
    Evaluate<S>: run::RunMachine,
    <Evaluate<S> as run::RunMachine>::Output: run::ExtractTerminal,
{
    type Output = <<Evaluate<S> as run::RunMachine>::Output as run::ExtractTerminal>::Output;
}
