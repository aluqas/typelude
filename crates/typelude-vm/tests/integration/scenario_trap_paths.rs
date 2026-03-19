use static_assertions::assert_type_eq_all;
use typelude_std::{
    core::{ELit, Evaluate},
    std::col::array::Nil,
    tyarray,
};
use typelude_vm::{
    opcode::{OpAdd, OpGetLocal, OpIf, OpLoad, OpPush, OpReturn},
    shared::trap::{
        BadLocalIndex, BadMemoryIndex, InvalidCondition, ReturnUnderflow, StackUnderflow,
    },
    vm::{
        run::{
            composed::{ComposedVmState, DefaultVmFx, EInterpProgram, ERunVm},
            direct::ERun,
            outcome::Raised,
        },
        step::Trap,
        surface::aliases::PureVm,
    },
};
use typenum::U0;

use crate::support::TrapReason;

#[test]
fn direct_stack_underflow_program_traps() {
    type Prog = tyarray![OpAdd];
    type Initial = PureVm<Nil, Nil, Nil, Nil, Nil, Prog>;
    type Final = Evaluate<ERun<ELit<Initial>>>;

    assert_type_eq_all!(Final, Trap<StackUnderflow, Initial>);
}

#[test]
fn direct_bad_local_index_program_traps() {
    type Prog = tyarray![OpGetLocal<ELit<U0>>];
    type Initial = PureVm<Nil, Nil, Nil, Nil, Nil, Prog>;
    type Final = Evaluate<ERun<ELit<Initial>>>;

    assert_type_eq_all!(<Final as TrapReason>::Output, BadLocalIndex<U0>);
}

#[test]
fn direct_bad_memory_index_program_traps() {
    type Initial = PureVm<tyarray![ELit<U0>], Nil, Nil, Nil, Nil, tyarray![OpLoad]>;
    type Final = Evaluate<ERun<ELit<Initial>>>;

    assert_type_eq_all!(<Final as TrapReason>::Output, BadMemoryIndex<U0>);
}

#[test]
fn direct_return_underflow_program_traps() {
    type Prog = tyarray![OpReturn];
    type Initial = PureVm<Nil, Nil, Nil, Nil, Nil, Prog>;
    type Final = Evaluate<ERun<ELit<Initial>>>;

    assert_type_eq_all!(Final, Trap<ReturnUnderflow, Initial>);
}

#[test]
fn composed_invalid_if_condition_program_traps() {
    type State = ComposedVmState<tyarray![U0], Nil, Nil, Nil>;
    type Prog = Evaluate<
        EInterpProgram<
            tyarray![OpIf<tyarray![OpPush<ELit<U0>>], tyarray![OpPush<ELit<U0>>]>],
            DefaultVmFx<State>,
        >,
    >;
    type Out = Evaluate<ERunVm<State, Prog>>;
    type Expected = Raised<
        InvalidCondition,
        State,
        tyarray![
            typelude_vm::vm::effect::VmTraceEvent<
                OpIf<tyarray![OpPush<ELit<U0>>], tyarray![OpPush<ELit<U0>>]>,
            >
        ],
    >;

    assert_type_eq_all!(Out, Expected);
}
