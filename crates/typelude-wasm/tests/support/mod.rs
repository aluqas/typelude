#![allow(dead_code, unused_imports)]

pub mod runtime;

use typelude_std::core::Evaluate;
use typelude_wasm::RunWasm;

pub type Run<Initial> = Evaluate<RunWasm<Initial>>;

pub use typelude_wasm::{
    ModuleProgramRun, StateBranches, StateGlobals, StateLocals, StateMemory, StateProgram,
    StateStack,
};
