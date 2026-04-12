#![recursion_limit = "65536"]

use core::ops::Add;

use static_assertions::assert_type_eq_all;
use typelude::wasm::{
    GlobalConst, GlobalMut, HostCall, HostCallResult, HostFuncBinding, HostGlobalBinding,
    HostMemoryBinding, HostTableBinding, InvokeExport, InvokeExportWithEnv, InvokeFunc,
    StateBranches, StateExportGlobal, StateExportMemory, StateExportTable, StateStack, TArr,
    TTerm, TableEntry, WasmGlobal, WasmHostEnv, WasmI32, WasmI32Type, WasmMemory, WasmTable,
};
use typenum::{twat, Const, ToUInt, U0, U1, U2, U3, U5, U7, U8, U9, U42, operator_aliases::Sum};

type NoArgs = TTerm;
type OneArg<A> = TArr<WasmI32<A>, TTerm>;
type TwoArgs<A, B> = TArr<WasmI32<A>, TArr<WasmI32<B>, TTerm>>;
type MaxPages = <Const<4294967295> as ToUInt>::Output;

struct HostAdd;

impl<Store, A, B>
    HostCall<
        typelude::wasm::WasmFuncType<
            TArr<WasmI32Type, TArr<WasmI32Type, TTerm>>,
            TArr<WasmI32Type, TTerm>,
        >,
        Store,
        TArr<WasmI32<A>, TArr<WasmI32<B>, TTerm>>,
    > for HostAdd
where
    A: Add<B>,
{
    type Output = HostCallResult<Store, TArr<WasmI32<Sum<A, B>>, TTerm>>;
}

#[test]
fn wasm_wat_invokes_exported_add() {
    type Module = typelude::twat! {
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
    assert_type_eq_all!(<Final as StateBranches>::Output, TTerm);
}

#[test]
fn wasm_wat_supports_internal_calls() {
    type Module = typelude::twat! {
        module: r#"
            (module
              (func $add2 (param i32) (result i32)
                local.get 0
                i32.const 2
                i32.add)
              (func (export "main") (param i32) (result i32)
                local.get 0
                call $add2))
        "#,
    };
    type Final = InvokeFunc<Module, U1, OneArg<U3>>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
}

#[test]
fn wasm_wat_handles_loop_and_br_if() {
    type Module = typelude::twat! {
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
fn wasm_wat_supports_if_without_block_results() {
    type Module = typelude::twat! {
        module: r#"
            (module
              (func (export "main") (param i32) (result i32)
                (local i32)
                i32.const 2
                local.set 1
                local.get 0
                if
                  local.get 0
                  local.set 1
                else
                  i32.const 3
                  local.set 1
                end
                local.get 1))
        "#,
    };
    type Final = InvokeFunc<Module, U0, OneArg<U0>>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U3>, TTerm>);
}

#[test]
fn wasm_wat_supports_select() {
    type Module = typelude::twat! {
        module: r#"
            (module
              (func (export "main") (param i32) (result i32)
                i32.const 2
                i32.const 3
                local.get 0
                select))
        "#,
    };
    type Final = InvokeFunc<Module, U0, OneArg<U1>>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U2>, TTerm>);
}

#[test]
fn wasm_wat_supports_memory_ops_and_size() {
    type Stored8Module = typelude::twat! {
        module: r#"
            (module
              (memory 1)
              (func (export "main") (result i32)
                i32.const 0
                i32.const 255
                i32.store8
                i32.const 0
                i32.load8_u))
        "#,
    };
    type Stored8 = InvokeFunc<Stored8Module, U0, NoArgs>;

    type StoredModule = typelude::twat! {
        module: r#"
            (module
              (memory 1)
              (func (export "main") (result i32)
                i32.const 0
                i32.const 258
                i32.store
                i32.const 1
                i32.load8_u))
        "#,
    };
    type Stored = InvokeFunc<StoredModule, U0, NoArgs>;

    type SizedModule = typelude::twat! {
        module: r#"
            (module
              (memory 2)
              (func (export "main") (result i32)
                memory.size))
        "#,
    };
    type SizedState = InvokeFunc<SizedModule, U0, NoArgs>;

    assert_type_eq_all!(
        <Stored8 as StateStack>::Output,
        TArr<WasmI32<<Const<255> as ToUInt>::Output>, TTerm>
    );
    assert_type_eq_all!(<Stored as StateStack>::Output, TArr<WasmI32<U1>, TTerm>);
    assert_type_eq_all!(<SizedState as StateStack>::Output, TArr<WasmI32<U2>, TTerm>);
}

#[test]
fn wasm_wat_zero_initializes_extra_locals() {
    type Module = typelude::twat! {
        module: r#"
            (module
              (func (export "main") (result i32)
                (local i32)
                local.get 0))
        "#,
    };
    type Final = InvokeFunc<Module, U0, NoArgs>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U0>, TTerm>);
}

#[test]
fn wasm_wat_supports_recursive_fibonacci() {
    type Module = typelude::twat! {
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
fn wasm_wat_supports_global_get_and_set() {
    type Module = typelude::twat! {
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
fn wasm_wat_supports_nonzero_offsets_and_memory_grow() {
    type Module = typelude::twat! {
        module: r#"
            (module
              (memory 1 2)
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
fn wasm_wat_supports_active_data_segments() {
    type Module = typelude::twat! {
        module: r#"
            (module
              (memory 1)
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
fn wasm_wat_executes_start_before_invocation() {
    type Module = typelude::twat! {
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
fn wasm_wat_supports_call_indirect_on_default_table() {
    type Module = typelude::twat! {
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
fn wasm_wat_supports_explicit_table_indices_and_table_exports() {
    type Module = typelude::twat! {
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

#[test]
fn wasm_wat_supports_imported_functions_via_invoke_export() {
    type Module = typelude::twat! {
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
                typelude::wasm::tstr::TS!("host"),
                typelude::wasm::tstr::TS!("add"),
                HostAdd,
            >,
            TTerm,
        >,
        TTerm,
        TTerm,
        TTerm,
    >;
    type Final =
        InvokeExportWithEnv<Module, Env, typelude::wasm::tstr::TS!("main"), TwoArgs<U2, U3>>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
}

#[test]
fn wasm_wat_supports_imported_globals_and_global_exports() {
    type Module = typelude::twat! {
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
                typelude::wasm::tstr::TS!("host"),
                typelude::wasm::tstr::TS!("g"),
                WasmGlobal<GlobalMut, WasmI32<U3>>,
            >,
            TTerm,
        >,
        TTerm,
        TTerm,
    >;
    type Final = InvokeExportWithEnv<Module, Env, typelude::wasm::tstr::TS!("main"), NoArgs>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
    assert_type_eq_all!(
        <Final as StateExportGlobal<typelude::wasm::tstr::TS!("copy")>>::Output,
        WasmGlobal<GlobalConst, WasmI32<U3>>
    );
}

#[test]
fn wasm_wat_supports_imported_memory() {
    type Module = typelude::twat! {
        module: r#"
            (module
              (import "host" "memory" (memory 1))
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
                typelude::wasm::tstr::TS!("host"),
                typelude::wasm::tstr::TS!("memory"),
                WasmMemory<U1, U1, TArr<typelude::wasm::MemoryCell<U0, U42>, TTerm>>,
            >,
            TTerm,
        >,
        TTerm,
    >;
    type Final = InvokeExportWithEnv<Module, Env, typelude::wasm::tstr::TS!("main"), NoArgs>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U42>, TTerm>);
}

#[test]
fn wasm_wat_supports_imported_tables_and_call_indirect() {
    type Module = typelude::twat! {
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
                typelude::wasm::tstr::TS!("host"),
                typelude::wasm::tstr::TS!("table"),
                WasmTable<U1, U1, TTerm>,
            >,
            TTerm,
        >,
    >;
    type Final = InvokeExportWithEnv<Module, Env, typelude::wasm::tstr::TS!("main"), NoArgs>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U7>, TTerm>);
}

#[test]
fn wasm_wat_exposes_memory_and_table_exports_by_name() {
    type Module = typelude::twat! {
        module: r#"
            (module
              (memory (export "memory") 1)
              (table (export "table") 1 funcref)
              (global $g (export "g") (mut i32) (i32.const 2))
              (func $start
                i32.const 9
                global.set $g
                i32.const 1
                i32.const 8
                i32.store8)
              (start $start)
              (func $main (export "main") (result i32)
                global.get $g)
              (elem (i32.const 0) func $main))
        "#,
    };
    type Final = InvokeExport<Module, typelude::wasm::tstr::TS!("main"), NoArgs>;

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U9>, TTerm>);
    assert_type_eq_all!(
        <Final as StateExportGlobal<typelude::wasm::tstr::TS!("g")>>::Output,
        WasmGlobal<GlobalMut, WasmI32<U9>>
    );
    assert_type_eq_all!(
        <Final as StateExportMemory<typelude::wasm::tstr::TS!("memory")>>::Output,
        WasmMemory<U1, MaxPages, TArr<typelude::wasm::MemoryCell<U1, U8>, TTerm>>
    );
    assert_type_eq_all!(
        <Final as StateExportTable<typelude::wasm::tstr::TS!("table")>>::Output,
        WasmTable<U1, U1, TArr<TableEntry<U0, U1>, TTerm>>
    );
}
