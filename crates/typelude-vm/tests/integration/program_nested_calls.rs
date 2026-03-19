use static_assertions::assert_type_eq_all;
use typelude_std::{core::ELit, tyarray};
use typelude_vm::opcode::{OpAdd, OpCall, OpGetLocal, OpLet, OpPush, OpReturn};
use typenum::{U0, U1, U2, U3, U6, U9};

use crate::support::{DirectProgramRun, OutcomeState, StateLocals, StateStack};

type AddTwoAndReturn = tyarray![OpPush<ELit<U2>>, OpAdd, OpReturn];
type OuterAddAndReturn = tyarray![OpCall<AddTwoAndReturn>, OpPush<ELit<U3>>, OpAdd, OpReturn];
type NestedCallProgram = tyarray![OpPush<ELit<U1>>, OpCall<OuterAddAndReturn>];

type ShadowLocalAndReturn = tyarray![OpPush<ELit<U9>>, OpLet, OpReturn];
type CallerLocalsProgram =
    tyarray![OpPush<ELit<U1>>, OpLet, OpCall<ShadowLocalAndReturn>, OpGetLocal<ELit<U0>>];

#[test]
fn nested_call_addition_produces_expected_result() {
    type DirectOut = DirectProgramRun<NestedCallProgram>;
    type DirectState = <DirectOut as OutcomeState>::Output;

    assert_type_eq_all!(<DirectState as StateStack>::Output, tyarray![ELit<U6>]);
}

#[test]
fn callee_locals_do_not_leak_back_to_caller() {
    type DirectOut = DirectProgramRun<CallerLocalsProgram>;
    type DirectState = <DirectOut as OutcomeState>::Output;

    assert_type_eq_all!(<DirectState as StateLocals>::Output, tyarray![ELit<U1>]);
    assert_type_eq_all!(<DirectState as StateStack>::Output, tyarray![ELit<U1>]);
}
