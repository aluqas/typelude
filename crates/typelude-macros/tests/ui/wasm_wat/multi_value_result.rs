#![recursion_limit = "65536"]

extern crate self as typelude;

pub mod core {
    pub use typelude_std::core::*;
    pub use typenum;
}

pub use typelude_std::{Eval, Evaluate};
pub use typelude_wasm as wasm;
pub use typenum;

use typelude_macros::wasm_wat;

type _Bad = wasm_wat! {
    module: r#"
        (module
          (func (export "main") (result i32 i32)
            i32.const 1
            i32.const 2))
    "#,
};

fn main() {}
