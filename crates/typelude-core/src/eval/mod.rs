//! **Evaluation Infrastructure**
//!
//! Provides core infrastructure for type-level computation.
//! Now re-exports from `typelude-kernel`.

pub use typelude_kernel::{
    Eval, Evaluate, Sealed, // Export Sealed!
    app::{App},
    bridge::{ELit, ECall, ELazyCall, EPureApp},
};

pub mod expr;
pub use expr::*;
