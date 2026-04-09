#![recursion_limit = "65536"]

mod support;

extern crate self as typelude;

pub mod wasm {
    pub use typelude_wasm::*;
}
use static_assertions::assert_type_eq_all;
use typelude_wasm::{InvokeFunc, StateStack, TArr, TTerm, WasmI32};
pub use typenum;
use typenum::{Const, ToUInt, U0, U2, U3, U5, U8};

use crate::support::runtime::{RuntimeSnapshot, run_wat_snapshot};

type NoArgs = TTerm;
type OneArg<A> = TArr<WasmI32<A>, TTerm>;
type TwoArgs<A, B> = TArr<WasmI32<A>, TArr<WasmI32<B>, TTerm>>;

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
    assert_eq!(
        runtime,
        RuntimeSnapshot {
            result_i32: 5,
            memory_pages: None,
            memory_prefix: Vec::new(),
        }
    );

    type Module = typelude_macros::wasm_wat! {
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

    type Module = typelude_macros::wasm_wat! {
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

    type Module = typelude_macros::wasm_wat! {
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

    type Module = typelude_macros::wasm_wat! {
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

    type Module = typelude_macros::wasm_wat! {
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
