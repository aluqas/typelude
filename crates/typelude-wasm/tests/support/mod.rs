#![allow(dead_code, unused_imports)]

use typelude_std::core::Evaluate;
use typelude_wasm::RunWasm;

pub type Run<Initial> = Evaluate<RunWasm<Initial>>;

pub use typelude_wasm::{
    ProgramRun, StateBranches, StateLocals, StateMemory, StateProgram, StateStack,
};
