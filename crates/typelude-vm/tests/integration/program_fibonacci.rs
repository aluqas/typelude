use static_assertions::assert_type_eq_all;
use typelude_col::{TTerm, tarr};
use typelude_vm::{
    opcode::{
        control::OpWhile,
        host::OpHostCall,
        local::{OpGetLocal, OpLet, OpSetLocal},
        numeric::{OpAdd, OpLt},
        stack::OpPush,
    },
    vm::{
        protocol::request::{HostRequest, HostSignature},
        semantics::state::VmState,
        value::Lit,
    },
};
use typenum::{U0, U1, U2, U10, U55, U89};

use crate::support::{
    N162, N163, OutcomeRequest, OutcomeState, ProgramRun, RequestResponse, RequestSig,
    SourceTraceLen, StateLocals, StateStack,
};

struct Print;

impl HostSignature for Print {
    type Response = U0;
}

type FibCond<N> = tarr![OpPush<Lit<N>>, OpGetLocal<Lit<U0>>, OpLt];
type FibBody = tarr![
    OpGetLocal<Lit<U2>>,
    OpGetLocal<Lit<U1>>,
    OpAdd,
    OpGetLocal<Lit<U1>>,
    OpSetLocal<Lit<U2>>,
    OpSetLocal<Lit<U1>>,
    OpPush<Lit<U1>>,
    OpGetLocal<Lit<U0>>,
    OpAdd,
    OpSetLocal<Lit<U0>>
];
type FibProgram10 = tarr![
    OpPush<Lit<U0>>,
    OpPush<Lit<U1>>,
    OpPush<Lit<U0>>,
    OpLet,
    OpLet,
    OpLet,
    OpWhile<FibCond<U10>, FibBody>,
    OpGetLocal<Lit<U2>>
];
type FibProgram10WithHost = tarr![
    OpPush<Lit<U0>>,
    OpPush<Lit<U1>>,
    OpPush<Lit<U0>>,
    OpLet,
    OpLet,
    OpLet,
    OpWhile<FibCond<U10>, FibBody>,
    OpGetLocal<Lit<U2>>,
    OpHostCall<Print>
];
type FibFinalState =
    VmState<tarr![Lit<U55>], tarr![Lit<U10>, Lit<U89>, Lit<U55>], TTerm, TTerm, TTerm>;
type FibOut = ProgramRun<FibProgram10>;
type FibOutState = <FibOut as OutcomeState>::Output;
type FibHostOut = ProgramRun<FibProgram10WithHost>;
type FibHostOutState = <FibHostOut as OutcomeState>::Output;

#[test]
fn fibonacci_10_composed_final_state_is_stable() {
    assert_type_eq_all!(<FibOutState as StateStack>::Output, tarr![Lit<U55>]);
    assert_type_eq_all!(<FibOutState as StateLocals>::Output, tarr![Lit<U10>, Lit<U89>, Lit<U55>]);
    assert_type_eq_all!(FibOutState, FibFinalState);
}

#[test]
fn fibonacci_10_composed_trace_length_is_stable() {
    assert_type_eq_all!(SourceTraceLen<FibOut>, N162);
}

#[test]
fn fibonacci_10_then_host_suspend_has_stable_state_and_request() {
    type ComposedSig = <<FibHostOut as OutcomeRequest>::Output as RequestSig>::Output;
    type ComposedResponse = <<FibHostOut as OutcomeRequest>::Output as RequestResponse>::Output;
    type ExpectedRequest = HostRequest<Print, tarr![Lit<U55>], U0>;

    assert_type_eq_all!(<FibHostOutState as StateStack>::Output, tarr![Lit<U55>]);
    assert_type_eq_all!(FibHostOutState, FibFinalState);
    assert_type_eq_all!(ComposedSig, Print);
    assert_type_eq_all!(ComposedResponse, U0);
    assert_type_eq_all!(<FibHostOut as OutcomeRequest>::Output, ExpectedRequest);
    assert_type_eq_all!(SourceTraceLen<FibHostOut>, N163);
}
