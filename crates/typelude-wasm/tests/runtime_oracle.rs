#![recursion_limit = "65536"]

mod support;

extern crate self as typelude;

pub mod wasm {
    pub use typelude_wasm::*;
}

use static_assertions::assert_type_eq_all;
use typelude_wasm::{
    GlobalConst, GlobalMut, HostCall, HostCallResult, HostFuncBinding, HostGlobalBinding,
    HostMemoryBinding, HostTableBinding, InvokeExport, InvokeExportWithEnv, StateExportGlobal,
    StateExportMemory, StateExportTable, StateStack, TArr, TTerm, WasmF32, WasmGlobal,
    WasmHostEnv, WasmI32, WasmI32Type, WasmI64, WasmMemory, WasmTable,
};
pub use typenum;
use typenum::{operator_aliases::Sum, Const, ToUInt, U0, U1, U2, U3, U5, U7, U8};

use crate::support::runtime::{
    run_wat_i64_result, run_wat_snapshot, run_wat_snapshot_with_env, RuntimeEnv,
    RuntimeGlobalImport, RuntimeMemoryImport, RuntimeSnapshot, RuntimeTableImport,
};

type NoArgs = TTerm;
type OneArg<A> = TArr<WasmI32<A>, TTerm>;
type TwoArgs<A, B> = TArr<WasmI32<A>, TArr<WasmI32<B>, TTerm>>;
type U9 = <Const<9> as ToUInt>::Output;
type U10 = <Const<10> as ToUInt>::Output;
type U20 = <Const<20> as ToUInt>::Output;
type U40 = <Const<40> as ToUInt>::Output;
type U42 = <Const<42> as ToUInt>::Output;
type U247 = <Const<247> as ToUInt>::Output;
type U255 = <Const<255> as ToUInt>::Output;
type U4660 = <Const<4660> as ToUInt>::Output;
type U1065353216 = <Const<1065353216usize> as ToUInt>::Output;
type U4607182418800017408 = <Const<4607182418800017408usize> as ToUInt>::Output;
type U18446744073709551614 = <Const<18446744073709551614usize> as ToUInt>::Output;

struct HostAdd;

impl<Store, A, B>
    HostCall<
        typelude_wasm::WasmFuncType<
            TArr<WasmI32Type, TArr<WasmI32Type, TTerm>>,
            TArr<WasmI32Type, TTerm>,
        >,
        Store,
        TArr<WasmI32<A>, TArr<WasmI32<B>, TTerm>>,
    > for HostAdd
where
    A: core::ops::Add<B>,
{
    type Output = HostCallResult<Store, TArr<WasmI32<Sum<A, B>>, TTerm>>;
}

const CORE_WAT: &str = r#"
    (module
      (func $add_internal (param i32 i32) (result i32)
        local.get 0
        local.get 1
        i32.add)
      (func $fib_impl (param i32) (result i32)
        local.get 0
        i32.eqz
        if
          i32.const 0
          return
        end
        local.get 0
        i32.const 1
        i32.sub
        i32.eqz
        if
          i32.const 1
          return
        end
        local.get 0
        i32.const 1
        i32.sub
        call $fib_impl
        local.get 0
        i32.const 2
        i32.sub
        call $fib_impl
        i32.add)
      (func (export "add") (param i32 i32) (result i32)
        local.get 0
        local.get 1
        i32.add)
      (func (export "internal_call") (param i32) (result i32)
        local.get 0
        i32.const 2
        call $add_internal)
      (func (export "loop_countdown") (param i32) (result i32)
        block
          loop
            local.get 0
            i32.eqz
            br_if 1
            local.get 0
            i32.const 1
            i32.sub
            local.set 0
            br 0
          end
        end
        local.get 0)
      (func (export "fib") (param i32) (result i32)
        local.get 0
        call $fib_impl))
"#;

type CoreModule = typelude_macros::twat! {
    module: r#"
        (module
          (func $add_internal (param i32 i32) (result i32)
            local.get 0
            local.get 1
            i32.add)
          (func $fib_impl (param i32) (result i32)
            local.get 0
            i32.eqz
            if
              i32.const 0
              return
            end
            local.get 0
            i32.const 1
            i32.sub
            i32.eqz
            if
              i32.const 1
              return
            end
            local.get 0
            i32.const 1
            i32.sub
            call $fib_impl
            local.get 0
            i32.const 2
            i32.sub
            call $fib_impl
            i32.add)
          (func (export "add") (param i32 i32) (result i32)
            local.get 0
            local.get 1
            i32.add)
          (func (export "internal_call") (param i32) (result i32)
            local.get 0
            i32.const 2
            call $add_internal)
          (func (export "loop_countdown") (param i32) (result i32)
            block
              loop
                local.get 0
                i32.eqz
                br_if 1
                local.get 0
                i32.const 1
                i32.sub
                local.set 0
                br 0
              end
            end
            local.get 0)
          (func (export "fib") (param i32) (result i32)
            local.get 0
            call $fib_impl))
    "#,
};

const STATE_WAT: &str = r#"
    (module
      (memory (export "memory") 1 2)
      (global $g (export "g") (mut i32) (i32.const 2))
      (global $started (mut i32) (i32.const 0))
      (data (i32.const 16) "\2a")
      (func $start
        i32.const 7
        global.set $started
        i32.const 1
        i32.const 8
        i32.store8)
      (start $start)
      (func (export "memory_byte") (result i32)
        i32.const 0
        i32.const 255
        i32.store8
        i32.const 0
        i32.load8_u)
      (func (export "offset_grow") (result i32)
        i32.const 0
        i32.const 258
        i32.store offset=4
        i32.const 1
        memory.grow
        drop
        i32.const 5
        i32.load8_u)
      (func (export "data_read") (result i32)
        i32.const 16
        i32.load8_u)
      (func (export "global_update") (result i32)
        global.get $g
        i32.const 3
        i32.add
        global.set $g
        global.get $g)
      (func (export "started") (result i32)
        global.get $started))
"#;

type StateModule = typelude_macros::twat! {
    module: r#"
        (module
          (memory (export "memory") 1 2)
          (global $g (export "g") (mut i32) (i32.const 2))
          (global $started (mut i32) (i32.const 0))
          (data (i32.const 16) "\2a")
          (func $start
            i32.const 7
            global.set $started
            i32.const 1
            i32.const 8
            i32.store8)
          (start $start)
          (func (export "memory_byte") (result i32)
            i32.const 0
            i32.const 255
            i32.store8
            i32.const 0
            i32.load8_u)
          (func (export "offset_grow") (result i32)
            i32.const 0
            i32.const 258
            i32.store offset=4
            i32.const 1
            memory.grow
            drop
            i32.const 5
            i32.load8_u)
          (func (export "data_read") (result i32)
            i32.const 16
            i32.load8_u)
          (func (export "global_update") (result i32)
            global.get $g
            i32.const 3
            i32.add
            global.set $g
            global.get $g)
          (func (export "started") (result i32)
            global.get $started))
    "#,
};

const DISPATCH_WAT: &str = r#"
    (module
      (type $ret (func (result i32)))
      (table (export "table") 2 2 funcref)
      (table $t1 1 1 funcref)
      (elem (i32.const 0) $one $two)
      (elem (table $t1) (i32.const 0) func $three)
      (func $one (type $ret) (result i32)
        i32.const 1)
      (func $two (type $ret) (result i32)
        i32.const 2)
      (func $three (type $ret) (result i32)
        i32.const 3)
      (func (export "default_indirect") (result i32)
        i32.const 1
        call_indirect (type $ret))
      (func (export "explicit_indirect") (result i32)
        i32.const 0
        call_indirect $t1 (type $ret))
      (func (export "br_table") (param i32) (result i32)
        block
          block
            block
              block
                local.get 0
                br_table 0 1 2 3
              end
              i32.const 10
              return
            end
            i32.const 20
            return
          end
          i32.const 30
          return
        end
        i32.const 40))
"#;

type DispatchModule = typelude_macros::twat! {
    module: r#"
        (module
          (type $ret (func (result i32)))
          (table (export "table") 2 2 funcref)
          (table $t1 1 1 funcref)
          (elem (i32.const 0) $one $two)
          (elem (table $t1) (i32.const 0) func $three)
          (func $one (type $ret) (result i32)
            i32.const 1)
          (func $two (type $ret) (result i32)
            i32.const 2)
          (func $three (type $ret) (result i32)
            i32.const 3)
          (func (export "default_indirect") (result i32)
            i32.const 1
            call_indirect (type $ret))
          (func (export "explicit_indirect") (result i32)
            i32.const 0
            call_indirect $t1 (type $ret))
          (func (export "br_table") (param i32) (result i32)
            block
              block
                block
                  block
                    local.get 0
                    br_table 0 1 2 3
                  end
                  i32.const 10
                  return
                end
                i32.const 20
                return
              end
              i32.const 30
              return
            end
            i32.const 40))
    "#,
};

const RUNTIME_PARITY_WAT: &str = r#"
    (module
      (memory 1)
      (func (export "i32_parity") (result i32)
        i32.const 240
        i32.const 15
        i32.or
        i32.const 8
        i32.xor
        i32.const 1
        i32.shl
        i32.const 2
        i32.div_u)
      (func (export "wrap_i64") (result i32)
        i64.const 4294967298
        i32.wrap_i64)
      (func (export "partial_width_i32") (result i32)
        i32.const 1
        i32.const 4660
        i32.store16
        i32.const 1
        i32.load16_u)
      (func (export "partial_width_i64") (result i64)
        i32.const 0
        i64.const -2
        i64.store32
        i32.const 0
        i64.load32_s)
      (func (export "reinterpret_i64") (result i64)
        i64.const 4607182418800017408
        f64.reinterpret_i64
        i64.reinterpret_f64)
      (func (export "reinterpret_f32") (result f32)
        i32.const 1065353216
        f32.reinterpret_i32))
"#;

type RuntimeParityModule = typelude_macros::twat! {
    module: r#"
        (module
          (memory 1)
          (func (export "i32_parity") (result i32)
            i32.const 240
            i32.const 15
            i32.or
            i32.const 8
            i32.xor
            i32.const 1
            i32.shl
            i32.const 2
            i32.div_u)
          (func (export "wrap_i64") (result i32)
            i64.const 4294967298
            i32.wrap_i64)
          (func (export "partial_width_i32") (result i32)
            i32.const 1
            i32.const 4660
            i32.store16
            i32.const 1
            i32.load16_u)
          (func (export "partial_width_i64") (result i64)
            i32.const 0
            i64.const -2
            i64.store32
            i32.const 0
            i64.load32_s)
          (func (export "reinterpret_i64") (result i64)
            i64.const 4607182418800017408
            f64.reinterpret_i64
            i64.reinterpret_f64)
          (func (export "reinterpret_f32") (result f32)
            i32.const 1065353216
            f32.reinterpret_i32))
    "#,
};

const IMPORT_FUNCS_GLOBALS_WAT: &str = r#"
    (module
      (import "host" "add" (func $add (param i32 i32) (result i32)))
      (import "host" "g" (global $g (mut i32)))
      (global $copy (export "copy") i32 (global.get $g))
      (func (export "imported_add") (param i32 i32) (result i32)
        local.get 0
        local.get 1
        call $add)
      (func (export "imported_global") (result i32)
        global.get $g
        i32.const 2
        i32.add
        global.set $g
        global.get $g))
"#;

type ImportFuncsGlobalsModule = typelude_macros::twat! {
    module: r#"
        (module
          (import "host" "add" (func $add (param i32 i32) (result i32)))
          (import "host" "g" (global $g (mut i32)))
          (global $copy (export "copy") i32 (global.get $g))
          (func (export "imported_add") (param i32 i32) (result i32)
            local.get 0
            local.get 1
            call $add)
          (func (export "imported_global") (result i32)
            global.get $g
            i32.const 2
            i32.add
            global.set $g
            global.get $g))
    "#,
};

const IMPORT_MEMORY_TABLE_WAT: &str = r#"
    (module
      (type $ret (func (result i32)))
      (import "host" "memory" (memory $memory 1))
      (import "host" "table" (table $table 1 funcref))
      (export "memory" (memory $memory))
      (export "table_export" (table $table))
      (data (i32.const 0) "\2a")
      (elem (i32.const 0) func $seven)
      (func $seven (type $ret) (result i32)
        i32.const 7)
      (func (export "imported_memory") (result i32)
        i32.const 0
        i32.load8_u)
      (func (export "imported_table") (result i32)
        i32.const 0
        call_indirect (type $ret)))
"#;

type ImportMemoryTableModule = typelude_macros::twat! {
    module: r#"
        (module
          (type $ret (func (result i32)))
          (import "host" "memory" (memory $memory 1))
          (import "host" "table" (table $table 1 funcref))
          (export "memory" (memory $memory))
          (export "table_export" (table $table))
          (data (i32.const 0) "\2a")
          (elem (i32.const 0) func $seven)
          (func $seven (type $ret) (result i32)
            i32.const 7)
          (func (export "imported_memory") (result i32)
            i32.const 0
            i32.load8_u)
          (func (export "imported_table") (result i32)
            i32.const 0
            call_indirect (type $ret)))
    "#,
};

type FunctionEnv = WasmHostEnv<
    TArr<
        HostFuncBinding<
            typelude_wasm::typelude_str::tstr!("host"),
            typelude_wasm::typelude_str::tstr!("add"),
            HostAdd,
        >,
        TTerm,
    >,
    TTerm,
    TTerm,
    TTerm,
>;

type GlobalEnv = WasmHostEnv<
    TTerm,
    TArr<
        HostGlobalBinding<
            typelude_wasm::typelude_str::tstr!("host"),
            typelude_wasm::typelude_str::tstr!("g"),
            WasmGlobal<GlobalMut, WasmI32<U3>>,
        >,
        TTerm,
    >,
    TTerm,
    TTerm,
>;

type MemoryEnv = WasmHostEnv<
    TTerm,
    TTerm,
    TArr<
        HostMemoryBinding<
            typelude_wasm::typelude_str::tstr!("host"),
            typelude_wasm::typelude_str::tstr!("memory"),
            WasmMemory<U1, U1, TTerm>,
        >,
        TTerm,
    >,
    TTerm,
>;

type TableEnv = WasmHostEnv<
    TTerm,
    TTerm,
    TTerm,
    TArr<
        HostTableBinding<
            typelude_wasm::typelude_str::tstr!("host"),
            typelude_wasm::typelude_str::tstr!("table"),
            WasmTable<U1, U1, TTerm>,
        >,
        TTerm,
    >,
>;

#[test]
fn wasmi_oracle_matches_core_execution_paths() {
    let add = run_wat_snapshot(CORE_WAT, "add", &[2, 3]).expect("wasmi should execute add");
    assert_eq!(add.result_i32, 5);

    let internal =
        run_wat_snapshot(CORE_WAT, "internal_call", &[3]).expect("wasmi should execute internal call");
    assert_eq!(internal.result_i32, 5);

    let countdown =
        run_wat_snapshot(CORE_WAT, "loop_countdown", &[3]).expect("wasmi should execute countdown");
    assert_eq!(countdown.result_i32, 0);

    let fib = run_wat_snapshot(CORE_WAT, "fib", &[6]).expect("wasmi should execute fibonacci");
    assert_eq!(fib.result_i32, 8);

    type Add = InvokeExport<CoreModule, typelude_wasm::typelude_str::tstr!("add"), TwoArgs<U2, U3>>;
    type Internal = InvokeExport<
        CoreModule,
        typelude_wasm::typelude_str::tstr!("internal_call"),
        OneArg<U3>,
    >;
    type Countdown = InvokeExport<
        CoreModule,
        typelude_wasm::typelude_str::tstr!("loop_countdown"),
        OneArg<U3>,
    >;
    type Fib = InvokeExport<
        CoreModule,
        typelude_wasm::typelude_str::tstr!("fib"),
        OneArg<<Const<6> as ToUInt>::Output>,
    >;

    assert_type_eq_all!(<Add as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
    assert_type_eq_all!(<Internal as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
    assert_type_eq_all!(<Countdown as StateStack>::Output, TArr<WasmI32<U0>, TTerm>);
    assert_type_eq_all!(<Fib as StateStack>::Output, TArr<WasmI32<U8>, TTerm>);
}

#[test]
fn wasmi_oracle_matches_memory_globals_and_start() {
    let memory = run_wat_snapshot_with_env(
        STATE_WAT,
        "memory_byte",
        &[],
        &RuntimeEnv {
            observed_memory_export: Some("memory"),
            observed_global_export: Some("g"),
            ..RuntimeEnv::default()
        },
    )
    .expect("wasmi should execute memory program");
    assert_eq!(
        memory,
        RuntimeSnapshot {
            result_i32: 255,
            memory_pages: Some(1),
            memory_prefix: vec![255, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            exported_global_i32: Some(2),
        }
    );

    let offset_grow = run_wat_snapshot_with_env(
        STATE_WAT,
        "offset_grow",
        &[],
        &RuntimeEnv {
            observed_memory_export: Some("memory"),
            ..RuntimeEnv::default()
        },
    )
    .expect("wasmi should execute offset/grow");
    assert_eq!(offset_grow.result_i32, 1);
    assert_eq!(offset_grow.memory_pages, Some(2));

    let data = run_wat_snapshot(STATE_WAT, "data_read", &[]).expect("wasmi should execute data");
    assert_eq!(data.result_i32, 42);

    let globals =
        run_wat_snapshot(STATE_WAT, "global_update", &[]).expect("wasmi should execute globals");
    assert_eq!(globals.result_i32, 5);

    let started = run_wat_snapshot(STATE_WAT, "started", &[]).expect("wasmi should execute start");
    assert_eq!(started.result_i32, 7);

    type Memory = InvokeExport<StateModule, typelude_wasm::typelude_str::tstr!("memory_byte"), NoArgs>;
    type OffsetGrow =
        InvokeExport<StateModule, typelude_wasm::typelude_str::tstr!("offset_grow"), NoArgs>;
    type Data = InvokeExport<StateModule, typelude_wasm::typelude_str::tstr!("data_read"), NoArgs>;
    type GlobalUpdate =
        InvokeExport<StateModule, typelude_wasm::typelude_str::tstr!("global_update"), NoArgs>;
    type Started = InvokeExport<StateModule, typelude_wasm::typelude_str::tstr!("started"), NoArgs>;

    assert_type_eq_all!(<Memory as StateStack>::Output, TArr<WasmI32<U255>, TTerm>);
    assert_type_eq_all!(<OffsetGrow as StateStack>::Output, TArr<WasmI32<U1>, TTerm>);
    assert_type_eq_all!(<Data as StateStack>::Output, TArr<WasmI32<U42>, TTerm>);
    assert_type_eq_all!(<GlobalUpdate as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
    assert_type_eq_all!(<Started as StateStack>::Output, TArr<WasmI32<U7>, TTerm>);
    assert_type_eq_all!(
        <GlobalUpdate as StateExportGlobal<typelude_wasm::typelude_str::tstr!("g")>>::Output,
        WasmGlobal<GlobalMut, WasmI32<U5>>
    );
    type _MemoryExport =
        <OffsetGrow as StateExportMemory<typelude_wasm::typelude_str::tstr!("memory")>>::Output;
}

#[test]
fn wasmi_oracle_matches_imports_and_exported_state() {
    let imported_function = run_wat_snapshot_with_env(
        IMPORT_FUNCS_GLOBALS_WAT,
        "imported_add",
        &[2, 3],
        &RuntimeEnv {
            add_func: Some(("host", "add")),
            ..RuntimeEnv::default()
        },
    )
    .expect("wasmi should execute imported function");
    assert_eq!(imported_function.result_i32, 5);

    let imported_global = run_wat_snapshot_with_env(
        IMPORT_FUNCS_GLOBALS_WAT,
        "imported_global",
        &[],
        &RuntimeEnv {
            global: Some(RuntimeGlobalImport {
                module: "host",
                field: "g",
                value: 3,
                mutable: true,
            }),
            observed_global_export: Some("copy"),
            ..RuntimeEnv::default()
        },
    )
    .expect("wasmi should execute imported global");
    assert_eq!(imported_global.result_i32, 5);
    assert_eq!(imported_global.exported_global_i32, Some(3));

    let imported_memory = run_wat_snapshot_with_env(
        IMPORT_MEMORY_TABLE_WAT,
        "imported_memory",
        &[],
        &RuntimeEnv {
            memory: Some(RuntimeMemoryImport {
                module: "host",
                field: "memory",
                min: 1,
                max: Some(1),
            }),
            observed_memory_export: Some("memory"),
            ..RuntimeEnv::default()
        },
    )
    .expect("wasmi should execute imported memory");
    assert_eq!(imported_memory.result_i32, 42);

    let imported_table = run_wat_snapshot_with_env(
        IMPORT_MEMORY_TABLE_WAT,
        "imported_table",
        &[],
        &RuntimeEnv {
            table: Some(RuntimeTableImport {
                module: "host",
                field: "table",
                min: 1,
                max: Some(1),
            }),
            ..RuntimeEnv::default()
        },
    )
    .expect("wasmi should execute imported table");
    assert_eq!(imported_table.result_i32, 7);

    type ImportedAdd = InvokeExportWithEnv<
        ImportFuncsGlobalsModule,
        FunctionEnv,
        typelude_wasm::typelude_str::tstr!("imported_add"),
        TwoArgs<U2, U3>,
    >;
    type ImportedGlobal = InvokeExportWithEnv<
        ImportFuncsGlobalsModule,
        GlobalEnv,
        typelude_wasm::typelude_str::tstr!("imported_global"),
        NoArgs,
    >;
    type ImportedMemory = InvokeExportWithEnv<
        ImportMemoryTableModule,
        MemoryEnv,
        typelude_wasm::typelude_str::tstr!("imported_memory"),
        NoArgs,
    >;
    type ImportedTable = InvokeExportWithEnv<
        ImportMemoryTableModule,
        TableEnv,
        typelude_wasm::typelude_str::tstr!("imported_table"),
        NoArgs,
    >;

    assert_type_eq_all!(<ImportedAdd as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
    assert_type_eq_all!(<ImportedGlobal as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
    assert_type_eq_all!(<ImportedMemory as StateStack>::Output, TArr<WasmI32<U42>, TTerm>);
    assert_type_eq_all!(<ImportedTable as StateStack>::Output, TArr<WasmI32<U7>, TTerm>);
    assert_type_eq_all!(
        <ImportedGlobal as StateExportGlobal<typelude_wasm::typelude_str::tstr!("copy")>>::Output,
        WasmGlobal<GlobalConst, WasmI32<U3>>
    );
    type _ImportedMemoryExport =
        <ImportedMemory as StateExportMemory<typelude_wasm::typelude_str::tstr!("memory")>>::Output;
    type _ImportedTableExport =
        <ImportedTable as StateExportTable<typelude_wasm::typelude_str::tstr!("table_export")>>::Output;
}

#[test]
fn wasmi_oracle_matches_dispatch_and_runtime_parity_ops() {
    let default_indirect = run_wat_snapshot(DISPATCH_WAT, "default_indirect", &[])
        .expect("wasmi should dispatch default table");
    assert_eq!(default_indirect.result_i32, 2);

    let explicit_indirect = run_wat_snapshot(DISPATCH_WAT, "explicit_indirect", &[])
        .expect("wasmi should dispatch explicit table");
    assert_eq!(explicit_indirect.result_i32, 3);

    assert_eq!(
        run_wat_snapshot(DISPATCH_WAT, "br_table", &[0]).expect("br_table 0").result_i32,
        10
    );
    assert_eq!(
        run_wat_snapshot(DISPATCH_WAT, "br_table", &[1]).expect("br_table 1").result_i32,
        20
    );
    assert_eq!(
        run_wat_snapshot(DISPATCH_WAT, "br_table", &[9]).expect("br_table default").result_i32,
        40
    );

    let i32_parity = run_wat_snapshot(RUNTIME_PARITY_WAT, "i32_parity", &[])
        .expect("wasmi should execute i32 parity ops");
    assert_eq!(i32_parity.result_i32, 247);

    let wrap =
        run_wat_snapshot(RUNTIME_PARITY_WAT, "wrap_i64", &[]).expect("wasmi should execute wrap");
    assert_eq!(wrap.result_i32, 2);

    let partial_i32 = run_wat_snapshot(RUNTIME_PARITY_WAT, "partial_width_i32", &[])
        .expect("wasmi should execute i32 partial width");
    assert_eq!(partial_i32.result_i32, 4660);

    let partial_i64 = run_wat_i64_result(RUNTIME_PARITY_WAT, "partial_width_i64", &[])
        .expect("wasmi should execute i64 partial width");
    assert_eq!(partial_i64, -2);

    let reinterpret_i64 = run_wat_i64_result(RUNTIME_PARITY_WAT, "reinterpret_i64", &[])
        .expect("wasmi should execute reinterpret");
    assert_eq!(reinterpret_i64, 4607182418800017408i64);

    type DefaultIndirect =
        InvokeExport<DispatchModule, typelude_wasm::typelude_str::tstr!("default_indirect"), NoArgs>;
    type ExplicitIndirect =
        InvokeExport<DispatchModule, typelude_wasm::typelude_str::tstr!("explicit_indirect"), NoArgs>;
    type BrTable0 =
        InvokeExport<DispatchModule, typelude_wasm::typelude_str::tstr!("br_table"), OneArg<U0>>;
    type BrTable1 =
        InvokeExport<DispatchModule, typelude_wasm::typelude_str::tstr!("br_table"), OneArg<U1>>;
    type BrTableDefault =
        InvokeExport<DispatchModule, typelude_wasm::typelude_str::tstr!("br_table"), OneArg<U9>>;
    type I32Parity =
        InvokeExport<RuntimeParityModule, typelude_wasm::typelude_str::tstr!("i32_parity"), NoArgs>;
    type WrapI64 =
        InvokeExport<RuntimeParityModule, typelude_wasm::typelude_str::tstr!("wrap_i64"), NoArgs>;
    type PartialWidthI32 =
        InvokeExport<RuntimeParityModule, typelude_wasm::typelude_str::tstr!("partial_width_i32"), NoArgs>;
    type PartialWidthI64 =
        InvokeExport<RuntimeParityModule, typelude_wasm::typelude_str::tstr!("partial_width_i64"), NoArgs>;
    type ReinterpretI64 =
        InvokeExport<RuntimeParityModule, typelude_wasm::typelude_str::tstr!("reinterpret_i64"), NoArgs>;
    type ReinterpretF32 =
        InvokeExport<RuntimeParityModule, typelude_wasm::typelude_str::tstr!("reinterpret_f32"), NoArgs>;

    assert_type_eq_all!(<DefaultIndirect as StateStack>::Output, TArr<WasmI32<U2>, TTerm>);
    assert_type_eq_all!(<ExplicitIndirect as StateStack>::Output, TArr<WasmI32<U3>, TTerm>);
    assert_type_eq_all!(<BrTable0 as StateStack>::Output, TArr<WasmI32<U10>, TTerm>);
    assert_type_eq_all!(<BrTable1 as StateStack>::Output, TArr<WasmI32<U20>, TTerm>);
    assert_type_eq_all!(<BrTableDefault as StateStack>::Output, TArr<WasmI32<U40>, TTerm>);
    assert_type_eq_all!(<I32Parity as StateStack>::Output, TArr<WasmI32<U247>, TTerm>);
    assert_type_eq_all!(<WrapI64 as StateStack>::Output, TArr<WasmI32<U2>, TTerm>);
    assert_type_eq_all!(<PartialWidthI32 as StateStack>::Output, TArr<WasmI32<U4660>, TTerm>);
    assert_type_eq_all!(<PartialWidthI64 as StateStack>::Output, TArr<WasmI64<U18446744073709551614>, TTerm>);
    assert_type_eq_all!(<ReinterpretI64 as StateStack>::Output, TArr<WasmI64<U4607182418800017408>, TTerm>);
    assert_type_eq_all!(<ReinterpretF32 as StateStack>::Output, TArr<WasmF32<U1065353216>, TTerm>);
    type _TableExport =
        <DefaultIndirect as StateExportTable<typelude_wasm::typelude_str::tstr!("table")>>::Output;
}
