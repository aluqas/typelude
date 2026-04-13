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
          (global (export "g") i32 (i32.const 1)))
    "#,
};

type Instance = typelude::Evaluate<
    typelude::wasm::InstantiateModule<Module, typelude::wasm::EmptyHostEnv>,
>;
type Bad = <Instance as typelude::wasm::ResolveExportFunc<
    typelude::wasm::typelude_str::tstr!("g"),
>>::Output;

fn main() {
    let _: PhantomData<Bad>;
}
