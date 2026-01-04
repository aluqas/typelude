//! **Evaluation Infrastructure**
//!
//! Provides core infrastructure for type-level computation.
//! Now re-exports from `typelude-kernel`.

pub use typelude_core::{
    Eval,
    Evaluate,
    Sealed, // Export Sealed!
    app::App,
    bridge::{ECall, ELazyCall, ELit, EPureApp},
};

pub mod expr;
pub use expr::*;
