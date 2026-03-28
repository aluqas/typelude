use static_assertions::assert_type_eq_all;
use typelude_std::{core::ELit, tyarray};
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
    },
};
use typenum::{U0, U1, U2, U10, U55, U89, U162, U163};

use crate::support::{
    OutcomeRequest, OutcomeState, ProgramRun, RequestResponse, RequestSig, SourceTraceLen,
    StateLocals, StateStack,
};

struct Print;

impl HostSignature for Print {
    type Response = U0;
}

type FibCond<N> = tyarray![OpPush<ELit<N>>, OpGetLocal<ELit<U0>>, OpLt];
type FibBody = tyarray![
    OpGetLocal<ELit<U2>>,
    OpGetLocal<ELit<U1>>,
    OpAdd,
    OpGetLocal<ELit<U1>>,
    OpSetLocal<ELit<U2>>,
    OpSetLocal<ELit<U1>>,
    OpPush<ELit<U1>>,
    OpGetLocal<ELit<U0>>,
    OpAdd,
    OpSetLocal<ELit<U0>>
];
type FibProgram10 = tyarray![
    OpPush<ELit<U0>>,
    OpPush<ELit<U1>>,
    OpPush<ELit<U0>>,
    OpLet,
    OpLet,
    OpLet,
    OpWhile<FibCond<U10>, FibBody>,
    OpGetLocal<ELit<U2>>
];
type FibProgram10WithHost = tyarray![
    OpPush<ELit<U0>>,
    OpPush<ELit<U1>>,
    OpPush<ELit<U0>>,
    OpLet,
    OpLet,
    OpLet,
    OpWhile<FibCond<U10>, FibBody>,
    OpGetLocal<ELit<U2>>,
    OpHostCall<Print>
];
type FibFinalState = VmState<
    tyarray![ELit<U55>],
    tyarray![ELit<U10>, ELit<U89>, ELit<U55>],
    typelude_std::std::col::array::Nil,
    typelude_std::std::col::array::Nil,
    typelude_std::std::col::array::Nil,
>;
type FibOut = ProgramRun<FibProgram10>;
type FibOutState = <FibOut as OutcomeState>::Output;
type FibHostOut = ProgramRun<FibProgram10WithHost>;
type FibHostOutState = <FibHostOut as OutcomeState>::Output;

#[test]
fn fibonacci_10_composed_final_state_is_stable() {
    assert_type_eq_all!(<FibOutState as StateStack>::Output, tyarray![ELit<U55>]);
    assert_type_eq_all!(
        <FibOutState as StateLocals>::Output,
        tyarray![ELit<U10>, ELit<U89>, ELit<U55>]
    );
    assert_type_eq_all!(FibOutState, FibFinalState);
}

#[test]
fn fibonacci_10_composed_trace_length_is_stable() {
    assert_type_eq_all!(SourceTraceLen<FibOut>, U162);
}

#[test]
fn fibonacci_10_then_host_suspend_has_stable_state_and_request() {
    type ComposedSig = <<FibHostOut as OutcomeRequest>::Output as RequestSig>::Output;
    type ComposedResponse = <<FibHostOut as OutcomeRequest>::Output as RequestResponse>::Output;
    type ExpectedRequest = HostRequest<Print, tyarray![ELit<U55>], U0>;

    assert_type_eq_all!(<FibHostOutState as StateStack>::Output, tyarray![ELit<U55>]);
    assert_type_eq_all!(FibHostOutState, FibFinalState);
    assert_type_eq_all!(ComposedSig, Print);
    assert_type_eq_all!(ComposedResponse, U0);
    assert_type_eq_all!(<FibHostOut as OutcomeRequest>::Output, ExpectedRequest);
    assert_type_eq_all!(SourceTraceLen<FibHostOut>, U163);
}
