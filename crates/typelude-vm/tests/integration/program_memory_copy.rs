use static_assertions::assert_type_eq_all;
use typelude_std::{core::ELit, std::col::array::Nil, tyarray};
use typelude_vm::{
    opcode::{OpLoad, OpPush, OpStore, OpSwap},
    vm::run::composed::ComposedVmState,
};
use typenum::{U0, U1, U2, U7};

use crate::support::{ComposedRun, OutcomeState, StateMemory, StateStack};

type MemoryCopyProgram = tyarray![
    OpPush<ELit<U0>>,
    OpLoad,
    OpPush<ELit<U1>>,
    OpSwap,
    OpStore,
    OpPush<ELit<U1>>,
    OpLoad
];
type StoreAndLoadProgram =
    tyarray![OpPush<ELit<U1>>, OpPush<ELit<U7>>, OpStore, OpPush<ELit<U1>>, OpLoad];

#[test]
fn memory_copy_program_final_state_is_stable() {
    type InitialMemory = tyarray![ELit<U2>, ELit<U0>];
    type ComposedInitial = ComposedVmState<Nil, Nil, InitialMemory, Nil>;
    type ComposedOut = ComposedRun<ComposedInitial, MemoryCopyProgram>;
    type ComposedState = <ComposedOut as OutcomeState>::Output;

    assert_type_eq_all!(<ComposedState as StateStack>::Output, tyarray![ELit<U2>]);
    assert_type_eq_all!(<ComposedState as StateMemory>::Output, tyarray![ELit<U2>, ELit<U2>]);
}

#[test]
fn store_and_load_roundtrip_is_stable() {
    type InitialMemory = tyarray![ELit<U0>, ELit<U0>];
    type ComposedInitial = ComposedVmState<Nil, Nil, InitialMemory, Nil>;
    type ComposedOut = ComposedRun<ComposedInitial, StoreAndLoadProgram>;
    type ComposedState = <ComposedOut as OutcomeState>::Output;

    assert_type_eq_all!(<ComposedState as StateStack>::Output, tyarray![ELit<U7>]);
    assert_type_eq_all!(<ComposedState as StateMemory>::Output, tyarray![ELit<U0>, ELit<U7>]);
}
