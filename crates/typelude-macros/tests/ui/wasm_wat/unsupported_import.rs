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
          (import "env" "x" (func (param f32))))
    "#,
};

fn main() {}
