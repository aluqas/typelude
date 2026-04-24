//! Pure small-step VM semantics independent of runtime effects.

pub mod frame;
pub mod helpers;
pub mod instr;
pub mod lowering;
pub mod state;
pub mod step;
