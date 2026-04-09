use static_assertions::assert_type_eq_all;
use typelude_bool::True;
use typelude_col::tarr;
use typelude_vm::{
    opcode::{
        control::{OpCall, OpIf, OpReturn},
        numeric::OpAdd,
        stack::OpPush,
    },
    vm::{
        runtime::effects::trace::{CoreTraceEvent, SourceTraceEvent},
        value::Lit,
    },
};
use typenum::{U1, U2};

use crate::support::{
    CoreTraceLen, N3, N5, OutcomeCoreTrace, OutcomeSourceTrace, ProgramRun, SourceTraceLen,
};

type IfProgram = tarr![OpPush<Lit<True>>, OpIf<tarr![OpPush<Lit<U1>>], tarr![OpPush<Lit<U2>>]>];
type Callee = tarr![OpPush<Lit<U2>>, OpAdd, OpReturn];
type CallProgram = tarr![OpPush<Lit<U1>>, OpCall<Callee>];

#[test]
fn if_trace_counts_only_taken_branch() {
    type Out = ProgramRun<IfProgram>;
    type ExpectedSourceTrace = tarr![
        SourceTraceEvent<OpPush<Lit<True>>>,
        SourceTraceEvent<OpIf<tarr![OpPush<Lit<U1>>], tarr![OpPush<Lit<U2>>]>>,
        SourceTraceEvent<OpPush<Lit<U1>>>
    ];
    type ExpectedCoreTrace = tarr![
        CoreTraceEvent<OpPush<Lit<True>>>,
        CoreTraceEvent<OpIf<tarr![OpPush<Lit<U1>>], tarr![OpPush<Lit<U2>>]>>,
        CoreTraceEvent<OpPush<Lit<U1>>>
    ];

    assert_type_eq_all!(<Out as OutcomeSourceTrace>::Output, ExpectedSourceTrace);
    assert_type_eq_all!(<Out as OutcomeCoreTrace>::Output, ExpectedCoreTrace);
    assert_type_eq_all!(SourceTraceLen<Out>, N3);
    assert_type_eq_all!(CoreTraceLen<Out>, N3);
}

#[test]
fn call_trace_includes_call_and_return() {
    type Out = ProgramRun<CallProgram>;
    type ExpectedSourceTrace = tarr![
        SourceTraceEvent<OpPush<Lit<U1>>>,
        SourceTraceEvent<OpCall<Callee>>,
        SourceTraceEvent<OpPush<Lit<U2>>>,
        SourceTraceEvent<OpAdd>,
        SourceTraceEvent<OpReturn>
    ];
    type ExpectedCoreTrace = tarr![
        CoreTraceEvent<OpPush<Lit<U1>>>,
        CoreTraceEvent<OpCall<Callee>>,
        CoreTraceEvent<OpPush<Lit<U2>>>,
        CoreTraceEvent<OpAdd>,
        CoreTraceEvent<OpReturn>
    ];

    assert_type_eq_all!(<Out as OutcomeSourceTrace>::Output, ExpectedSourceTrace);
    assert_type_eq_all!(<Out as OutcomeCoreTrace>::Output, ExpectedCoreTrace);
    assert_type_eq_all!(SourceTraceLen<Out>, N5);
    assert_type_eq_all!(CoreTraceLen<Out>, N5);
}
