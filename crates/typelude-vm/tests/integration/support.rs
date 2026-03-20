#![allow(dead_code)]

use typelude_std::{
    core::{ELit, Evaluate},
    std::col::array::{ELen, Nil},
};
use typelude_vm::{
    opcode::host::OpHostCall,
    vm::{
        protocol::request::HostRequest,
        runtime::{
            effects::trace::VmTraceEvent,
            outcome::{Done, Raised, Suspended},
            run::ERunVm,
        },
        semantics::state::VmState,
    },
};

pub type EmptyState = VmState<Nil, Nil, Nil, Nil, Nil>;

pub type Run<Initial> = Evaluate<ERunVm<ELit<Initial>>>;
pub type ProgramRun<Prog> = Run<VmState<Nil, Nil, Nil, Nil, Prog>>;

pub trait OutcomeState {
    type Output;
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

impl<Request, State, Trace> OutcomeRequest for Suspended<Request, State, Trace> {
    type Output = Request;
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

impl<S, L, M, F, P> StateStack for VmState<S, L, M, F, P> {
    type Output = S;
}

impl<S, L, M, F, P> StateLocals for VmState<S, L, M, F, P> {
    type Output = L;
}

impl<S, L, M, F, P> StateMemory for VmState<S, L, M, F, P> {
    type Output = M;
}

pub trait RequestSig {
    type Output;
}

impl<Sig, Args> RequestSig for HostRequest<Sig, Args> {
    type Output = Sig;
}

pub type TraceLen<O> = Evaluate<ELen<ELit<<O as OutcomeTrace>::Output>>>;
pub type HostTraceEvent<Sig> = VmTraceEvent<OpHostCall<Sig>>;
