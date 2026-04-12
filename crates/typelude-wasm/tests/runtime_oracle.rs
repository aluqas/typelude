#![recursion_limit = "65536"]

mod support;

extern crate self as typelude;

pub mod wasm {
    pub use typelude_wasm::*;
}
use static_assertions::assert_type_eq_all;
use typelude_wasm::{
    GlobalConst, GlobalMut, HostCall, HostCallResult, HostFuncBinding, HostGlobalBinding,
    HostMemoryBinding, HostTableBinding, InvokeExport, InvokeExportWithEnv, InvokeFunc,
    StateExportGlobal, StateStack, TArr, TTerm, WasmGlobal, WasmHostEnv, WasmI32, WasmI32Type,
    WasmMemory, WasmTable,
};
pub use typenum;
use typenum::{Const, ToUInt, U0, U1, U2, U3, U5, U7, U8, U9, U42, operator_aliases::Sum};

use crate::support::runtime::{
    RuntimeEnv, RuntimeGlobalImport, RuntimeMemoryImport, RuntimeSnapshot, RuntimeTableImport,
    run_wat_snapshot, run_wat_snapshot_with_env,
};

type NoArgs = TTerm;
type OneArg<A> = TArr<WasmI32<A>, TTerm>;
type TwoArgs<A, B> = TArr<WasmI32<A>, TArr<WasmI32<B>, TTerm>>;

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

#[test]
fn wasmi_oracle_matches_type_level_add() {
    const MODULE: &str = r#"
        (module
          (func (export "main") (param i32 i32) (result i32)
            local.get 0
            local.get 1
            i32.add))
    "#;

    let runtime = run_wat_snapshot(MODULE, "main", &[2, 3]).expect("wasmi should execute add");
    assert_eq!(runtime, RuntimeSnapshot {
        result_i32: 5,
        memory_pages: None,
        memory_prefix: Vec::new(),
        exported_global_i32: None,
    });

    type Module = typelude_macros::twat! {
        module: r#"
            (module
              (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add))
        "#,
    };
    type Final = InvokeFunc<Module, U0, TwoArgs<U2, U3>>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
}

#[test]
fn wasmi_oracle_matches_type_level_countdown_loop() {
    const MODULE: &str = r#"
        (module
          (func (export "main") (param i32) (result i32)
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
            local.get 0))
    "#;

    let runtime = run_wat_snapshot(MODULE, "main", &[3]).expect("wasmi should execute loop");
    assert_eq!(runtime.result_i32, 0);

    type Module = typelude_macros::twat! {
        module: r#"
            (module
              (func (export "main") (param i32) (result i32)
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
                local.get 0))
        "#,
    };
    type Final = InvokeFunc<Module, U0, OneArg<U3>>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U0>, TTerm>);
}

#[test]
fn wasmi_oracle_matches_type_level_memory_store8_and_size() {
    const MODULE: &str = r#"
        (module
          (memory (export "memory") 1)
          (func (export "main") (result i32)
            i32.const 0
            i32.const 255
            i32.store8
            i32.const 0
            i32.load8_u))
    "#;

    let runtime =
        run_wat_snapshot(MODULE, "main", &[]).expect("wasmi should execute memory program");
    assert_eq!(runtime.result_i32, 255);
    assert_eq!(runtime.memory_pages, Some(1));
    assert_eq!(runtime.memory_prefix.first().copied(), Some(255));

    type Module = typelude_macros::twat! {
        module: r#"
            (module
              (memory (export "memory") 1)
              (func (export "main") (result i32)
                i32.const 0
                i32.const 255
                i32.store8
                i32.const 0
                i32.load8_u))
        "#,
    };
    type Final = InvokeFunc<Module, U0, NoArgs>;

    assert_type_eq_all!(
        <Final as StateStack>::Output,
        TArr<WasmI32<<Const<255> as ToUInt>::Output>, TTerm>
    );
}

#[test]
fn wasmi_oracle_matches_type_level_iterative_fibonacci() {
    const MODULE: &str = r#"
        (module
          (func (export "main") (param i32) (result i32)
            (local i32 i32 i32)
            i32.const 0
            local.set 1
            i32.const 1
            local.set 2
            block
              loop
                local.get 0
                i32.eqz
                br_if 1
                local.get 2
                local.set 3
                local.get 1
                local.get 2
                i32.add
                local.set 2
                local.get 3
                local.set 1
                local.get 0
                i32.const 1
                i32.sub
                local.set 0
                br 0
              end
            end
            local.get 1))
    "#;

    let runtime = run_wat_snapshot(MODULE, "main", &[6]).expect("wasmi should execute fibonacci");
    assert_eq!(runtime.result_i32, 8);

    type Module = typelude_macros::twat! {
        module: r#"
            (module
              (func (export "main") (param i32) (result i32)
                (local i32 i32 i32)
                i32.const 0
                local.set 1
                i32.const 1
                local.set 2
                block
                  loop
                    local.get 0
                    i32.eqz
                    br_if 1
                    local.get 2
                    local.set 3
                    local.get 1
                    local.get 2
                    i32.add
                    local.set 2
                    local.get 3
                    local.set 1
                    local.get 0
                    i32.const 1
                    i32.sub
                    local.set 0
                    br 0
                  end
                end
                local.get 1))
        "#,
    };
    type Final = InvokeFunc<Module, U0, OneArg<<Const<6> as ToUInt>::Output>>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U8>, TTerm>);
}

#[test]
fn wasmi_oracle_matches_type_level_recursive_fibonacci() {
    const MODULE: &str = r#"
        (module
          (func (export "main") (param i32) (result i32)
            local.get 0
            call $fib)
          (func $fib (param i32) (result i32)
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
            call $fib
            local.get 0
            i32.const 2
            i32.sub
            call $fib
            i32.add))
    "#;

    let runtime =
        run_wat_snapshot(MODULE, "main", &[6]).expect("wasmi should execute recursive fibonacci");
    assert_eq!(runtime.result_i32, 8);

    type Module = typelude_macros::twat! {
        module: r#"
            (module
              (func (export "main") (param i32) (result i32)
                local.get 0
                call $fib)
              (func $fib (param i32) (result i32)
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
                call $fib
                local.get 0
                i32.const 2
                i32.sub
                call $fib
                i32.add))
        "#,
    };
    type Final = InvokeFunc<Module, U0, OneArg<<Const<6> as ToUInt>::Output>>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U8>, TTerm>);
}

#[test]
fn wasmi_oracle_matches_type_level_globals() {
    const MODULE: &str = r#"
        (module
          (global $g (mut i32) (i32.const 2))
          (func (export "main") (result i32)
            global.get $g
            i32.const 3
            i32.add
            global.set $g
            global.get $g))
    "#;

    let runtime = run_wat_snapshot(MODULE, "main", &[]).expect("wasmi should execute globals");
    assert_eq!(runtime.result_i32, 5);

    type Module = typelude_macros::twat! {
        module: r#"
            (module
              (global $g (mut i32) (i32.const 2))
              (func (export "main") (result i32)
                global.get $g
                i32.const 3
                i32.add
                global.set $g
                global.get $g))
        "#,
    };
    type Final = InvokeFunc<Module, U0, NoArgs>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
}

#[test]
fn wasmi_oracle_matches_type_level_offsets_and_memory_grow() {
    const MODULE: &str = r#"
        (module
          (memory (export "memory") 1 2)
          (func (export "main") (result i32)
            i32.const 0
            i32.const 258
            i32.store offset=4
            i32.const 1
            memory.grow
            drop
            i32.const 5
            i32.load8_u))
    "#;

    let runtime = run_wat_snapshot(MODULE, "main", &[]).expect("wasmi should execute offset/grow");
    assert_eq!(runtime.result_i32, 1);
    assert_eq!(runtime.memory_pages, Some(2));

    type Module = typelude_macros::twat! {
        module: r#"
            (module
              (memory (export "memory") 1 2)
              (func (export "main") (result i32)
                i32.const 0
                i32.const 258
                i32.store offset=4
                i32.const 1
                memory.grow
                drop
                i32.const 5
                i32.load8_u))
        "#,
    };
    type Final = InvokeFunc<Module, U0, NoArgs>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U1>, TTerm>);
}

#[test]
fn wasmi_oracle_matches_type_level_imported_function_call() {
    const MODULE: &str = r#"
        (module
          (import "host" "add" (func $add (param i32 i32) (result i32)))
          (func (export "main") (param i32 i32) (result i32)
            local.get 0
            local.get 1
            call $add))
    "#;

    let runtime = run_wat_snapshot_with_env(MODULE, "main", &[2, 3], &RuntimeEnv {
        add_func: Some(("host", "add")),
        ..RuntimeEnv::default()
    })
    .expect("wasmi should execute imported function call");
    assert_eq!(runtime.result_i32, 5);

    type Module = typelude_macros::twat! {
        module: r#"
            (module
              (import "host" "add" (func $add (param i32 i32) (result i32)))
              (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                call $add))
        "#,
    };
    type Env = WasmHostEnv<
        TArr<
            HostFuncBinding<
                typelude_wasm::tstr::TS!("host"),
                typelude_wasm::tstr::TS!("add"),
                HostAdd,
            >,
            TTerm,
        >,
        TTerm,
        TTerm,
        TTerm,
    >;
    type Final =
        InvokeExportWithEnv<Module, Env, typelude_wasm::tstr::TS!("main"), TwoArgs<U2, U3>>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
}

#[test]
fn wasmi_oracle_matches_type_level_imported_global_and_exported_global() {
    const MODULE: &str = r#"
        (module
          (import "host" "g" (global (mut i32)))
          (global $copy (export "copy") i32 (global.get 0))
          (func (export "main") (result i32)
            global.get 0
            i32.const 2
            i32.add
            global.set 0
            global.get 0))
    "#;

    let runtime = run_wat_snapshot_with_env(MODULE, "main", &[], &RuntimeEnv {
        global: Some(RuntimeGlobalImport {
            module: "host",
            field: "g",
            value: 3,
            mutable: true,
        }),
        observed_global_export: Some("copy"),
        ..RuntimeEnv::default()
    })
    .expect("wasmi should execute imported global program");
    assert_eq!(runtime.result_i32, 5);
    assert_eq!(runtime.exported_global_i32, Some(3));

    type Module = typelude_macros::twat! {
        module: r#"
            (module
              (import "host" "g" (global (mut i32)))
              (global $copy (export "copy") i32 (global.get 0))
              (func (export "main") (result i32)
                global.get 0
                i32.const 2
                i32.add
                global.set 0
                global.get 0))
        "#,
    };
    type Env = WasmHostEnv<
        TTerm,
        TArr<
            HostGlobalBinding<
                typelude_wasm::tstr::TS!("host"),
                typelude_wasm::tstr::TS!("g"),
                WasmGlobal<GlobalMut, WasmI32<U3>>,
            >,
            TTerm,
        >,
        TTerm,
        TTerm,
    >;
    type Final = InvokeExportWithEnv<Module, Env, typelude_wasm::tstr::TS!("main"), NoArgs>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
    assert_type_eq_all!(
        <Final as StateExportGlobal<typelude_wasm::tstr::TS!("copy")>>::Output,
        WasmGlobal<GlobalConst, WasmI32<U3>>
    );
}

#[test]
fn wasmi_oracle_matches_type_level_imported_memory() {
    const MODULE: &str = r#"
        (module
          (import "host" "memory" (memory 1))
          (data (i32.const 0) "\2a")
          (func (export "main") (result i32)
            i32.const 0
            i32.load8_u))
    "#;

    let runtime = run_wat_snapshot_with_env(MODULE, "main", &[], &RuntimeEnv {
        memory: Some(RuntimeMemoryImport {
            module: "host",
            field: "memory",
            min: 1,
            max: Some(1),
        }),
        observed_memory_export: Some("memory"),
        ..RuntimeEnv::default()
    })
    .expect("wasmi should execute imported memory program");
    assert_eq!(runtime.result_i32, 42);
    assert_eq!(runtime.memory_pages, None);

    type Module = typelude_macros::twat! {
        module: r#"
            (module
              (import "host" "memory" (memory 1))
              (data (i32.const 0) "\2a")
              (func (export "main") (result i32)
                i32.const 0
                i32.load8_u))
        "#,
    };
    type Env = WasmHostEnv<
        TTerm,
        TTerm,
        TArr<
            HostMemoryBinding<
                typelude_wasm::tstr::TS!("host"),
                typelude_wasm::tstr::TS!("memory"),
                WasmMemory<U1, U1, TTerm>,
            >,
            TTerm,
        >,
        TTerm,
    >;
    type Final = InvokeExportWithEnv<Module, Env, typelude_wasm::tstr::TS!("main"), NoArgs>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U42>, TTerm>);
}

#[test]
fn wasmi_oracle_matches_type_level_imported_table_call_indirect() {
    const MODULE: &str = r#"
        (module
          (type $ret (func (result i32)))
          (import "host" "table" (table 1 funcref))
          (elem (i32.const 0) func $seven)
          (func $seven (type $ret) (result i32)
            i32.const 7)
          (func (export "main") (result i32)
            i32.const 0
            call_indirect (type $ret)))
    "#;

    let runtime = run_wat_snapshot_with_env(MODULE, "main", &[], &RuntimeEnv {
        table: Some(RuntimeTableImport {
            module: "host",
            field: "table",
            min: 1,
            max: Some(1),
        }),
        ..RuntimeEnv::default()
    })
    .expect("wasmi should execute imported table call_indirect");
    assert_eq!(runtime.result_i32, 7);

    type Module = typelude_macros::twat! {
        module: r#"
            (module
              (type $ret (func (result i32)))
              (import "host" "table" (table 1 funcref))
              (elem (i32.const 0) func $seven)
              (func $seven (type $ret) (result i32)
                i32.const 7)
              (func (export "main") (result i32)
                i32.const 0
                call_indirect (type $ret)))
        "#,
    };
    type Env = WasmHostEnv<
        TTerm,
        TTerm,
        TTerm,
        TArr<
            HostTableBinding<
                typelude_wasm::tstr::TS!("host"),
                typelude_wasm::tstr::TS!("table"),
                WasmTable<U1, U1, TTerm>,
            >,
            TTerm,
        >,
    >;
    type Final = InvokeExportWithEnv<Module, Env, typelude_wasm::tstr::TS!("main"), NoArgs>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U7>, TTerm>);
}

#[test]
fn wasmi_oracle_matches_type_level_export_name_and_exported_state() {
    const MODULE: &str = r#"
        (module
          (memory (export "memory") 1)
          (global $g (export "g") (mut i32) (i32.const 2))
          (func $start
            i32.const 9
            global.set $g
            i32.const 1
            i32.const 8
            i32.store8)
          (start $start)
          (func (export "main") (result i32)
            global.get $g))
    "#;

    let runtime = run_wat_snapshot_with_env(MODULE, "main", &[], &RuntimeEnv {
        observed_memory_export: Some("memory"),
        observed_global_export: Some("g"),
        ..RuntimeEnv::default()
    })
    .expect("wasmi should execute start and export observation");
    assert_eq!(runtime.result_i32, 9);
    assert_eq!(runtime.exported_global_i32, Some(9));
    assert_eq!(runtime.memory_prefix.get(1).copied(), Some(8));

    type Module = typelude_macros::twat! {
        module: r#"
            (module
              (memory (export "memory") 1)
              (global $g (export "g") (mut i32) (i32.const 2))
              (func $start
                i32.const 9
                global.set $g
                i32.const 1
                i32.const 8
                i32.store8)
              (start $start)
              (func (export "main") (result i32)
                global.get $g))
        "#,
    };
    type Final = InvokeExport<Module, typelude_wasm::tstr::TS!("main"), NoArgs>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U9>, TTerm>);
    assert_type_eq_all!(
        <Final as StateExportGlobal<typelude_wasm::tstr::TS!("g")>>::Output,
        WasmGlobal<GlobalMut, WasmI32<U9>>
    );
}

#[test]
fn wasmi_oracle_matches_type_level_active_data_segments() {
    const MODULE: &str = r#"
        (module
          (memory (export "memory") 1)
          (data (i32.const 0) "\2a")
          (func (export "main") (result i32)
            i32.const 0
            i32.load8_u))
    "#;

    let runtime = run_wat_snapshot(MODULE, "main", &[]).expect("wasmi should execute active data");
    assert_eq!(runtime.result_i32, 42);
    assert_eq!(runtime.memory_prefix.first().copied(), Some(42));

    type Module = typelude_macros::twat! {
        module: r#"
            (module
              (memory (export "memory") 1)
              (data (i32.const 0) "\2a")
              (func (export "main") (result i32)
                i32.const 0
                i32.load8_u))
        "#,
    };
    type Final = InvokeFunc<Module, U0, NoArgs>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U42>, TTerm>);
}

#[test]
fn wasmi_oracle_matches_type_level_start_execution() {
    const MODULE: &str = r#"
        (module
          (global $g (mut i32) (i32.const 0))
          (func $start
            i32.const 7
            global.set $g)
          (start $start)
          (func (export "main") (result i32)
            global.get $g))
    "#;

    let runtime = run_wat_snapshot(MODULE, "main", &[]).expect("wasmi should execute start");
    assert_eq!(runtime.result_i32, 7);

    type Module = typelude_macros::twat! {
        module: r#"
            (module
              (global $g (mut i32) (i32.const 0))
              (func $start
                i32.const 7
                global.set $g)
              (start $start)
              (func (export "main") (result i32)
                global.get $g))
        "#,
    };
    type Final = InvokeFunc<Module, U1, NoArgs>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U7>, TTerm>);
}

#[test]
fn wasmi_oracle_matches_type_level_default_table_call_indirect() {
    const MODULE: &str = r#"
        (module
          (type $ret (func (result i32)))
          (table 2 funcref)
          (elem (i32.const 0) $one $two)
          (func $one (type $ret) (result i32)
            i32.const 1)
          (func $two (type $ret) (result i32)
            i32.const 2)
          (func (export "main") (result i32)
            i32.const 1
            call_indirect (type $ret)))
    "#;

    let runtime =
        run_wat_snapshot(MODULE, "main", &[]).expect("wasmi should execute default indirect call");
    assert_eq!(runtime.result_i32, 2);

    type Module = typelude_macros::twat! {
        module: r#"
            (module
              (type $ret (func (result i32)))
              (table 2 funcref)
              (elem (i32.const 0) $one $two)
              (func $one (type $ret) (result i32)
                i32.const 1)
              (func $two (type $ret) (result i32)
                i32.const 2)
              (func (export "main") (result i32)
                i32.const 1
                call_indirect (type $ret)))
        "#,
    };
    type Final = InvokeFunc<Module, U2, NoArgs>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U2>, TTerm>);
}

#[test]
fn wasmi_oracle_matches_type_level_explicit_table_call_indirect() {
    const MODULE: &str = r#"
        (module
          (type $ret (func (result i32)))
          (table (export "t0") 1 funcref)
          (table $t1 1 funcref)
          (elem (table $t1) (i32.const 0) func $three)
          (func $three (type $ret) (result i32)
            i32.const 3)
          (func (export "main") (result i32)
            i32.const 0
            call_indirect $t1 (type $ret)))
    "#;

    let runtime = run_wat_snapshot(MODULE, "main", &[])
        .expect("wasmi should execute explicit indirect call");
    assert_eq!(runtime.result_i32, 3);

    type Module = typelude_macros::twat! {
        module: r#"
            (module
              (type $ret (func (result i32)))
              (table (export "t0") 1 funcref)
              (table $t1 1 funcref)
              (elem (table $t1) (i32.const 0) func $three)
              (func $three (type $ret) (result i32)
                i32.const 3)
              (func (export "main") (result i32)
                i32.const 0
                call_indirect $t1 (type $ret)))
        "#,
    };
    type Final = InvokeFunc<Module, U1, NoArgs>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U3>, TTerm>);
}
