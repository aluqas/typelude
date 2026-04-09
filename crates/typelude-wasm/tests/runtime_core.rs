mod support;

use static_assertions::assert_type_eq_all;
use typelude_col::{TTerm, tarr};
use typelude_std::core::Evaluate;
use typelude_wasm::{
    ModuleProgramRun, RunWasm, WasmFunc, WasmI32, WasmMemory, WasmModule, WasmState,
    opcode::{
        OpBlock, OpBr, OpBrIf, OpCall, OpI32Add, OpI32Const, OpI32Eqz, OpI32Load, OpI32Load8U,
        OpI32Store, OpI32Store8, OpI32Sub, OpIf, OpLocalGet, OpLocalSet, OpLocalTee, OpLoop,
        OpMemorySize, OpReturn, OpSelect,
    },
};
use typenum::{U0, U1, U2, U3, U5, U6, U8, U9, U258};

use crate::support::{StateBranches, StateLocals, StateMemory, StateProgram, StateStack};

type ZeroPages = WasmMemory<U0, TTerm>;
type OnePage = WasmMemory<U1, TTerm>;
type EmptyModule = WasmModule<TTerm, ZeroPages>;
type OnePageModule = WasmModule<TTerm, OnePage>;

#[test]
fn const_and_add_produce_expected_stack() {
    type Program = tarr![OpI32Const<U1>, OpI32Const<U2>, OpI32Add];
    type FinalState = ModuleProgramRun<EmptyModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U3>]);
    assert_type_eq_all!(<FinalState as StateProgram>::Output, TTerm);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

#[test]
fn sub_uses_wasm_operand_order() {
    type Program = tarr![OpI32Const<U5>, OpI32Const<U2>, OpI32Sub];
    type FinalState = ModuleProgramRun<EmptyModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U3>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

#[test]
fn eqz_returns_canonical_i32_boolean() {
    type ZeroProgram = tarr![OpI32Const<U0>, OpI32Eqz];
    type NonZeroProgram = tarr![OpI32Const<U5>, OpI32Eqz];

    type ZeroState = ModuleProgramRun<EmptyModule, ZeroProgram>;
    type NonZeroState = ModuleProgramRun<EmptyModule, NonZeroProgram>;

    assert_type_eq_all!(<ZeroState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<NonZeroState as StateStack>::Output, tarr![WasmI32<U0>]);
}

#[test]
fn local_set_and_get_round_trip() {
    type Program = tarr![OpI32Const<U5>, OpLocalSet<U0>, OpLocalGet<U0>];
    type InitialState =
        WasmState<EmptyModule, TTerm, tarr![WasmI32<U0>], ZeroPages, TTerm, TTerm, Program>;
    type FinalState = Evaluate<RunWasm<InitialState>>;

    assert_type_eq_all!(<FinalState as StateLocals>::Output, tarr![WasmI32<U5>]);
    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U5>]);
}

#[test]
fn local_tee_updates_local_and_preserves_stack() {
    type Program = tarr![OpI32Const<U5>, OpLocalTee<U0>];
    type InitialState =
        WasmState<EmptyModule, TTerm, tarr![WasmI32<U0>], ZeroPages, TTerm, TTerm, Program>;
    type FinalState = Evaluate<RunWasm<InitialState>>;

    assert_type_eq_all!(<FinalState as StateLocals>::Output, tarr![WasmI32<U5>]);
    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U5>]);
}

type AddTwoAndReturn = WasmFunc<U0, TTerm, tarr![OpI32Const<U2>, OpI32Add, OpReturn]>;
type SimpleCallModule = WasmModule<tarr![AddTwoAndReturn], ZeroPages>;

type OuterAddAndReturn =
    WasmFunc<U0, TTerm, tarr![OpCall<U1>, OpI32Const<U3>, OpI32Add, OpReturn]>;
type NestedCallModule = WasmModule<tarr![OuterAddAndReturn, AddTwoAndReturn], ZeroPages>;

#[test]
fn simple_call_and_return_resume_caller_continuation() {
    type Program = tarr![OpI32Const<U1>, OpCall<U0>, OpI32Const<U3>, OpI32Add];
    type FinalState = ModuleProgramRun<SimpleCallModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U6>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

#[test]
fn nested_calls_produce_expected_result() {
    type Program = tarr![OpI32Const<U1>, OpCall<U0>];
    type FinalState = ModuleProgramRun<NestedCallModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U6>]);
}

type ShadowLocalAndReturn =
    WasmFunc<U0, tarr![WasmI32<U0>], tarr![OpI32Const<U9>, OpLocalSet<U0>, OpReturn]>;
type ShadowLocalModule = WasmModule<tarr![ShadowLocalAndReturn], ZeroPages>;

#[test]
fn callee_locals_do_not_leak_back_to_caller() {
    type Program = tarr![OpCall<U0>, OpLocalGet<U0>];
    type InitialState =
        WasmState<ShadowLocalModule, TTerm, tarr![WasmI32<U1>], ZeroPages, TTerm, TTerm, Program>;
    type FinalState = Evaluate<RunWasm<InitialState>>;

    assert_type_eq_all!(<FinalState as StateLocals>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
}

type FallthroughAdd = WasmFunc<U0, TTerm, tarr![OpI32Const<U2>, OpI32Add]>;
type FallthroughModule = WasmModule<tarr![FallthroughAdd], ZeroPages>;

#[test]
fn callee_fallthrough_returns_via_end_func() {
    type Program = tarr![OpI32Const<U1>, OpCall<U0>, OpI32Const<U3>, OpI32Add];
    type FinalState = ModuleProgramRun<FallthroughModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U6>]);
}

#[test]
fn if_true_branch_executes_then_program() {
    type Program = tarr![OpI32Const<U1>, OpIf<tarr![OpI32Const<U2>], tarr![OpI32Const<U3>]>];
    type FinalState = ModuleProgramRun<EmptyModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U2>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

#[test]
fn if_false_branch_executes_else_program() {
    type Program = tarr![OpI32Const<U0>, OpIf<tarr![OpI32Const<U2>], tarr![OpI32Const<U3>]>];
    type FinalState = ModuleProgramRun<EmptyModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U3>]);
}

#[test]
fn select_chooses_true_value_on_nonzero_condition() {
    type Program = tarr![OpI32Const<U2>, OpI32Const<U3>, OpI32Const<U1>, OpSelect];
    type FinalState = ModuleProgramRun<EmptyModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U2>]);
}

#[test]
fn select_chooses_false_value_on_zero_condition() {
    type Program = tarr![OpI32Const<U2>, OpI32Const<U3>, OpI32Const<U0>, OpSelect];
    type FinalState = ModuleProgramRun<EmptyModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U3>]);
}

#[test]
fn block_fallthrough_leaves_branch_stack_empty() {
    type Program = tarr![OpBlock<tarr![OpI32Const<U1>]>];
    type FinalState = ModuleProgramRun<EmptyModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

#[test]
fn br_zero_exits_innermost_block() {
    type Program = tarr![OpBlock<tarr![OpBr<U0>, OpI32Const<U9>]>, OpI32Const<U1>];
    type FinalState = ModuleProgramRun<EmptyModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

#[test]
fn nested_block_br_one_exits_outer_block() {
    type Program = tarr![
        OpBlock<tarr![OpBlock<tarr![OpBr<U1>, OpI32Const<U9>]>, OpI32Const<U8>]>,
        OpI32Const<U1>
    ];
    type FinalState = ModuleProgramRun<EmptyModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

#[test]
fn loop_and_br_if_drive_countdown_to_zero() {
    type Program = tarr![
        OpI32Const<U3>,
        OpLocalSet<U0>,
        OpBlock<
            tarr![
                OpLoop<
                    tarr![
                        OpLocalGet<U0>,
                        OpI32Eqz,
                        OpBrIf<U1>,
                        OpLocalGet<U0>,
                        OpI32Const<U1>,
                        OpI32Sub,
                        OpLocalSet<U0>,
                        OpBr<U0>
                    ],
                >
            ],
        >,
        OpLocalGet<U0>
    ];
    type InitialState =
        WasmState<EmptyModule, TTerm, tarr![WasmI32<U0>], ZeroPages, TTerm, TTerm, Program>;
    type FinalState = Evaluate<RunWasm<InitialState>>;

    assert_type_eq_all!(<FinalState as StateLocals>::Output, tarr![WasmI32<U0>]);
    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U0>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

#[test]
fn loop_fallthrough_pops_active_loop_label() {
    type Program = tarr![OpLoop<tarr![OpI32Const<U1>]>];
    type FinalState = ModuleProgramRun<EmptyModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

type BranchingCallee = WasmFunc<U0, TTerm, tarr![OpBlock<tarr![OpBr<U0>]>, OpReturn]>;
type BranchingModule = WasmModule<tarr![BranchingCallee], ZeroPages>;

#[test]
fn calls_inside_control_flow_restore_caller_branches() {
    type Program = tarr![OpBlock<tarr![OpCall<U0>, OpBr<U0>, OpI32Const<U9>]>, OpI32Const<U1>];
    type FinalState = ModuleProgramRun<BranchingModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

type ScopedBranchCallee = WasmFunc<U0, TTerm, tarr![OpBlock<tarr![OpI32Const<U1>]>, OpReturn]>;
type ScopedBranchModule = WasmModule<tarr![ScopedBranchCallee], ZeroPages>;

type AddParamsAndReturn =
    WasmFunc<U2, TTerm, tarr![OpLocalGet<U0>, OpLocalGet<U1>, OpI32Add, OpReturn]>;
type AddParamsModule = WasmModule<tarr![AddParamsAndReturn], ZeroPages>;

#[test]
fn call_binds_params_from_stack_in_wasm_order() {
    type Program = tarr![OpI32Const<U2>, OpI32Const<U3>, OpCall<U0>];
    type FinalState = ModuleProgramRun<AddParamsModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U5>]);
}

type ZeroInitLocalAndReturn = WasmFunc<U0, tarr![WasmI32<U0>], tarr![OpLocalGet<U0>, OpReturn]>;
type ZeroInitLocalModule = WasmModule<tarr![ZeroInitLocalAndReturn], ZeroPages>;

#[test]
fn extra_locals_are_zero_initialized() {
    type Program = tarr![OpCall<U0>];
    type FinalState = ModuleProgramRun<ZeroInitLocalModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U0>]);
}

#[test]
fn callee_branch_stack_does_not_leak_back_to_caller() {
    type Program = tarr![OpCall<U0>];
    type FinalState = ModuleProgramRun<ScopedBranchModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

#[test]
fn memory_size_reports_initial_page_count() {
    type Program = tarr![OpMemorySize];
    type FinalState = ModuleProgramRun<OnePageModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateMemory>::Output, OnePage);
}

#[test]
fn load8_u_of_unwritten_in_bounds_byte_returns_zero() {
    type Program = tarr![OpI32Const<U0>, OpI32Load8U];
    type FinalState = ModuleProgramRun<OnePageModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U0>]);
    assert_type_eq_all!(<FinalState as StateMemory>::Output, OnePage);
}

#[test]
fn load_of_unwritten_in_bounds_word_returns_zero() {
    type Program = tarr![OpI32Const<U0>, OpI32Load];
    type FinalState = ModuleProgramRun<OnePageModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U0>]);
}

#[test]
fn store8_then_load8_u_round_trips_low_byte() {
    type Program =
        tarr![OpI32Const<U0>, OpI32Const<U258>, OpI32Store8, OpI32Const<U0>, OpI32Load8U];
    type FinalState = ModuleProgramRun<OnePageModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U2>]);
}

#[test]
fn store_then_load_round_trips_i32_value() {
    type Program = tarr![OpI32Const<U0>, OpI32Const<U258>, OpI32Store, OpI32Const<U0>, OpI32Load];
    type FinalState = ModuleProgramRun<OnePageModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U258>]);
}

#[test]
fn store_uses_little_endian_layout() {
    type Program =
        tarr![OpI32Const<U0>, OpI32Const<U258>, OpI32Store, OpI32Const<U1>, OpI32Load8U];
    type FinalState = ModuleProgramRun<OnePageModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
}

#[test]
fn later_byte_writes_shadow_earlier_ones() {
    type Program = tarr![
        OpI32Const<U0>,
        OpI32Const<U1>,
        OpI32Store8,
        OpI32Const<U0>,
        OpI32Const<U2>,
        OpI32Store8,
        OpI32Const<U0>,
        OpI32Load8U
    ];
    type FinalState = ModuleProgramRun<OnePageModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U2>]);
}

#[test]
fn control_flow_and_memory_compose_in_looped_program() {
    type Program = tarr![
        OpI32Const<U3>,
        OpLocalSet<U0>,
        OpI32Const<U0>,
        OpLocalSet<U1>,
        OpBlock<
            tarr![
                OpLoop<
                    tarr![
                        OpLocalGet<U0>,
                        OpI32Eqz,
                        OpBrIf<U1>,
                        OpLocalGet<U1>,
                        OpLocalGet<U0>,
                        OpI32Store8,
                        OpLocalGet<U1>,
                        OpI32Const<U1>,
                        OpI32Add,
                        OpLocalSet<U1>,
                        OpLocalGet<U0>,
                        OpI32Const<U1>,
                        OpI32Sub,
                        OpLocalSet<U0>,
                        OpBr<U0>
                    ],
                >
            ],
        >,
        OpI32Const<U1>,
        OpI32Load8U
    ];
    type InitialState = WasmState<
        OnePageModule,
        TTerm,
        tarr![WasmI32<U0>, WasmI32<U0>],
        OnePage,
        TTerm,
        TTerm,
        Program,
    >;
    type FinalState = Evaluate<RunWasm<InitialState>>;

    assert_type_eq_all!(<FinalState as StateLocals>::Output, tarr![WasmI32<U0>, WasmI32<U3>]);
    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U2>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

type RecursiveSumDown = WasmFunc<
    U1,
    TTerm,
    tarr![
        OpLocalGet<U0>,
        OpI32Eqz,
        OpIf<tarr![OpI32Const<U0>, OpReturn], TTerm>,
        OpLocalGet<U0>,
        OpLocalGet<U0>,
        OpI32Const<U1>,
        OpI32Sub,
        OpCall<U0>,
        OpI32Add,
        OpReturn
    ],
>;
type RecursiveSumModule = WasmModule<tarr![RecursiveSumDown], ZeroPages>;

#[test]
fn self_recursive_calls_execute_correctly() {
    type Program = tarr![OpI32Const<U3>, OpCall<U0>];
    type FinalState = ModuleProgramRun<RecursiveSumModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U6>]);
}

type IsEven = WasmFunc<
    U1,
    TTerm,
    tarr![
        OpLocalGet<U0>,
        OpI32Eqz,
        OpIf<tarr![OpI32Const<U1>, OpReturn], TTerm>,
        OpLocalGet<U0>,
        OpI32Const<U1>,
        OpI32Sub,
        OpCall<U1>,
        OpReturn
    ],
>;
type IsOdd = WasmFunc<
    U1,
    TTerm,
    tarr![
        OpLocalGet<U0>,
        OpI32Eqz,
        OpIf<tarr![OpI32Const<U0>, OpReturn], TTerm>,
        OpLocalGet<U0>,
        OpI32Const<U1>,
        OpI32Sub,
        OpCall<U0>,
        OpReturn
    ],
>;
type EvenOddModule = WasmModule<tarr![IsEven, IsOdd], ZeroPages>;

#[test]
fn mutual_recursion_even_odd_executes_correctly() {
    type EvenProgram = tarr![OpI32Const<U6>, OpCall<U0>];
    type OddProgram = tarr![OpI32Const<U5>, OpCall<U1>];

    type EvenState = ModuleProgramRun<EvenOddModule, EvenProgram>;
    type OddState = ModuleProgramRun<EvenOddModule, OddProgram>;

    assert_type_eq_all!(<EvenState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<OddState as StateStack>::Output, tarr![WasmI32<U1>]);
}
