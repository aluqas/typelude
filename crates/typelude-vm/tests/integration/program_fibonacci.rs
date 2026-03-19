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
    shared::request::HostRequest,
    vm::algebra::state::VmState,
};
use typenum::{U0, U1, U2, U10, U55, U89, U162, U163};

use crate::support::{
    OutcomeRequest, OutcomeState, ProgramRun, RequestSig, StateLocals, StateStack, TraceLen,
};

struct Print;

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

#[test]
fn fibonacci_10_composed_final_state_is_stable() {
    type ComposedOut = ProgramRun<FibProgram10>;
    type ComposedState = <ComposedOut as OutcomeState>::Output;

    assert_type_eq_all!(<ComposedState as StateStack>::Output, tyarray![ELit<U55>]);
    assert_type_eq_all!(<ComposedState as StateLocals>::Output, tyarray![
        ELit<U10>,
        ELit<U89>,
        ELit<U55>
    ]);
    assert_type_eq_all!(ComposedState, FibFinalState);
}

#[test]
fn fibonacci_10_composed_trace_length_is_stable() {
    type ComposedOut = ProgramRun<FibProgram10>;

    assert_type_eq_all!(TraceLen<ComposedOut>, U162);
}

#[test]
fn fibonacci_10_then_host_suspend_has_stable_state_and_request() {
    type ComposedOut = ProgramRun<FibProgram10WithHost>;
    type ComposedState = <ComposedOut as OutcomeState>::Output;
    type ComposedSig = <<ComposedOut as OutcomeRequest>::Output as RequestSig>::Output;
    type ExpectedRequest = HostRequest<Print, tyarray![ELit<U55>]>;

    assert_type_eq_all!(<ComposedState as StateStack>::Output, tyarray![ELit<U55>]);
    assert_type_eq_all!(ComposedState, FibFinalState);
    assert_type_eq_all!(ComposedSig, Print);
    assert_type_eq_all!(<ComposedOut as OutcomeRequest>::Output, ExpectedRequest);
    assert_type_eq_all!(TraceLen<ComposedOut>, U163);
}
