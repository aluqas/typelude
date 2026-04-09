#![recursion_limit = "65536"]

use static_assertions::assert_type_eq_all;
use typelude::wasm::{StateBranches, StateStack, TArr, TTerm, WasmI32};
use typenum::{U0, U1, U2, U3, U5};

#[test]
fn wasm_wat_invokes_exported_add() {
    type Final = typelude::wasm_wat! {
        module: r#"
            (module
              (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add))
        "#,
        invoke: "main",
        args: [U2, U3],
    };

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
    assert_type_eq_all!(<Final as StateBranches>::Output, TTerm);
}

#[test]
fn wasm_wat_supports_internal_calls() {
    type Final = typelude::wasm_wat! {
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
        invoke: "main",
        args: [U3],
    };

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U5>, TTerm>);
}

#[test]
fn wasm_wat_handles_loop_and_br_if() {
    type Final = typelude::wasm_wat! {
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
        invoke: "main",
        args: [U3],
    };

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U0>, TTerm>);
}

#[test]
fn wasm_wat_supports_if_without_block_results() {
    type Final = typelude::wasm_wat! {
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
        invoke: "main",
        args: [U0],
    };

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U3>, TTerm>);
}

#[test]
fn wasm_wat_supports_select() {
    type Final = typelude::wasm_wat! {
        module: r#"
            (module
              (func (export "main") (param i32) (result i32)
                i32.const 2
                i32.const 3
                local.get 0
                select))
        "#,
        invoke: "main",
        args: [U1],
    };

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U2>, TTerm>);
}

#[test]
fn wasm_wat_supports_memory_ops_and_size() {
    type Stored8 = typelude::wasm_wat! {
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
        invoke: "main",
    };

    type Stored = typelude::wasm_wat! {
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
        invoke: "main",
    };

    type SizedState = typelude::wasm_wat! {
        module: r#"
            (module
              (memory 2)
              (func (export "main") (result i32)
                memory.size))
        "#,
        invoke: "main",
    };

    assert_type_eq_all!(
        <Stored8 as StateStack>::Output,
        TArr<WasmI32<<typenum::Const<255> as typenum::ToUInt>::Output>, TTerm>
    );
    assert_type_eq_all!(<Stored as StateStack>::Output, TArr<WasmI32<U1>, TTerm>);
    assert_type_eq_all!(<SizedState as StateStack>::Output, TArr<WasmI32<U2>, TTerm>);
}

#[test]
fn wasm_wat_zero_initializes_extra_locals() {
    type Final = typelude::wasm_wat! {
        module: r#"
            (module
              (func (export "main") (result i32)
                (local i32)
                local.get 0))
        "#,
        invoke: "main",
    };

    assert_type_eq_all!(<Final as StateStack>::Output, TArr<WasmI32<U0>, TTerm>);
}
