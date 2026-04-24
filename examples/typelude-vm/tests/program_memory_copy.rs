#![recursion_limit = "65536"]

mod support;

use static_assertions::assert_type_eq_all;
use typelude_col::{TTerm, tarr};
use typelude_vm::{
    opcode::{
        memory::{OpLoad, OpStore},
        stack::{OpPush, OpSwap},
    },
    vm::{semantics::state::VmState, value::Lit},
};
use typenum::{U0, U1, U2, U7};

use crate::support::{OutcomeState, Run, StateMemory, StateStack};

type MemoryCopyProgram =
    tarr![OpPush<Lit<U0>>, OpLoad, OpPush<Lit<U1>>, OpSwap, OpStore, OpPush<Lit<U1>>, OpLoad];
type StoreAndLoadProgram =
    tarr![OpPush<Lit<U1>>, OpPush<Lit<U7>>, OpStore, OpPush<Lit<U1>>, OpLoad];

#[test]
fn memory_copy_program_final_state_is_stable() {
    type InitialMemory = tarr![Lit<U2>, Lit<U0>];
    type ComposedInitial = VmState<TTerm, TTerm, InitialMemory, TTerm, MemoryCopyProgram>;
    type ComposedOut = Run<ComposedInitial>;
    type ComposedState = <ComposedOut as OutcomeState>::Output;

    assert_type_eq_all!(<ComposedState as StateStack>::Output, tarr![Lit<U2>]);
    assert_type_eq_all!(<ComposedState as StateMemory>::Output, tarr![Lit<U2>, Lit<U2>]);
}

#[test]
fn store_and_load_roundtrip_is_stable() {
    type InitialMemory = tarr![Lit<U0>, Lit<U0>];
    type ComposedInitial = VmState<TTerm, TTerm, InitialMemory, TTerm, StoreAndLoadProgram>;
    type ComposedOut = Run<ComposedInitial>;
    type ComposedState = <ComposedOut as OutcomeState>::Output;

    assert_type_eq_all!(<ComposedState as StateStack>::Output, tarr![Lit<U7>]);
    assert_type_eq_all!(<ComposedState as StateMemory>::Output, tarr![Lit<U0>, Lit<U7>]);
}
