#![recursion_limit = "65536"]

extern crate self as typelude;

pub mod wasm {
    pub use typelude_wasm::*;
}

use static_assertions::assert_type_eq_all;
use typelude_wasm::{
    InvokeExportChecked, StateStack, TArr, TTerm, TrapCallIndirectNull, TrapCallIndirectTableOob,
    TrapCallIndirectTypeMismatch, TrapMemoryOob, TrapUnreachable, WasmI32, WasmTrap,
};
use typenum::U1;

type UnreachableModule = typelude_macros::twat! {
    module: r#"
        (module
          (func (export "main") (result i32)
            unreachable
            i32.const 0))
    "#,
};

type CallIndirectNullModule = typelude_macros::twat! {
    module: r#"
        (module
          (type $ret (func (result i32)))
          (table 1 1 funcref)
          (func (export "main") (result i32)
            i32.const 0
            call_indirect (type $ret)))
    "#,
};

type CallIndirectOobModule = typelude_macros::twat! {
    module: r#"
        (module
          (type $ret (func (result i32)))
          (table 1 1 funcref)
          (func (export "main") (result i32)
            i32.const 1
            call_indirect (type $ret)))
    "#,
};

type CallIndirectTypeMismatchModule = typelude_macros::twat! {
    module: r#"
        (module
          (type $ret0 (func (result i32)))
          (type $ret1 (func (param i32) (result i32)))
          (table 1 1 funcref)
          (elem (i32.const 0) $one)
          (func $one (type $ret0) (result i32)
            i32.const 1)
          (func (export "main") (result i32)
            i32.const 0
            call_indirect (type $ret1)))
    "#,
};

type MemoryOobLoadModule = typelude_macros::twat! {
    module: r#"
        (module
          (memory 1)
          (func (export "main") (result i32)
            i32.const 65536
            i32.load8_u))
    "#,
};

type MemoryOobStoreModule = typelude_macros::twat! {
    module: r#"
        (module
          (memory 1)
          (func (export "main") (result i32)
            i32.const 65536
            i32.const 1
            i32.store8
            i32.const 0))
    "#,
};

type MemoryGrowModule = typelude_macros::twat! {
    module: r#"
        (module
          (memory 1)
          (func (export "main") (result i32)
            i32.const 1
            memory.grow))
    "#,
};

#[test]
fn checked_runtime_traps_unreachable() {
    type Result = InvokeExportChecked<UnreachableModule, typelude_wasm::typelude_str::tstr!("main"), TTerm>;
    assert_type_eq_all!(Result, WasmTrap<TrapUnreachable>);
}

#[test]
fn checked_runtime_traps_call_indirect_failures() {
    type NullResult =
        InvokeExportChecked<CallIndirectNullModule, typelude_wasm::typelude_str::tstr!("main"), TTerm>;
    type OobResult =
        InvokeExportChecked<CallIndirectOobModule, typelude_wasm::typelude_str::tstr!("main"), TTerm>;
    type MismatchResult = InvokeExportChecked<
        CallIndirectTypeMismatchModule,
        typelude_wasm::typelude_str::tstr!("main"),
        TTerm,
    >;

    assert_type_eq_all!(NullResult, WasmTrap<TrapCallIndirectNull>);
    assert_type_eq_all!(OobResult, WasmTrap<TrapCallIndirectTableOob>);
    assert_type_eq_all!(MismatchResult, WasmTrap<TrapCallIndirectTypeMismatch>);
}

#[test]
fn checked_runtime_traps_memory_oob() {
    type LoadResult =
        InvokeExportChecked<MemoryOobLoadModule, typelude_wasm::typelude_str::tstr!("main"), TTerm>;
    type StoreResult =
        InvokeExportChecked<MemoryOobStoreModule, typelude_wasm::typelude_str::tstr!("main"), TTerm>;

    assert_type_eq_all!(LoadResult, WasmTrap<TrapMemoryOob>);
    assert_type_eq_all!(StoreResult, WasmTrap<TrapMemoryOob>);
}

#[test]
fn checked_runtime_keeps_memory_grow_as_value_result() {
    type Result = InvokeExportChecked<MemoryGrowModule, typelude_wasm::typelude_str::tstr!("main"), TTerm>;

    assert_type_eq_all!(<Result as StateStack>::Output, TArr<WasmI32<U1>, TTerm>);
}
