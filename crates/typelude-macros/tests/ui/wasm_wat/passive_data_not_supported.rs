mod typelude {
    pub mod core {
        pub use typelude_std::core::*;
        pub use typenum;
    }

    pub use typelude_std::{Eval, Evaluate};
    pub use typelude_wasm as wasm;
    pub use typenum;
}

use typelude_macros::wasm_wat;

type _Bad = wasm_wat! {
    module: r#"
        (module
          (memory 1)
          (data "x"))
    "#,
};

fn main() {}
