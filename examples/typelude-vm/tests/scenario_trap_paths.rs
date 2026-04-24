#![recursion_limit = "65536"]

mod support;

use static_assertions::assert_type_eq_all;
use typelude_col::{TTerm, tarr};
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
        runtime::{
            effects::trace::{CoreTraceEvent, SourceTraceEvent, TraceBundle},
            outcome::Raised,
        },
        semantics::state::VmState,
        value::Lit,
    },
};
use typenum::U0;

use crate::support::{ProgramRun, Run};

#[test]
fn stack_underflow_program_traps() {
    type Prog = tarr![OpAdd];
    type Initial = VmState<TTerm, TTerm, TTerm, TTerm, Prog>;
    type Final = ProgramRun<Prog>;
    type Expected = Raised<
        StackUnderflow,
        Initial,
        TraceBundle<tarr![SourceTraceEvent<OpAdd>], tarr![CoreTraceEvent<OpAdd>]>,
    >;

    assert_type_eq_all!(Final, Expected);
}

#[test]
fn bad_local_index_program_traps() {
    type Prog = tarr![OpGetLocal<Lit<U0>>];
    type Initial = VmState<TTerm, TTerm, TTerm, TTerm, Prog>;
    type Final = ProgramRun<Prog>;
    type Expected = Raised<
        BadLocalIndex<U0>,
        Initial,
        TraceBundle<
            tarr![SourceTraceEvent<OpGetLocal<Lit<U0>>>],
            tarr![CoreTraceEvent<OpGetLocal<Lit<U0>>>],
        >,
    >;

    assert_type_eq_all!(Final, Expected);
}

#[test]
fn bad_memory_index_program_traps() {
    type Initial = VmState<tarr![Lit<U0>], TTerm, TTerm, TTerm, tarr![OpLoad]>;
    type Final = Run<Initial>;
    type Expected = Raised<
        BadMemoryIndex<U0>,
        Initial,
        TraceBundle<tarr![SourceTraceEvent<OpLoad>], tarr![CoreTraceEvent<OpLoad>]>,
    >;

    assert_type_eq_all!(Final, Expected);
}

#[test]
fn return_underflow_program_traps() {
    type Prog = tarr![OpReturn];
    type Initial = VmState<TTerm, TTerm, TTerm, TTerm, Prog>;
    type Final = ProgramRun<Prog>;
    type Expected = Raised<
        ReturnUnderflow,
        Initial,
        TraceBundle<tarr![SourceTraceEvent<OpReturn>], tarr![CoreTraceEvent<OpReturn>]>,
    >;

    assert_type_eq_all!(Final, Expected);
}

#[test]
fn invalid_if_condition_program_traps() {
    type Prog = tarr![OpIf<tarr![OpPush<Lit<U0>>], tarr![OpPush<Lit<U0>>]>];
    type State = VmState<tarr![Lit<U0>], TTerm, TTerm, TTerm, Prog>;
    type Out = Run<State>;
    type Expected = Raised<
        InvalidCondition,
        State,
        TraceBundle<
            tarr![SourceTraceEvent<OpIf<tarr![OpPush<Lit<U0>>], tarr![OpPush<Lit<U0>>]>>],
            tarr![CoreTraceEvent<OpIf<tarr![OpPush<Lit<U0>>], tarr![OpPush<Lit<U0>>]>>],
        >,
    >;

    assert_type_eq_all!(Out, Expected);
}
