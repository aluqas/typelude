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

use typelude_macros::twat;

type Module = twat! {
    module: r#"
        (module
          (global (mut i32) (i32.const 1))
          (global i32 (global.get 0)))
    "#,
};

type Bad = typelude::Evaluate<
    typelude::wasm::InstantiateModule<Module, typelude::wasm::EmptyHostEnv>,
>;

fn main() {
    let _: PhantomData<Bad>;
}
