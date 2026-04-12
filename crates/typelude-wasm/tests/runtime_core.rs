#![recursion_limit = "65536"]

mod support;

use core::ops::Add;

use static_assertions::assert_type_eq_all;
use typelude_col::{TTerm, tarr};
use typelude_std::core::Evaluate;
use typelude_wasm::{
    ExportFunc, ExportGlobal, ExportMemory, ExportTable, GlobalConst, GlobalMut, HostCall,
    HostCallResult, HostFuncBinding, HostGlobalBinding, HostMemoryBinding, HostTableBinding,
    ImportFunc, ImportGlobal, ImportMemory, ImportTable, InitGlobalGet, InitI32Const,
    InvokeFuncWithEnv, ModuleProgramRun, NoLimit, NoStart, RunWasm, StartFunc, TableEntry,
    MemoryCell, WasmDataSegment, WasmElemSegment, WasmExport, WasmFunc, WasmFuncSpace,
    WasmFuncType, WasmGlobal, WasmGlobalDecl, WasmHostEnv, WasmI32, WasmI32Type, WasmImport,
    WasmMemory, WasmMemoryDecl, WasmModule, WasmResolvedModule, WasmState, WasmStore, WasmTable,
    WasmTableDecl,
    StateExportGlobal, StateExportMemory, StateExportTable, StateTables,
    opcode::{
        OpBlock, OpBr, OpBrIf, OpCall, OpCallIndirect, OpDrop, OpGlobalGet, OpGlobalSet,
        OpI32Add, OpI32Const, OpI32Eqz, OpI32Load, OpI32Load8U, OpI32Store, OpI32Store8,
        OpI32Sub, OpIf, OpLocalGet, OpLocalSet, OpLocalTee, OpLoop, OpMemoryGrow, OpMemorySize,
        OpReturn, OpSelect,
    },
};
use typenum::{Const, ToUInt, U0, U1, U2, U3, U5, U6, U7, U8, U9, U42, U258, operator_aliases::Sum};

use crate::support::{
    StateBranches, StateGlobals, StateLocals, StateMemory, StateProgram, StateStack,
};

type MaxPages = <Const<4294967295> as ToUInt>::Output;
type PageBytes = <Const<65536> as ToUInt>::Output;
type LittleEndian258Cells = tarr![
    MemoryCell<U3, U0>,
    MemoryCell<U2, U0>,
    MemoryCell<U1, U1>,
    MemoryCell<U0, U2>
];
type ZeroPages = WasmMemory<U0, MaxPages, TTerm>;
type OnePage = WasmMemory<U1, MaxPages, TTerm>;
type Store<Memory> = WasmStore<Memory, TTerm, TTerm>;
type Resolved<Funcs> = WasmResolvedModule<Funcs, TTerm, TTerm>;
type ModuleWithDecls<Funcs, MinPages, MaxPageLimit, DataSegments, GlobalsDecl> = WasmModule<
    TTerm,
    WasmFuncSpace<TTerm, Funcs>,
    WasmMemoryDecl<MinPages, MaxPageLimit, DataSegments>,
    TTerm,
    GlobalsDecl,
    TTerm,
    NoStart,
>;
type Module<Funcs, MinPages> = ModuleWithDecls<Funcs, MinPages, NoLimit, TTerm, TTerm>;
type ModuleWithGlobals<Funcs, MinPages, GlobalsDecl> =
    ModuleWithDecls<Funcs, MinPages, NoLimit, TTerm, GlobalsDecl>;
type ModuleWithMemoryLimits<Funcs, MinPages, MaxPageLimit> =
    ModuleWithDecls<Funcs, MinPages, MaxPageLimit, TTerm, TTerm>;
type ModuleWithData<Funcs, MinPages, DataSegments> =
    ModuleWithDecls<Funcs, MinPages, NoLimit, DataSegments, TTerm>;
type ModuleWithDataAndGlobals<Funcs, MinPages, DataSegments, GlobalsDecl> =
    ModuleWithDecls<Funcs, MinPages, NoLimit, DataSegments, GlobalsDecl>;
type ModuleWithTables<Funcs, Types, TablesDecl> = WasmModule<
    TTerm,
    WasmFuncSpace<Types, Funcs>,
    WasmMemoryDecl<U0, NoLimit, TTerm>,
    TablesDecl,
    TTerm,
    TTerm,
    NoStart,
>;
type InitialState<Funcs, Memory, Locals, Program> =
    WasmState<Resolved<Funcs>, Store<Memory>, TTerm, Locals, TTerm, TTerm, Program>;

type EmptyModule = Module<TTerm, U0>;
type OnePageModule = Module<TTerm, U1>;
type Fn0<LocalInits, Program> = WasmFunc<WasmFuncType<TTerm, TTerm>, LocalInits, Program>;
type Fn1<LocalInits, Program> =
    WasmFunc<WasmFuncType<tarr![WasmI32Type], TTerm>, LocalInits, Program>;
type Fn2<LocalInits, Program> =
    WasmFunc<WasmFuncType<tarr![WasmI32Type, WasmI32Type], TTerm>, LocalInits, Program>;

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
    type TestState = InitialState<TTerm, ZeroPages, tarr![WasmI32<U0>], Program>;
    type FinalState = Evaluate<RunWasm<TestState>>;

    assert_type_eq_all!(<FinalState as StateLocals>::Output, tarr![WasmI32<U5>]);
    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U5>]);
}

#[test]
fn local_tee_updates_local_and_preserves_stack() {
    type Program = tarr![OpI32Const<U5>, OpLocalTee<U0>];
    type TestState = InitialState<TTerm, ZeroPages, tarr![WasmI32<U0>], Program>;
    type FinalState = Evaluate<RunWasm<TestState>>;

    assert_type_eq_all!(<FinalState as StateLocals>::Output, tarr![WasmI32<U5>]);
    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U5>]);
}

type AddTwoAndReturn = Fn0<TTerm, tarr![OpI32Const<U2>, OpI32Add, OpReturn]>;
type SimpleCallModule = Module<tarr![AddTwoAndReturn], U0>;

type OuterAddAndReturn =
    Fn0<TTerm, tarr![OpCall<U1>, OpI32Const<U3>, OpI32Add, OpReturn]>;
type NestedCallModule = Module<tarr![OuterAddAndReturn, AddTwoAndReturn], U0>;

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
    Fn0<tarr![WasmI32<U0>], tarr![OpI32Const<U9>, OpLocalSet<U0>, OpReturn]>;
type ShadowLocalModule = Module<tarr![ShadowLocalAndReturn], U0>;

#[test]
fn callee_locals_do_not_leak_back_to_caller() {
    type Program = tarr![OpCall<U0>, OpLocalGet<U0>];
    type TestState = InitialState<tarr![ShadowLocalAndReturn], ZeroPages, tarr![WasmI32<U1>], Program>;
    type FinalState = Evaluate<RunWasm<TestState>>;

    assert_type_eq_all!(<FinalState as StateLocals>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
}

type FallthroughAdd = Fn0<TTerm, tarr![OpI32Const<U2>, OpI32Add]>;
type FallthroughModule = Module<tarr![FallthroughAdd], U0>;

#[test]
fn callee_fallthrough_returns_via_end_func() {
    type Program = tarr![OpI32Const<U1>, OpCall<U0>, OpI32Const<U3>, OpI32Add];
    type FinalState = ModuleProgramRun<FallthroughModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U6>]);
}

type ConstGlobalDecls = tarr![WasmGlobalDecl<GlobalConst, InitI32Const<U5>>];
type ConstGlobalModule = ModuleWithGlobals<TTerm, U0, ConstGlobalDecls>;
type MutableGlobalDecls = tarr![WasmGlobalDecl<GlobalMut, InitI32Const<U3>>];
type MutableGlobalModule = ModuleWithGlobals<TTerm, U0, MutableGlobalDecls>;

#[test]
fn declared_globals_are_materialized_in_declaration_order() {
    type GlobalDecls = tarr![
        WasmGlobalDecl<GlobalConst, InitI32Const<U1>>,
        WasmGlobalDecl<GlobalMut, InitI32Const<U2>>
    ];
    type GlobalModule = ModuleWithGlobals<TTerm, U0, GlobalDecls>;
    type FinalState = ModuleProgramRun<GlobalModule, TTerm>;

    assert_type_eq_all!(
        <FinalState as StateGlobals>::Output,
        tarr![WasmGlobal<GlobalConst, WasmI32<U1>>, WasmGlobal<GlobalMut, WasmI32<U2>>]
    );
}

#[test]
fn const_global_initialized_with_i32_const_is_observable_via_global_get() {
    type Program = tarr![OpGlobalGet<U0>];
    type FinalState = ModuleProgramRun<ConstGlobalModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U5>]);
    assert_type_eq_all!(
        <FinalState as StateGlobals>::Output,
        tarr![WasmGlobal<GlobalConst, WasmI32<U5>>]
    );
}

#[test]
fn mutable_global_initialized_with_i32_const_is_observable_via_global_get() {
    type Program = tarr![OpGlobalGet<U0>];
    type FinalState = ModuleProgramRun<MutableGlobalModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U3>]);
    assert_type_eq_all!(
        <FinalState as StateGlobals>::Output,
        tarr![WasmGlobal<GlobalMut, WasmI32<U3>>]
    );
}

#[test]
fn global_set_updates_mutable_global() {
    type Program = tarr![OpI32Const<U9>, OpGlobalSet<U0>, OpGlobalGet<U0>];
    type FinalState = ModuleProgramRun<MutableGlobalModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U9>]);
    assert_type_eq_all!(
        <FinalState as StateGlobals>::Output,
        tarr![WasmGlobal<GlobalMut, WasmI32<U9>>]
    );
}

type ReadGlobalAndLocal = Fn0<
    tarr![WasmI32<U0>],
    tarr![OpI32Const<U9>, OpLocalSet<U0>, OpGlobalGet<U0>, OpLocalGet<U0>, OpReturn],
>;
type LocalAndGlobalModule =
    ModuleWithGlobals<tarr![ReadGlobalAndLocal], U0, tarr![WasmGlobalDecl<GlobalMut, InitI32Const<U7>>]>;

#[test]
fn locals_and_globals_remain_independent() {
    type Program = tarr![OpCall<U0>];
    type FinalState = ModuleProgramRun<LocalAndGlobalModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U9>, WasmI32<U7>]);
    assert_type_eq_all!(
        <FinalState as StateGlobals>::Output,
        tarr![WasmGlobal<GlobalMut, WasmI32<U7>>]
    );
}

type SetGlobalInCallee = Fn0<TTerm, tarr![OpI32Const<U9>, OpGlobalSet<U0>, OpReturn]>;
type GlobalCallModule =
    ModuleWithGlobals<tarr![SetGlobalInCallee], U0, tarr![WasmGlobalDecl<GlobalMut, InitI32Const<U3>>]>;

#[test]
fn global_state_survives_function_calls_and_returns() {
    type Program = tarr![OpCall<U0>, OpGlobalGet<U0>];
    type FinalState = ModuleProgramRun<GlobalCallModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U9>]);
    assert_type_eq_all!(
        <FinalState as StateGlobals>::Output,
        tarr![WasmGlobal<GlobalMut, WasmI32<U9>>]
    );
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
    type TestState = InitialState<TTerm, ZeroPages, tarr![WasmI32<U0>], Program>;
    type FinalState = Evaluate<RunWasm<TestState>>;

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

type BranchingCallee = Fn0<TTerm, tarr![OpBlock<tarr![OpBr<U0>]>, OpReturn]>;
type BranchingModule = Module<tarr![BranchingCallee], U0>;

#[test]
fn calls_inside_control_flow_restore_caller_branches() {
    type Program = tarr![OpBlock<tarr![OpCall<U0>, OpBr<U0>, OpI32Const<U9>]>, OpI32Const<U1>];
    type FinalState = ModuleProgramRun<BranchingModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

type ScopedBranchCallee = Fn0<TTerm, tarr![OpBlock<tarr![OpI32Const<U1>]>, OpReturn]>;
type ScopedBranchModule = Module<tarr![ScopedBranchCallee], U0>;

type AddParamsAndReturn =
    Fn2<TTerm, tarr![OpLocalGet<U0>, OpLocalGet<U1>, OpI32Add, OpReturn]>;
type AddParamsModule = Module<tarr![AddParamsAndReturn], U0>;

#[test]
fn call_binds_params_from_stack_in_wasm_order() {
    type Program = tarr![OpI32Const<U2>, OpI32Const<U3>, OpCall<U0>];
    type FinalState = ModuleProgramRun<AddParamsModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U5>]);
}

type ZeroInitLocalAndReturn = Fn0<tarr![WasmI32<U0>], tarr![OpLocalGet<U0>, OpReturn]>;
type ZeroInitLocalModule = Module<tarr![ZeroInitLocalAndReturn], U0>;

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
    type Program = tarr![OpI32Const<U0>, OpI32Load8U<U0>];
    type FinalState = ModuleProgramRun<OnePageModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U0>]);
    assert_type_eq_all!(<FinalState as StateMemory>::Output, OnePage);
}

#[test]
fn load_of_unwritten_in_bounds_word_returns_zero() {
    type Program = tarr![OpI32Const<U0>, OpI32Load<U0>];
    type FinalState = ModuleProgramRun<OnePageModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U0>]);
}

type GrowOneToTwoPagesModule = ModuleWithMemoryLimits<TTerm, U1, U2>;
type GrowTwoToThreePagesModule = ModuleWithMemoryLimits<TTerm, U2, U3>;
type FixedOnePageModule = ModuleWithMemoryLimits<TTerm, U1, U1>;

#[test]
fn memory_grow_within_max_returns_old_page_count_and_updates_memory() {
    type Program = tarr![OpI32Const<U1>, OpMemoryGrow, OpMemorySize];
    type FinalState = ModuleProgramRun<GrowOneToTwoPagesModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U2>, WasmI32<U1>]);
    assert_type_eq_all!(<FinalState as StateMemory>::Output, WasmMemory<U2, U2, TTerm>);
}

#[test]
fn memory_grow_zero_returns_old_page_count_and_leaves_memory_unchanged() {
    type Program = tarr![OpI32Const<U0>, OpMemoryGrow, OpMemorySize];
    type FinalState = ModuleProgramRun<GrowTwoToThreePagesModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U2>, WasmI32<U2>]);
    assert_type_eq_all!(<FinalState as StateMemory>::Output, WasmMemory<U2, U3, TTerm>);
}

#[test]
fn memory_grow_beyond_max_returns_failure_and_leaves_memory_unchanged() {
    type Program = tarr![OpI32Const<U1>, OpMemoryGrow];
    type FinalState = ModuleProgramRun<FixedOnePageModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<MaxPages>]);
    assert_type_eq_all!(<FinalState as StateMemory>::Output, WasmMemory<U1, U1, TTerm>);
}

#[test]
fn store8_then_load8_u_round_trips_low_byte() {
    type Program =
        tarr![OpI32Const<U0>, OpI32Const<U258>, OpI32Store8<U0>, OpI32Const<U0>, OpI32Load8U<U0>];
    type FinalState = ModuleProgramRun<OnePageModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U2>]);
}

#[test]
fn store_then_load_round_trips_i32_value() {
    type Program = tarr![OpI32Const<U0>, OpI32Const<U258>, OpI32Store<U0>, OpI32Const<U0>, OpI32Load<U0>];
    type FinalState = ModuleProgramRun<OnePageModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U258>]);
}

#[test]
fn store_uses_little_endian_layout() {
    type Program =
        tarr![OpI32Const<U0>, OpI32Const<U258>, OpI32Store<U0>, OpI32Const<U1>, OpI32Load8U<U0>];
    type FinalState = ModuleProgramRun<OnePageModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U1>]);
}

#[test]
fn later_byte_writes_shadow_earlier_ones() {
    type Program = tarr![
        OpI32Const<U0>,
        OpI32Const<U1>,
        OpI32Store8<U0>,
        OpI32Const<U0>,
        OpI32Const<U2>,
        OpI32Store8<U0>,
        OpI32Const<U0>,
        OpI32Load8U<U0>
    ];
    type FinalState = ModuleProgramRun<OnePageModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U2>]);
}

#[test]
fn bytes_written_before_grow_remain_readable_after_grow() {
    type Program = tarr![
        OpI32Const<U0>,
        OpI32Const<U258>,
        OpI32Store<U0>,
        OpI32Const<U1>,
        OpMemoryGrow,
        OpDrop,
        OpI32Const<U0>,
        OpI32Load<U0>
    ];
    type FinalState = ModuleProgramRun<GrowOneToTwoPagesModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U258>]);
    assert_type_eq_all!(
        <FinalState as StateMemory>::Output,
        WasmMemory<U2, U2, LittleEndian258Cells>
    );
}

#[test]
fn newly_grown_region_reads_as_zero_before_any_write() {
    type Program = tarr![
        OpI32Const<U1>,
        OpMemoryGrow,
        OpDrop,
        OpI32Const<PageBytes>,
        OpI32Load8U<U0>
    ];
    type FinalState = ModuleProgramRun<GrowOneToTwoPagesModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U0>]);
}

#[test]
fn write_after_grow_succeeds_in_newly_valid_range() {
    type Program = tarr![
        OpI32Const<U1>,
        OpMemoryGrow,
        OpDrop,
        OpI32Const<PageBytes>,
        OpI32Const<U7>,
        OpI32Store8<U0>,
        OpI32Const<PageBytes>,
        OpI32Load8U<U0>
    ];
    type FinalState = ModuleProgramRun<GrowOneToTwoPagesModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U7>]);
}

type PreloadedDataModule = ModuleWithData<
    TTerm,
    U1,
    tarr![WasmDataSegment<InitI32Const<U2>, tarr![U7, U8]>]
>;

#[test]
fn active_data_segment_preloads_memory_before_first_instruction() {
    type Program = tarr![OpI32Const<U2>, OpI32Load8U<U0>];
    type FinalState = ModuleProgramRun<PreloadedDataModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U7>]);
}

type SplitDataModule = ModuleWithData<
    TTerm,
    U1,
    tarr![
        WasmDataSegment<InitI32Const<U0>, tarr![U1]>,
        WasmDataSegment<InitI32Const<U1>, tarr![U2]>
    ]
>;

#[test]
fn multiple_data_segments_apply_in_declaration_order() {
    type Program = tarr![OpI32Const<U0>, OpI32Load8U<U0>, OpI32Const<U1>, OpI32Load8U<U0>];
    type FinalState = ModuleProgramRun<SplitDataModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U2>, WasmI32<U1>]);
}

type OverlappingDataModule = ModuleWithData<
    TTerm,
    U1,
    tarr![
        WasmDataSegment<InitI32Const<U0>, tarr![U1, U2]>,
        WasmDataSegment<InitI32Const<U1>, tarr![U9]>
    ]
>;

#[test]
fn overlapping_data_segments_use_last_write_wins() {
    type Program = tarr![OpI32Const<U1>, OpI32Load8U<U0>];
    type FinalState = ModuleProgramRun<OverlappingDataModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U9>]);
}

type WordDataModule = ModuleWithData<
    TTerm,
    U1,
    tarr![WasmDataSegment<InitI32Const<U0>, tarr![U2, U1, U0, U0]>]
>;

#[test]
fn load_can_read_a_word_backed_entirely_by_instantiated_data() {
    type Program = tarr![OpI32Const<U0>, OpI32Load<U0>];
    type FinalState = ModuleProgramRun<WordDataModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U258>]);
}

type DataAndGlobalsModule = ModuleWithDataAndGlobals<
    TTerm,
    U1,
    tarr![WasmDataSegment<InitI32Const<U0>, tarr![U42]>],
    tarr![WasmGlobalDecl<GlobalMut, InitI32Const<U5>>]
>;

#[test]
fn declared_globals_and_data_backed_memory_can_coexist() {
    type Program = tarr![OpGlobalGet<U0>, OpI32Const<U0>, OpI32Load8U<U0>];
    type FinalState = ModuleProgramRun<DataAndGlobalsModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U42>, WasmI32<U5>]);
    assert_type_eq_all!(
        <FinalState as StateGlobals>::Output,
        tarr![WasmGlobal<GlobalMut, WasmI32<U5>>]
    );
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
                        OpI32Store8<U0>,
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
        OpI32Load8U<U0>
    ];
    type TestState =
        InitialState<TTerm, OnePage, tarr![WasmI32<U0>, WasmI32<U0>], Program>;
    type FinalState = Evaluate<RunWasm<TestState>>;

    assert_type_eq_all!(<FinalState as StateLocals>::Output, tarr![WasmI32<U0>, WasmI32<U3>]);
    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U2>]);
    assert_type_eq_all!(<FinalState as StateBranches>::Output, TTerm);
}

type RecursiveSumDown = Fn1<
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
type RecursiveSumModule = Module<tarr![RecursiveSumDown], U0>;

#[test]
fn self_recursive_calls_execute_correctly() {
    type Program = tarr![OpI32Const<U3>, OpCall<U0>];
    type FinalState = ModuleProgramRun<RecursiveSumModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U6>]);
}

type IsEven = Fn1<
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
type IsOdd = Fn1<
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
type EvenOddModule = Module<tarr![IsEven, IsOdd], U0>;

#[test]
fn mutual_recursion_even_odd_executes_correctly() {
    type EvenProgram = tarr![OpI32Const<U6>, OpCall<U0>];
    type OddProgram = tarr![OpI32Const<U5>, OpCall<U1>];

    type EvenState = ModuleProgramRun<EvenOddModule, EvenProgram>;
    type OddState = ModuleProgramRun<EvenOddModule, OddProgram>;

    assert_type_eq_all!(<EvenState as StateStack>::Output, tarr![WasmI32<U1>]);
    assert_type_eq_all!(<OddState as StateStack>::Output, tarr![WasmI32<U1>]);
}

type ReturnsOne = WasmFunc<WasmFuncType<TTerm, tarr![WasmI32Type]>, TTerm, tarr![OpI32Const<U1>, OpReturn]>;
type ReturnsTwo = WasmFunc<WasmFuncType<TTerm, tarr![WasmI32Type]>, TTerm, tarr![OpI32Const<U2>, OpReturn]>;
type IndirectTypes = tarr![WasmFuncType<TTerm, tarr![WasmI32Type]>];
type DefaultTableDecls = tarr![WasmTableDecl<
    U2,
    U2,
    tarr![WasmElemSegment<U0, InitI32Const<U0>, tarr![U0, U1]>]
>];
type DefaultTableModule = ModuleWithTables<tarr![ReturnsOne, ReturnsTwo], IndirectTypes, DefaultTableDecls>;

#[test]
fn active_elem_initializes_default_table_slots() {
    type FinalState = ModuleProgramRun<DefaultTableModule, TTerm>;

    assert_type_eq_all!(
        <FinalState as StateTables>::Output,
        tarr![WasmTable<U2, U2, tarr![TableEntry<U1, U1>, TableEntry<U0, U0>]>]
    );
}

#[test]
fn call_indirect_dispatches_through_default_table() {
    type Program = tarr![OpI32Const<U1>, OpCallIndirect<U0>];
    type FinalState = ModuleProgramRun<DefaultTableModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U2>]);
}

type OverlappingTableDecls = tarr![WasmTableDecl<
    U1,
    U1,
    tarr![
        WasmElemSegment<U0, InitI32Const<U0>, tarr![U0]>,
        WasmElemSegment<U0, InitI32Const<U0>, tarr![U1]>
    ]
>];
type OverlappingTableModule =
    ModuleWithTables<tarr![ReturnsOne, ReturnsTwo], IndirectTypes, OverlappingTableDecls>;

#[test]
fn overlapping_elem_segments_use_last_write_wins() {
    type Program = tarr![OpI32Const<U0>, OpCallIndirect<U0>];
    type FinalState = ModuleProgramRun<OverlappingTableModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U2>]);
}

type ReturnsThree =
    WasmFunc<WasmFuncType<TTerm, tarr![WasmI32Type]>, TTerm, tarr![OpI32Const<U3>, OpReturn]>;
type MultiTableDecls = tarr![
    WasmTableDecl<U1, U1, tarr![WasmElemSegment<U0, InitI32Const<U0>, tarr![U0]>]>,
    WasmTableDecl<U1, U1, tarr![WasmElemSegment<U1, InitI32Const<U0>, tarr![U1]>]>
];
type MultiTableModule =
    ModuleWithTables<tarr![ReturnsOne, ReturnsThree], IndirectTypes, MultiTableDecls>;

#[test]
fn explicit_table_index_is_respected_for_call_indirect() {
    type Program = tarr![OpI32Const<U0>, OpCallIndirect<U0, U1>];
    type FinalState = ModuleProgramRun<MultiTableModule, Program>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U3>]);
}

struct HostAdd;

impl<Memory, Tables, Globals, A, B>
    HostCall<
        WasmFuncType<tarr![WasmI32Type, WasmI32Type], tarr![WasmI32Type]>,
        WasmStore<Memory, Tables, Globals>,
        tarr![WasmI32<A>, WasmI32<B>],
    > for HostAdd
where
    A: Add<B>,
{
    type Output = HostCallResult<WasmStore<Memory, Tables, Globals>, tarr![WasmI32<Sum<A, B>>]>;
}

type ImportAddMain = Fn2<TTerm, tarr![OpLocalGet<U0>, OpLocalGet<U1>, OpCall<U0>, OpReturn]>;
type ImportAddModule = WasmModule<
    tarr![WasmImport<
        typelude_wasm::tstr::TS!("host"),
        typelude_wasm::tstr::TS!("add"),
        ImportFunc<WasmFuncType<tarr![WasmI32Type, WasmI32Type], tarr![WasmI32Type]>>,
    >],
    WasmFuncSpace<
        TTerm,
        tarr![ImportAddMain],
    >,
    WasmMemoryDecl<U0, NoLimit, TTerm>,
    TTerm,
    TTerm,
    tarr![WasmExport<typelude_wasm::tstr::TS!("main"), ExportFunc<U1>>],
    NoStart,
>;
type ImportAddEnv = WasmHostEnv<
    tarr![HostFuncBinding<
        typelude_wasm::tstr::TS!("host"),
        typelude_wasm::tstr::TS!("add"),
        HostAdd,
    >],
    TTerm,
    TTerm,
    TTerm,
>;

#[test]
fn imported_function_calls_resolve_through_host_env() {
    type FinalState = InvokeFuncWithEnv<ImportAddModule, ImportAddEnv, U1, tarr![WasmI32<U2>, WasmI32<U3>]>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U5>]);
}

type ImportedGlobalsModule = WasmModule<
    tarr![
        WasmImport<
            typelude_wasm::tstr::TS!("host"),
            typelude_wasm::tstr::TS!("const_g"),
            ImportGlobal<GlobalConst, WasmI32Type>
        >,
        WasmImport<
            typelude_wasm::tstr::TS!("host"),
            typelude_wasm::tstr::TS!("mut_g"),
            ImportGlobal<GlobalMut, WasmI32Type>
        >
    ],
    WasmFuncSpace<
        TTerm,
        tarr![Fn0<
            TTerm,
            tarr![OpGlobalGet<U0>, OpGlobalGet<U1>, OpI32Add, OpGlobalSet<U1>, OpGlobalGet<U1>, OpReturn]
        >],
    >,
    WasmMemoryDecl<U0, NoLimit, TTerm>,
    TTerm,
    TTerm,
    tarr![WasmExport<typelude_wasm::tstr::TS!("main"), ExportFunc<U0>>],
    NoStart,
>;
type ImportedGlobalsEnv = WasmHostEnv<
    TTerm,
    tarr![
        HostGlobalBinding<
            typelude_wasm::tstr::TS!("host"),
            typelude_wasm::tstr::TS!("const_g"),
            WasmGlobal<GlobalConst, WasmI32<U2>>
        >,
        HostGlobalBinding<
            typelude_wasm::tstr::TS!("host"),
            typelude_wasm::tstr::TS!("mut_g"),
            WasmGlobal<GlobalMut, WasmI32<U3>>
        >
    ],
    TTerm,
    TTerm,
>;

#[test]
fn imported_globals_share_the_same_index_space_as_defined_code() {
    type FinalState = InvokeFuncWithEnv<ImportedGlobalsModule, ImportedGlobalsEnv, U0, TTerm>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U5>]);
    assert_type_eq_all!(
        <FinalState as StateGlobals>::Output,
        tarr![WasmGlobal<GlobalConst, WasmI32<U2>>, WasmGlobal<GlobalMut, WasmI32<U5>>]
    );
}

type ImportedGlobalInitModule = WasmModule<
    tarr![WasmImport<
        typelude_wasm::tstr::TS!("host"),
        typelude_wasm::tstr::TS!("base"),
        ImportGlobal<GlobalConst, WasmI32Type>
    >],
    WasmFuncSpace<TTerm, tarr![Fn0<TTerm, tarr![OpGlobalGet<U1>, OpReturn]>]>,
    WasmMemoryDecl<U0, NoLimit, TTerm>,
    TTerm,
    tarr![WasmGlobalDecl<GlobalConst, InitGlobalGet<U0>>],
    tarr![WasmExport<typelude_wasm::tstr::TS!("main"), ExportFunc<U0>>],
    NoStart,
>;
type ImportedGlobalInitEnv = WasmHostEnv<
    TTerm,
    tarr![HostGlobalBinding<
        typelude_wasm::tstr::TS!("host"),
        typelude_wasm::tstr::TS!("base"),
        WasmGlobal<GlobalConst, WasmI32<U7>>
    >],
    TTerm,
    TTerm,
>;

#[test]
fn imported_immutable_global_can_initialize_defined_globals() {
    type FinalState = InvokeFuncWithEnv<ImportedGlobalInitModule, ImportedGlobalInitEnv, U0, TTerm>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U7>]);
    assert_type_eq_all!(
        <FinalState as StateGlobals>::Output,
        tarr![WasmGlobal<GlobalConst, WasmI32<U7>>, WasmGlobal<GlobalConst, WasmI32<U7>>]
    );
}

#[test]
fn imported_immutable_global_is_available_to_data_offsets() {
    type Module = WasmModule<
        tarr![WasmImport<
            typelude_wasm::tstr::TS!("host"),
            typelude_wasm::tstr::TS!("base"),
            ImportGlobal<GlobalConst, WasmI32Type>
        >],
        WasmFuncSpace<TTerm, tarr![Fn0<TTerm, tarr![OpI32Const<U3>, OpI32Load8U<U0>, OpReturn]>]>,
        WasmMemoryDecl<
            U1,
            NoLimit,
            tarr![WasmDataSegment<InitGlobalGet<U0>, tarr![U7, U8]>]
        >,
        TTerm,
        TTerm,
        tarr![WasmExport<typelude_wasm::tstr::TS!("main"), ExportFunc<U0>>],
        NoStart
    >;
    type Env = WasmHostEnv<
        TTerm,
        tarr![HostGlobalBinding<
            typelude_wasm::tstr::TS!("host"),
            typelude_wasm::tstr::TS!("base"),
            WasmGlobal<GlobalConst, WasmI32<U2>>
        >],
        TTerm,
        TTerm,
    >;
    type FinalState = InvokeFuncWithEnv<Module, Env, U0, TTerm>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U8>]);
}

#[test]
fn imported_memory_receives_active_data_segments() {
    type Module = WasmModule<
        tarr![WasmImport<
            typelude_wasm::tstr::TS!("host"),
            typelude_wasm::tstr::TS!("memory"),
            ImportMemory<U1, U1>
        >],
        WasmFuncSpace<TTerm, tarr![Fn0<TTerm, tarr![OpI32Const<U0>, OpI32Load8U<U0>, OpReturn]>]>,
        WasmMemoryDecl<U1, U1, tarr![WasmDataSegment<InitI32Const<U0>, tarr![U9]>]>,
        TTerm,
        TTerm,
        tarr![WasmExport<typelude_wasm::tstr::TS!("main"), ExportFunc<U0>>],
        NoStart
    >;
    type Env = WasmHostEnv<
        TTerm,
        TTerm,
        tarr![HostMemoryBinding<
            typelude_wasm::tstr::TS!("host"),
            typelude_wasm::tstr::TS!("memory"),
            WasmMemory<U1, U1, TTerm>
        >],
        TTerm,
    >;
    type FinalState = InvokeFuncWithEnv<Module, Env, U0, TTerm>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U9>]);
    assert_type_eq_all!(
        <FinalState as StateMemory>::Output,
        WasmMemory<U1, U1, tarr![MemoryCell<U0, U9>]>
    );
}

type ReturnsSeven =
    WasmFunc<WasmFuncType<TTerm, tarr![WasmI32Type]>, TTerm, tarr![OpI32Const<U7>, OpReturn]>;

#[test]
fn imported_table_receives_active_elem_segments() {
    type Module = WasmModule<
        tarr![WasmImport<
            typelude_wasm::tstr::TS!("host"),
            typelude_wasm::tstr::TS!("table"),
            ImportTable<U1, U1>
        >],
        WasmFuncSpace<
            tarr![WasmFuncType<TTerm, tarr![WasmI32Type]>],
            tarr![
                ReturnsSeven,
                Fn0<TTerm, tarr![OpI32Const<U0>, OpCallIndirect<U0>, OpReturn]>
            ]
        >,
        WasmMemoryDecl<U0, NoLimit, TTerm>,
        tarr![WasmTableDecl<U1, U1, tarr![WasmElemSegment<U0, InitI32Const<U0>, tarr![U0]>]>],
        TTerm,
        tarr![WasmExport<typelude_wasm::tstr::TS!("main"), ExportFunc<U1>>],
        NoStart
    >;
    type Env = WasmHostEnv<
        TTerm,
        TTerm,
        TTerm,
        tarr![HostTableBinding<
            typelude_wasm::tstr::TS!("host"),
            typelude_wasm::tstr::TS!("table"),
            WasmTable<U1, U1, TTerm>
        >],
    >;
    type FinalState = InvokeFuncWithEnv<Module, Env, U1, TTerm>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U7>]);
    assert_type_eq_all!(
        <FinalState as StateTables>::Output,
        tarr![WasmTable<U1, U1, tarr![TableEntry<U0, U0>]>]
    );
}

type ExportStart = Fn0<
    TTerm,
    tarr![OpI32Const<U9>, OpGlobalSet<U0>, OpI32Const<U1>, OpI32Const<U8>, OpI32Store8<U0>, OpReturn],
>;
type ExportMain = Fn0<TTerm, tarr![OpGlobalGet<U0>, OpReturn]>;
type ExportedStateModule = WasmModule<
    TTerm,
    WasmFuncSpace<
        tarr![WasmFuncType<TTerm, TTerm>, WasmFuncType<TTerm, tarr![WasmI32Type]>],
        tarr![ExportStart, ExportMain]
    >,
    WasmMemoryDecl<U1, NoLimit, TTerm>,
    tarr![WasmTableDecl<U1, U1, tarr![WasmElemSegment<U0, InitI32Const<U0>, tarr![U1]>]>],
    tarr![WasmGlobalDecl<GlobalMut, InitI32Const<U2>>],
    tarr![
        WasmExport<typelude_wasm::tstr::TS!("main"), ExportFunc<U1>>,
        WasmExport<typelude_wasm::tstr::TS!("g"), ExportGlobal<U0>>,
        WasmExport<typelude_wasm::tstr::TS!("memory"), ExportMemory>,
        WasmExport<typelude_wasm::tstr::TS!("table"), ExportTable<U0>>
    ],
    StartFunc<U0>
>;

#[test]
fn export_accessors_read_live_state_entries_by_name() {
    type FinalState = typelude_wasm::InvokeExport<ExportedStateModule, typelude_wasm::tstr::TS!("main"), TTerm>;

    assert_type_eq_all!(<FinalState as StateStack>::Output, tarr![WasmI32<U9>]);
    assert_type_eq_all!(
        <FinalState as StateExportGlobal<typelude_wasm::tstr::TS!("g")>>::Output,
        WasmGlobal<GlobalMut, WasmI32<U9>>
    );
    assert_type_eq_all!(
        <FinalState as StateExportMemory<typelude_wasm::tstr::TS!("memory")>>::Output,
        WasmMemory<U1, MaxPages, tarr![MemoryCell<U1, U8>]>
    );
    assert_type_eq_all!(
        <FinalState as StateExportTable<typelude_wasm::tstr::TS!("table")>>::Output,
        WasmTable<U1, U1, tarr![TableEntry<U0, U1>]>
    );
}
