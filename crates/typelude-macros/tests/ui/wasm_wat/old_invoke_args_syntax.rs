#![recursion_limit = "65536"]

extern crate self as typelude;

pub mod core {
    pub use typelude_std::core::*;
    pub use typenum;
}

pub use typelude_std::{Eval, Evaluate};
pub use typelude_wasm as wasm;
pub use typenum;

use typelude_macros::twat;

type _Bad = twat! {
    module: r#"
        (module
          (func (export "main") (param i32 i32) (result i32)
            local.get 0
            local.get 1
            i32.add))
    "#,
    invoke: "main",
    args: [typenum::U2, typenum::U3],
};

fn main() {}
