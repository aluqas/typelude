use static_assertions::assert_type_eq_all;
use typelude_std::{core::ELit, std::prim::bool::True, tyarray};
use typelude_vm::opcode::{OpAdd, OpCall, OpIf, OpPush, OpReturn};
use typenum::{U1, U2, U3, U5};

use crate::support::{
    ComposedRun, ComposedTraceLen, DirectHistoryLen, DirectTracedRun, EmptyComposedState,
    OutcomeState,
};

type IfProgram =
    tyarray![OpPush<ELit<True>>, OpIf<tyarray![OpPush<ELit<U1>>], tyarray![OpPush<ELit<U2>>]>];
type Callee = tyarray![OpPush<ELit<U2>>, OpAdd, OpReturn];
type CallProgram = tyarray![OpPush<ELit<U1>>, OpCall<Callee>];

#[test]
fn if_trace_counts_only_taken_branch() {
    type DirectOut = DirectTracedRun<IfProgram>;
    type DirectState = <DirectOut as OutcomeState>::Output;

    type ComposedOut = ComposedRun<EmptyComposedState, IfProgram>;

    assert_type_eq_all!(DirectHistoryLen<DirectState>, ComposedTraceLen<ComposedOut>, U3);
}

#[test]
fn call_trace_includes_call_and_return() {
    type DirectOut = DirectTracedRun<CallProgram>;
    type DirectState = <DirectOut as OutcomeState>::Output;

    assert_type_eq_all!(DirectHistoryLen<DirectState>, U5);
}
