#![recursion_limit = "65536"]

mod frame;
mod func;
mod helpers;
mod instr;
mod module;
pub mod opcode;
mod run;
mod state;
mod value;

pub use frame::ReturnFrame;
#[doc(hidden)]
pub use frame::{BranchBlock, BranchLoop, ResolvedBlock, ResolvedLoop};
pub use func::WasmFunc;
pub use module::WasmModule;
pub use run::{
    EmptyState, InvokeFunc, ModuleProgramRun, Run, RunWasm, StateBranches, StateLocals,
    StateMemory, StateProgram, StateStack,
};
#[doc(hidden)]
pub use state::MemoryCell;
pub use state::{WasmMemory, WasmState};
pub use typelude_col::{TArr, TTerm};
pub use value::WasmI32;
