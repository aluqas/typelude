use static_assertions::assert_type_eq_all;
use typelude_std::{core::ELit, std::prim::bool::True, tyarray};
use typelude_vm::{
    opcode::{
        control::{OpCall, OpIf, OpReturn},
        numeric::OpAdd,
        stack::OpPush,
    },
    vm::algebra::effect::trace::VmTraceEvent,
};
use typenum::{U1, U2, U3, U5};

use crate::support::{OutcomeTrace, ProgramRun, TraceLen};

type IfProgram =
    tyarray![OpPush<ELit<True>>, OpIf<tyarray![OpPush<ELit<U1>>], tyarray![OpPush<ELit<U2>>]>];
type Callee = tyarray![OpPush<ELit<U2>>, OpAdd, OpReturn];
type CallProgram = tyarray![OpPush<ELit<U1>>, OpCall<Callee>];

#[test]
fn if_trace_counts_only_taken_branch() {
    type Out = ProgramRun<IfProgram>;
    type ExpectedTrace = tyarray![
        VmTraceEvent<OpPush<ELit<True>>>,
        VmTraceEvent<OpIf<tyarray![OpPush<ELit<U1>>], tyarray![OpPush<ELit<U2>>]>>,
        VmTraceEvent<OpPush<ELit<U1>>>
    ];

    assert_type_eq_all!(<Out as OutcomeTrace>::Output, ExpectedTrace);
    assert_type_eq_all!(TraceLen<Out>, U3);
}

#[test]
fn call_trace_includes_call_and_return() {
    type Out = ProgramRun<CallProgram>;
    type ExpectedTrace = tyarray![
        VmTraceEvent<OpPush<ELit<U1>>>,
        VmTraceEvent<OpCall<Callee>>,
        VmTraceEvent<OpPush<ELit<U2>>>,
        VmTraceEvent<OpAdd>,
        VmTraceEvent<OpReturn>
    ];

    assert_type_eq_all!(<Out as OutcomeTrace>::Output, ExpectedTrace);
    assert_type_eq_all!(TraceLen<Out>, U5);
}
