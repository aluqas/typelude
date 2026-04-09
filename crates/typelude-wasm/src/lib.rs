#![recursion_limit = "65536"]

mod frame;
mod func;
mod helpers;
mod instr;
pub mod opcode;
mod run;
mod state;
mod value;

pub use typelude_col::{TArr, TTerm};

pub use frame::ReturnFrame;
#[doc(hidden)]
pub use frame::{BranchBlock, BranchLoop, ResolvedBlock, ResolvedLoop};
pub use func::WasmFunc;
pub use run::{
    EmptyState, ProgramRun, Run, RunWasm, StateBranches, StateLocals, StateMemory, StateProgram,
    StateStack,
};
#[doc(hidden)]
pub use state::MemoryCell;
pub use state::{WasmMemory, WasmState};
pub use value::WasmI32;
