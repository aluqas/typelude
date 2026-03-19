use static_assertions::assert_type_eq_all;
use typelude_std::{core::ELit, std::prim::bool::True, tyarray};
use typelude_vm::opcode::{OpAdd, OpGetLocal, OpIf, OpLet, OpLt, OpPush, OpSetLocal, OpWhile};
use typenum::{U0, U1, U2, U3};

use crate::support::{
    ComposedRun, DirectProgramRun, EmptyComposedState, OutcomeState, StateLocals, StateStack,
};

type IfProgram =
    tyarray![OpPush<ELit<True>>, OpIf<tyarray![OpPush<ELit<U1>>], tyarray![OpPush<ELit<U2>>]>];

type CounterCond<N> = tyarray![OpPush<ELit<N>>, OpGetLocal<ELit<U0>>, OpLt];
type CounterBody = tyarray![OpPush<ELit<U1>>, OpGetLocal<ELit<U0>>, OpAdd, OpSetLocal<ELit<U0>>];
type CounterProgram =
    tyarray![OpPush<ELit<U0>>, OpLet, OpWhile<CounterCond<U3>, CounterBody>, OpGetLocal<ELit<U0>>];

#[test]
fn if_branch_matches_across_runners() {
    type DirectOut = DirectProgramRun<IfProgram>;
    type DirectState = <DirectOut as OutcomeState>::Output;

    type ComposedOut = ComposedRun<EmptyComposedState, IfProgram>;
    type ComposedState = <ComposedOut as OutcomeState>::Output;

    assert_type_eq_all!(
        <DirectState as StateStack>::Output,
        <ComposedState as StateStack>::Output,
        tyarray![ELit<U1>]
    );
}

#[test]
fn while_counter_matches_across_runners() {
    type DirectOut = DirectProgramRun<CounterProgram>;
    type DirectState = <DirectOut as OutcomeState>::Output;

    type ComposedOut = ComposedRun<EmptyComposedState, CounterProgram>;
    type ComposedState = <ComposedOut as OutcomeState>::Output;

    assert_type_eq_all!(
        <DirectState as StateStack>::Output,
        <ComposedState as StateStack>::Output,
        tyarray![ELit<U3>]
    );
    assert_type_eq_all!(
        <DirectState as StateLocals>::Output,
        <ComposedState as StateLocals>::Output,
        tyarray![ELit<U3>]
    );
}
