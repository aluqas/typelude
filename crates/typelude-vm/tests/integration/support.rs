#![allow(dead_code)]

use typelude_std::{
    core::{ELit, Evaluate},
    std::col::array::{ELen, Nil},
};
use typelude_vm::{
    opcode::OpHostCall,
    shared::request::HostRequest,
    vm::{
        effect::VmTraceEvent,
        run::{
            composed::{ComposedVmState, DefaultVmFx, EInterpProgram, ERunVm},
            direct::{ERun, Machine},
            outcome::{Done, Raised, Suspended},
        },
        state::{VmMeta, VmState},
        step::{Halt, Suspend, Trap},
        surface::aliases::{ProgramVm, TracedVm},
    },
};

pub type EmptyComposedState = ComposedVmState<Nil, Nil, Nil, Nil>;

pub type DirectRun<Initial> = Evaluate<ERun<ELit<Initial>>>;
pub type DirectProgramRun<Prog> = DirectRun<ProgramVm<Prog>>;
pub type DirectTracedRun<Prog> = DirectRun<TracedVm<Nil, Nil, Nil, Nil, Nil, Prog, Nil>>;

pub type ComposedRun<State, Prog> =
    Evaluate<ERunVm<State, Evaluate<EInterpProgram<Prog, DefaultVmFx<State>>>>>;
pub type EmptyComposedRun<Prog> = ComposedRun<EmptyComposedState, Prog>;

pub trait OutcomeState {
    type Output;
}

impl<Core, Meta, Fx> OutcomeState for Machine<Core, Meta, Fx> {
    type Output = Machine<Core, Meta, Fx>;
}

impl<M> OutcomeState for Halt<M> {
    type Output = M;
}

impl<Reason, M> OutcomeState for Trap<Reason, M> {
    type Output = M;
}

impl<Request, M> OutcomeState for Suspend<Request, M> {
    type Output = M;
}

impl<A, State, Trace> OutcomeState for Done<A, State, Trace> {
    type Output = State;
}

impl<Reason, State, Trace> OutcomeState for Raised<Reason, State, Trace> {
    type Output = State;
}

impl<Request, State, Trace> OutcomeState for Suspended<Request, State, Trace> {
    type Output = State;
}

pub trait OutcomeTrace {
    type Output;
}

impl<A, State, Trace> OutcomeTrace for Done<A, State, Trace> {
    type Output = Trace;
}

impl<Reason, State, Trace> OutcomeTrace for Raised<Reason, State, Trace> {
    type Output = Trace;
}

impl<Request, State, Trace> OutcomeTrace for Suspended<Request, State, Trace> {
    type Output = Trace;
}

pub trait OutcomeRequest {
    type Output;
}

impl<Request, M> OutcomeRequest for Suspend<Request, M> {
    type Output = Request;
}

impl<Request, State, Trace> OutcomeRequest for Suspended<Request, State, Trace> {
    type Output = Request;
}

pub trait TrapReason {
    type Output;
}

impl<Reason, M> TrapReason for Trap<Reason, M> {
    type Output = Reason;
}

pub trait StateStack {
    type Output;
}

pub trait StateLocals {
    type Output;
}

pub trait StateMemory {
    type Output;
}

pub trait StateHistory {
    type Output;
}

impl<S, L, M, F, B, P, Meta, Fx> StateStack for Machine<VmState<S, L, M, F, B, P>, Meta, Fx> {
    type Output = S;
}

impl<S, L, M, F, B, P, Meta, Fx> StateLocals for Machine<VmState<S, L, M, F, B, P>, Meta, Fx> {
    type Output = L;
}

impl<S, L, M, F, B, P, Meta, Fx> StateMemory for Machine<VmState<S, L, M, F, B, P>, Meta, Fx> {
    type Output = M;
}

impl<S, L, M, F, B, P, History, Fuel, World, Fx> StateHistory
    for Machine<VmState<S, L, M, F, B, P>, VmMeta<History, Fuel, World>, Fx>
{
    type Output = History;
}

impl<S, L, M, F> StateStack for ComposedVmState<S, L, M, F> {
    type Output = S;
}

impl<S, L, M, F> StateLocals for ComposedVmState<S, L, M, F> {
    type Output = L;
}

impl<S, L, M, F> StateMemory for ComposedVmState<S, L, M, F> {
    type Output = M;
}

pub trait RequestSig {
    type Output;
}

impl<Sig, Args> RequestSig for HostRequest<Sig, Args> {
    type Output = Sig;
}

pub type DirectHistoryLen<M> = Evaluate<ELen<ELit<<M as StateHistory>::Output>>>;
pub type ComposedTraceLen<O> = Evaluate<ELen<ELit<<O as OutcomeTrace>::Output>>>;
pub type HostTraceEvent<Sig> = VmTraceEvent<OpHostCall<Sig>>;
