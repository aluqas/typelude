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
pub use helpers::call::{HostCall, HostCallResult};
pub use helpers::export::{
    ResolveExportFunc, ResolveExportGlobal, ResolveExportKind, ResolveExportMemory,
    ResolveExportTable,
};
pub use module::{
    ExportFunc, ExportGlobal, ExportMemory, ExportTable, GlobalConst, GlobalMut,
    HostFuncBinding, HostGlobalBinding, HostMemoryBinding, HostTableBinding, ImportFunc,
    ImportGlobal, ImportMemory, ImportTable, InitGlobalGet, InitI32Const, NoLimit, NoStart,
    StartFunc, WasmDataSegment, WasmElemSegment, WasmExport, WasmFuncSpace, WasmFuncType,
    WasmHostEnv, WasmHostFunc, WasmImport, WasmInstance, WasmMemoryDecl, WasmModule,
    WasmResolvedModule, WasmTableDecl, WasmGlobalDecl,
};
pub use run::{
    EmptyHostEnv, EmptyState, InstantiateModule, InvokeExport, InvokeExportWithEnv, InvokeFunc,
    InvokeFuncWithEnv, ModuleProgramRun, Run, RunWasm, StateBranches, StateGlobals, StateLocals,
    StateExportGlobal, StateExportMemory, StateExportTable, StateMemory, StateProgram,
    StateStack, StateStore, StateTables,
};
#[doc(hidden)]
pub use state::MemoryCell;
#[doc(hidden)]
pub use state::{NullFuncRef, TableEntry, WasmGlobal, WasmStore, WasmTable};
pub use state::{WasmMemory, WasmState};
pub use tstr;
pub use typelude_col::{TArr, TTerm};
pub use value::{WasmI32, WasmI32Type};
