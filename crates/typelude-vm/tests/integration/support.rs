#![allow(dead_code)]

use typelude_std::{
    core::{ELit, Evaluate},
    std::col::array::{ELen, Nil},
};
use typelude_vm::{
    opcode::host::OpHostCall,
    vm::{
        protocol::request::{HostRequest, HostRequestResponse},
        runtime::{
            effects::trace::{CoreTraceEvent, SourceTraceEvent, TraceBundle},
            outcome::{Done, Raised, Suspended},
            run::{EResumeVm, ERunVm},
        },
        semantics::state::VmState,
    },
};

pub type EmptyState = VmState<Nil, Nil, Nil, Nil, Nil>;

pub type Run<Initial> = Evaluate<ERunVm<ELit<Initial>>>;
pub type ProgramRun<Prog> = Run<VmState<Nil, Nil, Nil, Nil, Prog>>;
pub type Resume<Outcome, Response> = Evaluate<EResumeVm<Outcome, Response>>;

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

pub trait OutcomeTraceBundle {
    type Output;
}

impl<A, State, Trace> OutcomeTraceBundle for Done<A, State, Trace> {
    type Output = Trace;
}

impl<Reason, State, Trace> OutcomeTraceBundle for Raised<Reason, State, Trace> {
    type Output = Trace;
}

impl<Request, State, Trace> OutcomeTraceBundle for Suspended<Request, State, Trace> {
    type Output = Trace;
}

pub trait OutcomeSourceTrace {
    type Output;
}

pub trait OutcomeCoreTrace {
    type Output;
}

impl<A, State, Source, Core> OutcomeSourceTrace for Done<A, State, TraceBundle<Source, Core>> {
    type Output = Source;
}

impl<Reason, State, Source, Core> OutcomeSourceTrace
    for Raised<Reason, State, TraceBundle<Source, Core>>
{
    type Output = Source;
}

impl<Request, State, Source, Core> OutcomeSourceTrace
    for Suspended<Request, State, TraceBundle<Source, Core>>
{
    type Output = Source;
}

impl<A, State, Source, Core> OutcomeCoreTrace for Done<A, State, TraceBundle<Source, Core>> {
    type Output = Core;
}

impl<Reason, State, Source, Core> OutcomeCoreTrace
    for Raised<Reason, State, TraceBundle<Source, Core>>
{
    type Output = Core;
}

impl<Request, State, Source, Core> OutcomeCoreTrace
    for Suspended<Request, State, TraceBundle<Source, Core>>
{
    type Output = Core;
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

impl<Sig, Args, Response> RequestSig for HostRequest<Sig, Args, Response> {
    type Output = Sig;
}

pub trait RequestResponse {
    type Output;
}

impl<Request> RequestResponse for Request
where
    Request: HostRequestResponse,
{
    type Output = <Request as HostRequestResponse>::Output;
}

pub type SourceTraceLen<O> = Evaluate<ELen<ELit<<O as OutcomeSourceTrace>::Output>>>;
pub type CoreTraceLen<O> = Evaluate<ELen<ELit<<O as OutcomeCoreTrace>::Output>>>;
pub type HostSourceTraceEvent<Sig> = SourceTraceEvent<OpHostCall<Sig>>;
pub type HostCoreTraceEvent<Sig> = CoreTraceEvent<OpHostCall<Sig>>;
