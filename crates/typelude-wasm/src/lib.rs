//! WebAssembly 型レベル実装。
//!
//! WASM 仮想機械の型レベルモデル化。命令セット（opcode）、
//! フレーム、メモリ、グローバル変数、テーブル等の状態管理と、
//! インスタンス化・関数呼び出し・実行の型安全な表現。
//!
//! ## 主要概念
//!
//! - **opcode**: 命令型（numeric、control、memory、local、host等）
//! - **module**: WASM モジュール定義（関数、メモリ、グローバル等）
//! - **instance**: モジュールインスタンス（実行時の状態）
//! - **state**: スタック、メモリ、グローバル変数の型レベル状態
//! - **frame/instr**: 制御フロー・命令実行ステップ

#![recursion_limit = "65536"]

mod frame;
mod func;
mod helpers;
mod instr;
mod module;
pub mod opcode;
mod run;
mod state;
#[cfg(test)]
mod tests;
mod value;

pub use frame::ReturnFrame;
#[doc(hidden)]
pub use frame::{BranchBlock, BranchLoop, ResolvedBlock, ResolvedLoop};
pub use func::WasmFunc;
pub use helpers::{
    call::{HostCall, HostCallResult},
    export::{
        ResolveExportFunc, ResolveExportGlobal, ResolveExportKind, ResolveExportMemory,
        ResolveExportTable,
    },
};
pub use module::{
    ExportFunc, ExportGlobal, ExportMemory, ExportTable, GlobalConst, GlobalMut, HostFuncBinding,
    HostGlobalBinding, HostMemoryBinding, HostTableBinding, ImportFunc, ImportGlobal,
    ImportMemory, ImportTable, InitGlobalGet, InitI32Const, InitI64Const, NoLimit, NoMemoryDecl,
    NoStart, StartFunc, WasmConstExpr, WasmDataSegment, WasmElemSegment, WasmExport,
    WasmFuncSpace, WasmFuncType, WasmGlobalDecl, WasmHostEnv, WasmHostFunc, WasmImport,
    WasmInstance, WasmMemArg, WasmMemoryDecl, WasmModule, WasmModuleMemory, WasmModuleTables,
    WasmResolvedModule, WasmTableDecl,
};
pub use run::{
    EmptyHostEnv, EmptyState, InstantiateModule, InvokeExport, InvokeExportWithEnv, InvokeFunc,
    InvokeFuncWithEnv, ModuleProgramRun, Run, RunWasm, StateBranches, StateExportGlobal,
    StateExportMemory, StateExportTable, StateGlobals, StateLocals, StateMemory, StateProgram,
    StateStack, StateStore, StateTables,
};
#[doc(hidden)]
pub use state::MemoryCell;
#[doc(hidden)]
pub use state::{NullFuncRef, TableEntry, WasmGlobal, WasmStore, WasmTable};
pub use state::{WasmMemory, WasmState};
pub use typelude_col::{TArr, TTerm};
pub use typelude_str;
pub use value::{
    WasmF32, WasmF32Type, WasmF64, WasmF64Type, WasmI32, WasmI32Type, WasmI64, WasmI64Type,
};
