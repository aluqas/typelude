#![recursion_limit = "65536"]

extern crate self as typelude;

pub mod core {
    pub use typelude_std::core::*;
    pub use typenum;
}

pub use typelude_std::{Eval, Evaluate};
pub use typelude_wasm as wasm;
pub use typenum;

use ::core::marker::PhantomData;

use typelude_macros::wasm_wat;

type Module = wasm_wat! {
    module: r#"
        (module
          (import "host" "base" (global (mut i32)))
          (memory 1)
          (data (global.get 0) "\2a")
          (func (export "main") (result i32)
            i32.const 0
            i32.load8_u))
    "#,
};

type Env = typelude::wasm::WasmHostEnv<
    typelude::wasm::TTerm,
    typelude::wasm::TArr<
        typelude::wasm::HostGlobalBinding<
            typelude::wasm::tstr::TS!("host"),
            typelude::wasm::tstr::TS!("base"),
            typelude::wasm::WasmGlobal<
                typelude::wasm::GlobalMut,
                typelude::wasm::WasmI32<typenum::U0>,
            >,
        >,
        typelude::wasm::TTerm,
    >,
    typelude::wasm::TTerm,
    typelude::wasm::TTerm,
>;

type Bad = typelude::Evaluate<typelude::wasm::InstantiateModule<Module, Env>>;

fn main() {
    let _: PhantomData<Bad>;
}
