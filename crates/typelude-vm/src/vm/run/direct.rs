use core::marker::PhantomData;

use typelude_std::{
    core::{Eval, Evaluate},
    std::{
        col::array::{Array, Nil},
        debug::trace::Trace,
    },
};

use crate::vm::{
    effect::BeforeStep,
    state::{VmMeta, VmState},
    step::{Continue, Halt, RunNext, Step, Suspend, Trap},
};

#[derive(Debug)]
pub struct Machine<Core, Meta, Fx>(pub PhantomData<(Core, Meta, Fx)>);

impl<C, M, F> Eval for Machine<C, M, F> {
    type Output = Self;
}

impl<S, L, M, F, B, P, H, Fuel, W, Fx> Trace
    for Machine<VmState<S, L, M, F, B, P>, VmMeta<H, Fuel, W>, Fx>
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

pub trait RunMachine {
    type Output;
}

impl<Stack, Locals, Memory, Frames, Labels, Meta, Fx> RunMachine
    for Machine<VmState<Stack, Locals, Memory, Frames, Labels, Nil>, Meta, Fx>
{
    type Output = Halt<Machine<VmState<Stack, Locals, Memory, Frames, Labels, Nil>, Meta, Fx>>;
}

pub trait RunPrepared<Inst> {
    type Output;
}

impl<Inst, M> RunPrepared<Inst> for Continue<M>
where
    Inst: Step<M>,
    <Inst as Step<M>>::Output: RunNext,
{
    type Output = <<Inst as Step<M>>::Output as RunNext>::Output;
}

impl<Inst, Reason, M> RunPrepared<Inst> for Trap<Reason, M> {
    type Output = Trap<Reason, M>;
}

impl<Inst, Request, M> RunPrepared<Inst> for Suspend<Request, M> {
    type Output = Suspend<Request, M>;
}

pub trait RunWithFuel<M, Inst> {
    type Output;
}

impl<Fx, M, Inst> RunWithFuel<M, Inst> for Fx
where
    Fx: BeforeStep<M>,
    <Fx as BeforeStep<M>>::Output: RunPrepared<Inst>,
{
    type Output = <<Fx as BeforeStep<M>>::Output as RunPrepared<Inst>>::Output;
}

type RunningMachine<Stack, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx> =
    Machine<VmState<Stack, Locals, Memory, Frames, Labels, Array<Inst, Rest>>, Meta, Fx>;

type FuelOutput<Stack, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx> =
    <Fx as RunWithFuel<
        RunningMachine<Stack, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx>,
        Inst,
    >>::Output;

impl<Stack, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx> RunMachine
    for RunningMachine<Stack, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx>
where
    Rest: typelude_std::std::col::array::IsList,
    Fx: RunWithFuel<
            RunningMachine<Stack, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx>,
            Inst,
        >,
{
    type Output = FuelOutput<Stack, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx>;
}

pub trait ExtractTerminal {
    type Output;
}

impl<M> ExtractTerminal for Halt<M> {
    type Output = M;
}

impl<Reason, M> ExtractTerminal for Trap<Reason, M> {
    type Output = Trap<Reason, M>;
}

impl<Request, M> ExtractTerminal for Suspend<Request, M> {
    type Output = Suspend<Request, M>;
}

pub struct ERun<S>(pub PhantomData<S>);

impl<S> Eval for ERun<S>
where
    S: Eval,
    Evaluate<S>: RunMachine,
    <Evaluate<S> as RunMachine>::Output: ExtractTerminal,
{
    type Output = <<Evaluate<S> as RunMachine>::Output as ExtractTerminal>::Output;
}
