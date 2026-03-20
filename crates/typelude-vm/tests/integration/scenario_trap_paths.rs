use static_assertions::assert_type_eq_all;
use typelude_std::{core::ELit, std::col::array::Nil, tyarray};
use typelude_vm::{
    opcode::{
        control::{OpIf, OpReturn},
        local::OpGetLocal,
        memory::OpLoad,
        numeric::OpAdd,
        stack::OpPush,
    },
    vm::{
        protocol::trap::{
            BadLocalIndex, BadMemoryIndex, InvalidCondition, ReturnUnderflow, StackUnderflow,
        },
        runtime::{effects::trace::VmTraceEvent, outcome::Raised},
        semantics::state::VmState,
    },
};
use typenum::U0;

use crate::support::{ProgramRun, Run};

#[test]
fn stack_underflow_program_traps() {
    type Prog = tyarray![OpAdd];
    type Initial = VmState<Nil, Nil, Nil, Nil, Prog>;
    type Final = ProgramRun<Prog>;
    type Expected = Raised<StackUnderflow, Initial, tyarray![VmTraceEvent<OpAdd>]>;

    assert_type_eq_all!(Final, Expected);
}

#[test]
fn bad_local_index_program_traps() {
    type Prog = tyarray![OpGetLocal<ELit<U0>>];
    type Initial = VmState<Nil, Nil, Nil, Nil, Prog>;
    type Final = ProgramRun<Prog>;
    type Expected =
        Raised<BadLocalIndex<U0>, Initial, tyarray![VmTraceEvent<OpGetLocal<ELit<U0>>>]>;

    assert_type_eq_all!(Final, Expected);
}

#[test]
fn bad_memory_index_program_traps() {
    type Initial = VmState<tyarray![ELit<U0>], Nil, Nil, Nil, tyarray![OpLoad]>;
    type Final = Run<Initial>;
    type Expected = Raised<BadMemoryIndex<U0>, Initial, tyarray![VmTraceEvent<OpLoad>]>;

    assert_type_eq_all!(Final, Expected);
}

#[test]
fn return_underflow_program_traps() {
    type Prog = tyarray![OpReturn];
    type Initial = VmState<Nil, Nil, Nil, Nil, Prog>;
    type Final = ProgramRun<Prog>;
    type Expected = Raised<ReturnUnderflow, Initial, tyarray![VmTraceEvent<OpReturn>]>;

    assert_type_eq_all!(Final, Expected);
}

#[test]
fn invalid_if_condition_program_traps() {
    type Prog = tyarray![OpIf<tyarray![OpPush<ELit<U0>>], tyarray![OpPush<ELit<U0>>]>];
    type State = VmState<tyarray![U0], Nil, Nil, Nil, Prog>;
    type Out = Run<State>;
    type Expected = Raised<
        InvalidCondition,
        State,
        tyarray![VmTraceEvent<OpIf<tyarray![OpPush<ELit<U0>>], tyarray![OpPush<ELit<U0>>]>>],
    >;

    assert_type_eq_all!(Out, Expected);
}
