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
          (global f32 (f32.const 1.0)))
    "#,
};

fn main() {}
