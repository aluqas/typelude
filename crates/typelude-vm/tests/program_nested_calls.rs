#![recursion_limit = "65536"]

mod support;

use static_assertions::assert_type_eq_all;
use typelude_col::tarr;
use typelude_vm::{
    opcode::{
        control::{OpCall, OpReturn},
        local::{OpGetLocal, OpLet},
        numeric::OpAdd,
        stack::OpPush,
    },
    vm::value::Lit,
};
use typenum::{U0, U1, U2, U3, U6, U9};

use crate::support::{OutcomeState, ProgramRun, StateLocals, StateStack};

type AddTwoAndReturn = tarr![OpPush<Lit<U2>>, OpAdd, OpReturn];
type OuterAddAndReturn = tarr![OpCall<AddTwoAndReturn>, OpPush<Lit<U3>>, OpAdd, OpReturn];
type NestedCallProgram = tarr![OpPush<Lit<U1>>, OpCall<OuterAddAndReturn>];

type ShadowLocalAndReturn = tarr![OpPush<Lit<U9>>, OpLet, OpReturn];
type CallerLocalsProgram =
    tarr![OpPush<Lit<U1>>, OpLet, OpCall<ShadowLocalAndReturn>, OpGetLocal<Lit<U0>>];

#[test]
fn nested_call_addition_produces_expected_result() {
    type DirectOut = ProgramRun<NestedCallProgram>;
    type DirectState = <DirectOut as OutcomeState>::Output;

    assert_type_eq_all!(<DirectState as StateStack>::Output, tarr![Lit<U6>]);
}

#[test]
fn callee_locals_do_not_leak_back_to_caller() {
    type DirectOut = ProgramRun<CallerLocalsProgram>;
    type DirectState = <DirectOut as OutcomeState>::Output;

    assert_type_eq_all!(<DirectState as StateLocals>::Output, tarr![Lit<U1>]);
    assert_type_eq_all!(<DirectState as StateStack>::Output, tarr![Lit<U1>]);
}
