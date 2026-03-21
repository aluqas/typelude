use static_assertions::assert_type_eq_all;
use typelude_std::{core::ELit, std::prim::bool::True, tyarray};
use typelude_vm::{
    opcode::{
        control::{OpCall, OpIf, OpReturn},
        numeric::OpAdd,
        stack::OpPush,
    },
    vm::runtime::effects::trace::{CoreTraceEvent, SourceTraceEvent},
};
use typenum::{U1, U2, U3, U5};

use crate::support::{
    CoreTraceLen, OutcomeCoreTrace, OutcomeSourceTrace, ProgramRun, SourceTraceLen,
};

type IfProgram =
    tyarray![OpPush<ELit<True>>, OpIf<tyarray![OpPush<ELit<U1>>], tyarray![OpPush<ELit<U2>>]>];
type Callee = tyarray![OpPush<ELit<U2>>, OpAdd, OpReturn];
type CallProgram = tyarray![OpPush<ELit<U1>>, OpCall<Callee>];

#[test]
fn if_trace_counts_only_taken_branch() {
    type Out = ProgramRun<IfProgram>;
    type ExpectedSourceTrace = tyarray![
        SourceTraceEvent<OpPush<ELit<True>>>,
        SourceTraceEvent<OpIf<tyarray![OpPush<ELit<U1>>], tyarray![OpPush<ELit<U2>>]>>,
        SourceTraceEvent<OpPush<ELit<U1>>>
    ];
    type ExpectedCoreTrace = tyarray![
        CoreTraceEvent<OpPush<ELit<True>>>,
        CoreTraceEvent<OpIf<tyarray![OpPush<ELit<U1>>], tyarray![OpPush<ELit<U2>>]>>,
        CoreTraceEvent<OpPush<ELit<U1>>>
    ];

    assert_type_eq_all!(<Out as OutcomeSourceTrace>::Output, ExpectedSourceTrace);
    assert_type_eq_all!(<Out as OutcomeCoreTrace>::Output, ExpectedCoreTrace);
    assert_type_eq_all!(SourceTraceLen<Out>, U3);
    assert_type_eq_all!(CoreTraceLen<Out>, U3);
}

#[test]
fn call_trace_includes_call_and_return() {
    type Out = ProgramRun<CallProgram>;
    type ExpectedSourceTrace = tyarray![
        SourceTraceEvent<OpPush<ELit<U1>>>,
        SourceTraceEvent<OpCall<Callee>>,
        SourceTraceEvent<OpPush<ELit<U2>>>,
        SourceTraceEvent<OpAdd>,
        SourceTraceEvent<OpReturn>
    ];
    type ExpectedCoreTrace = tyarray![
        CoreTraceEvent<OpPush<ELit<U1>>>,
        CoreTraceEvent<OpCall<Callee>>,
        CoreTraceEvent<OpPush<ELit<U2>>>,
        CoreTraceEvent<OpAdd>,
        CoreTraceEvent<OpReturn>
    ];

    assert_type_eq_all!(<Out as OutcomeSourceTrace>::Output, ExpectedSourceTrace);
    assert_type_eq_all!(<Out as OutcomeCoreTrace>::Output, ExpectedCoreTrace);
    assert_type_eq_all!(SourceTraceLen<Out>, U5);
    assert_type_eq_all!(CoreTraceLen<Out>, U5);
}
