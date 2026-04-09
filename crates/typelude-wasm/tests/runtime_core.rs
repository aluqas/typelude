#![recursion_limit = "65536"]

mod support;

use static_assertions::assert_type_eq_all;
use typelude_col::{TTerm, tarr};
use typelude_std::core::Evaluate;
use typelude_wasm::{
    RunWasm, WasmFunc, WasmI32, WasmMemory, WasmState,
    opcode::{
        OpBlock, OpBr, OpBrIf, OpCall, OpI32Add, OpI32Const, OpI32Eqz, OpI32Load, OpI32Load8U,
        OpI32Store, OpI32Store8, OpI32Sub, OpIf, OpLocalGet, OpLocalSet, OpLocalTee, OpLoop,
        OpMemorySize, OpReturn, OpSelect,
    },
};
use typenum::{U0, U1, U2, U3, U5, U6, U8, U9, U258};

use crate::support::{
    ProgramRun, StateBranches, StateLocals, StateMemory, StateProgram, StateStack,
};

type ZeroPages = WasmMemory<U0, TTerm>;
type OnePage = WasmMemory<U1, TTerm>;

#[test]
fn const_and_add_produce_expected_stack() {
    type Program = tarr![OpI32Const<U1>, OpI32Const<U2>, OpI32Add];
    type FinalState = ProgramRun<Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U3>]);
    assert_type_eq_all!(<FinalState as StateProgram>::Output, TTerm);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

#[test]
fn sub_uses_wasm_operand_order() {
    type Program = tarr![OpI32Const<U5>, OpI32Const<U2>, OpI32Sub];
    type FinalState = ProgramRun<Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U3>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

#[test]
fn eqz_returns_canonical_i32_boolean() {
    type ZeroProgram = tarr![OpI32Const<U0>, OpI32Eqz];
    type NonZeroProgram = tarr![OpI32Const<U5>, OpI32Eqz];

    type ZeroState = ProgramRun<ZeroProgram>;
    type NonZeroState = ProgramRun<NonZeroProgram>;

    assert_type_eq_all!(<ZeroState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<NonZeroState as StateStack>::Output, tarr![WasmI32<U0>]);
}

#[test]
fn local_set_and_get_round_trip() {
    type Program = tarr![OpI32Const<U5>, OpLocalSet<U0>, OpLocalGet<U0>];
    type InitialState = WasmState<TTerm, tarr![WasmI32<U0>], ZeroPages, TTerm, TTerm, Program>;
    type FinalState = Evaluate<RunWasm<InitialState>>;

    assert_type_eq_all!(<FinalState as StateLocals>::Output, tarr![WasmI32<U5>]);
    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U5>]);
}

#[test]
fn local_tee_updates_local_and_preserves_stack() {
    type Program = tarr![OpI32Const<U5>, OpLocalTee<U0>];
    type InitialState = WasmState<TTerm, tarr![WasmI32<U0>], ZeroPages, TTerm, TTerm, Program>;
    type FinalState = Evaluate<RunWasm<InitialState>>;

    assert_type_eq_all!(<FinalState as StateLocals>::Output, tarr![WasmI32<U5>]);
    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U5>]);
}

type AddTwoAndReturn = WasmFunc<U0, TTerm, tarr![OpI32Const<U2>, OpI32Add, OpReturn]>;
type OuterAddAndReturn =
    WasmFunc<U0, TTerm, tarr![OpCall<AddTwoAndReturn>, OpI32Const<U3>, OpI32Add, OpReturn]>;

#[test]
fn simple_call_and_return_resume_caller_continuation() {
    type Program = tarr![OpI32Const<U1>, OpCall<AddTwoAndReturn>, OpI32Const<U3>, OpI32Add];
    type FinalState = ProgramRun<Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U6>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

#[test]
fn nested_calls_produce_expected_result() {
    type Program = tarr![OpI32Const<U1>, OpCall<OuterAddAndReturn>];
    type FinalState = ProgramRun<Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U6>]);
}

type ShadowLocalAndReturn =
    WasmFunc<U0, tarr![WasmI32<U0>], tarr![OpI32Const<U9>, OpLocalSet<U0>, OpReturn]>;

#[test]
fn callee_locals_do_not_leak_back_to_caller() {
    type Program = tarr![OpCall<ShadowLocalAndReturn>, OpLocalGet<U0>];
    type InitialState = WasmState<TTerm, tarr![WasmI32<U1>], ZeroPages, TTerm, TTerm, Program>;
    type FinalState = Evaluate<RunWasm<InitialState>>;

    assert_type_eq_all!(<FinalState as StateLocals>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
}

type FallthroughAdd = WasmFunc<U0, TTerm, tarr![OpI32Const<U2>, OpI32Add]>;

#[test]
fn callee_fallthrough_returns_via_end_func() {
    type Program = tarr![OpI32Const<U1>, OpCall<FallthroughAdd>, OpI32Const<U3>, OpI32Add];
    type FinalState = ProgramRun<Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U6>]);
}

#[test]
fn if_true_branch_executes_then_program() {
    type Program = tarr![OpI32Const<U1>, OpIf<tarr![OpI32Const<U2>], tarr![OpI32Const<U3>]>];
    type FinalState = ProgramRun<Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U2>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

#[test]
fn if_false_branch_executes_else_program() {
    type Program = tarr![OpI32Const<U0>, OpIf<tarr![OpI32Const<U2>], tarr![OpI32Const<U3>]>];
    type FinalState = ProgramRun<Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U3>]);
}

#[test]
fn select_chooses_true_value_on_nonzero_condition() {
    type Program = tarr![OpI32Const<U2>, OpI32Const<U3>, OpI32Const<U1>, OpSelect];
    type FinalState = ProgramRun<Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U2>]);
}

#[test]
fn select_chooses_false_value_on_zero_condition() {
    type Program = tarr![OpI32Const<U2>, OpI32Const<U3>, OpI32Const<U0>, OpSelect];
    type FinalState = ProgramRun<Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U3>]);
}

#[test]
fn block_fallthrough_leaves_branch_stack_empty() {
    type Program = tarr![OpBlock<tarr![OpI32Const<U1>]>];
    type FinalState = ProgramRun<Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

#[test]
fn br_zero_exits_innermost_block() {
    type Program = tarr![OpBlock<tarr![OpBr<U0>, OpI32Const<U9>]>, OpI32Const<U1>];
    type FinalState = ProgramRun<Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

#[test]
fn nested_block_br_one_exits_outer_block() {
    type Program = tarr![
        OpBlock<tarr![OpBlock<tarr![OpBr<U1>, OpI32Const<U9>]>, OpI32Const<U8>]>,
        OpI32Const<U1>
    ];
    type FinalState = ProgramRun<Program>;

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
    type InitialState = WasmState<TTerm, tarr![WasmI32<U0>], ZeroPages, TTerm, TTerm, Program>;
    type FinalState = Evaluate<RunWasm<InitialState>>;

    assert_type_eq_all!(<FinalState as StateLocals>::Output, tarr![WasmI32<U0>]);
    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U0>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

#[test]
fn loop_fallthrough_pops_active_loop_label() {
    type Program = tarr![OpLoop<tarr![OpI32Const<U1>]>];
    type FinalState = ProgramRun<Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

type BranchingCallee = WasmFunc<U0, TTerm, tarr![OpBlock<tarr![OpBr<U0>]>, OpReturn]>;

#[test]
fn calls_inside_control_flow_restore_caller_branches() {
    type Program =
        tarr![OpBlock<tarr![OpCall<BranchingCallee>, OpBr<U0>, OpI32Const<U9>]>, OpI32Const<U1>];
    type FinalState = ProgramRun<Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

type ScopedBranchCallee = WasmFunc<U0, TTerm, tarr![OpBlock<tarr![OpI32Const<U1>]>, OpReturn]>;

type AddParamsAndReturn = WasmFunc<
    U2,
    TTerm,
    tarr![OpLocalGet<U0>, OpLocalGet<U1>, OpI32Add, OpReturn],
>;

#[test]
fn call_binds_params_from_stack_in_wasm_order() {
    type Program = tarr![OpI32Const<U2>, OpI32Const<U3>, OpCall<AddParamsAndReturn>];
    type FinalState = ProgramRun<Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U5>]);
}

type ZeroInitLocalAndReturn = WasmFunc<U0, tarr![WasmI32<U0>], tarr![OpLocalGet<U0>, OpReturn]>;

#[test]
fn extra_locals_are_zero_initialized() {
    type Program = tarr![OpCall<ZeroInitLocalAndReturn>];
    type FinalState = ProgramRun<Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U0>]);
}

#[test]
fn callee_branch_stack_does_not_leak_back_to_caller() {
    type Program = tarr![OpCall<ScopedBranchCallee>];
    type FinalState = ProgramRun<Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

#[test]
fn memory_size_reports_initial_page_count() {
    type Program = tarr![OpMemorySize];
    type InitialState = WasmState<TTerm, TTerm, OnePage, TTerm, TTerm, Program>;
    type FinalState = Evaluate<RunWasm<InitialState>>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateMemory>::Output, OnePage);
}

#[test]
fn load8_u_of_unwritten_in_bounds_byte_returns_zero() {
    type Program = tarr![OpI32Const<U0>, OpI32Load8U];
    type InitialState = WasmState<TTerm, TTerm, OnePage, TTerm, TTerm, Program>;
    type FinalState = Evaluate<RunWasm<InitialState>>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U0>]);
    assert_type_eq_all!(<FinalState as StateMemory>::Output, OnePage);
}

#[test]
fn load_of_unwritten_in_bounds_word_returns_zero() {
    type Program = tarr![OpI32Const<U0>, OpI32Load];
    type InitialState = WasmState<TTerm, TTerm, OnePage, TTerm, TTerm, Program>;
    type FinalState = Evaluate<RunWasm<InitialState>>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U0>]);
}

#[test]
fn store8_then_load8_u_round_trips_low_byte() {
    type Program =
        tarr![OpI32Const<U0>, OpI32Const<U258>, OpI32Store8, OpI32Const<U0>, OpI32Load8U];
    type InitialState = WasmState<TTerm, TTerm, OnePage, TTerm, TTerm, Program>;
    type FinalState = Evaluate<RunWasm<InitialState>>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U2>]);
}

#[test]
fn store_then_load_round_trips_i32_value() {
    type Program = tarr![OpI32Const<U0>, OpI32Const<U258>, OpI32Store, OpI32Const<U0>, OpI32Load];
    type InitialState = WasmState<TTerm, TTerm, OnePage, TTerm, TTerm, Program>;
    type FinalState = Evaluate<RunWasm<InitialState>>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U258>]);
}

#[test]
fn store_uses_little_endian_layout() {
    type Program =
        tarr![OpI32Const<U0>, OpI32Const<U258>, OpI32Store, OpI32Const<U1>, OpI32Load8U];
    type InitialState = WasmState<TTerm, TTerm, OnePage, TTerm, TTerm, Program>;
    type FinalState = Evaluate<RunWasm<InitialState>>;

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
    type InitialState = WasmState<TTerm, TTerm, OnePage, TTerm, TTerm, Program>;
    type FinalState = Evaluate<RunWasm<InitialState>>;

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
    type InitialState =
        WasmState<TTerm, tarr![WasmI32<U0>, WasmI32<U0>], OnePage, TTerm, TTerm, Program>;
    type FinalState = Evaluate<RunWasm<InitialState>>;

    assert_type_eq_all!(<FinalState as StateLocals>::Output, tarr![WasmI32<U0>, WasmI32<U3>]);
    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U2>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}
