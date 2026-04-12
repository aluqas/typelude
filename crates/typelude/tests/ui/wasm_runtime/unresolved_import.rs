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
          (import "host" "add" (func $add (param i32 i32) (result i32)))
          (func (export "main") (param i32 i32) (result i32)
            local.get 0
            local.get 1
            call $add))
    "#,
};

type Bad = typelude::Evaluate<
    typelude::wasm::InstantiateModule<Module, typelude::wasm::EmptyHostEnv>,
>;

fn main() {
    let _: PhantomData<Bad>;
}
