#![allow(dead_code)]

use typelude_col::TTerm;
use typelude_num::peano::{Succ, Zero};
use typelude_std::core::{Add, Apply, Evaluate, OpLen};
use typelude_vm::{
    opcode::host::OpHostCall,
    vm::{
        protocol::request::{HostRequest, HostRequestResponse},
        runtime::{
            effects::trace::{CoreTraceEvent, SourceTraceEvent, TraceBundle},
            outcome::{Done, Raised, Suspended},
            run::{ResumeVm, RunVm},
        },
        semantics::state::VmState,
    },
};

pub type EmptyState = VmState<TTerm, TTerm, TTerm, TTerm, TTerm>;

pub type N0 = Zero;
pub type N1 = Succ<N0>;
pub type N2 = Succ<N1>;
pub type N3 = Succ<N2>;
pub type N4 = Succ<N3>;
pub type N5 = Succ<N4>;
pub type N6 = Succ<N5>;
pub type N7 = Succ<N6>;
pub type N8 = Succ<N7>;
pub type N9 = Succ<N8>;
pub type N10 = Succ<N9>;
pub type N20 = <N10 as Add<N10>>::Output;
pub type N40 = <N20 as Add<N20>>::Output;
pub type N80 = <N40 as Add<N40>>::Output;
pub type N160 = <N80 as Add<N80>>::Output;
pub type N162 = <N160 as Add<N2>>::Output;
pub type N163 = <N160 as Add<N3>>::Output;

pub type Run<Initial> = Evaluate<RunVm<Initial>>;
pub type ProgramRun<Prog> = Run<VmState<TTerm, TTerm, TTerm, TTerm, Prog>>;
pub type Resume<Outcome, Response> = Evaluate<ResumeVm<Outcome, Response>>;

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

pub type SourceTraceLen<O> = Evaluate<Apply<OpLen, <O as OutcomeSourceTrace>::Output>>;
pub type CoreTraceLen<O> = Evaluate<Apply<OpLen, <O as OutcomeCoreTrace>::Output>>;
pub type HostSourceTraceEvent<Sig> = SourceTraceEvent<OpHostCall<Sig>>;
pub type HostCoreTraceEvent<Sig> = CoreTraceEvent<OpHostCall<Sig>>;
