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
          (table 1 funcref)
          (func $f)
          (elem (i32.const 0) funcref (ref.func $f)))
    "#,
};

fn main() {}
