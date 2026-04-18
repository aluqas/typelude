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

#[doc(hidden)]
#[macro_export]
macro_rules! wasm_uint {
    ($value:literal) => {
        <$crate::typenum::Const<$value> as $crate::typenum::ToUInt>::Output
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! wasm_u64_uint {
    (18446744073709551615) => {
        $crate::typenum::operator_aliases::Or<
            <$crate::typenum::Const<9223372036854775808> as $crate::typenum::ToUInt>::Output,
            <$crate::typenum::Const<9223372036854775807> as $crate::typenum::ToUInt>::Output
        >
    };
    ($value:literal) => {
        $crate::wasm_uint!($value)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! wasm_u32_bits_le {
    ($b0:literal, $b1:literal, $b2:literal, $b3:literal) => {
        $crate::typenum::operator_aliases::Or<
            $crate::typenum::operator_aliases::Or<
                $crate::wasm_uint!($b0),
                $crate::typenum::operator_aliases::Shleft<
                    $crate::wasm_uint!($b1),
                    $crate::wasm_uint!(8),
                >,
            >,
            $crate::typenum::operator_aliases::Or<
                $crate::typenum::operator_aliases::Shleft<
                    $crate::wasm_uint!($b2),
                    $crate::wasm_uint!(16),
                >,
                $crate::typenum::operator_aliases::Shleft<
                    $crate::wasm_uint!($b3),
                    $crate::wasm_uint!(24),
                >,
            >,
        >
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! wasm_u64_bits_le {
    (
        $b0:literal, $b1:literal, $b2:literal, $b3:literal,
        $b4:literal, $b5:literal, $b6:literal, $b7:literal
    ) => {
        $crate::typenum::operator_aliases::Or<
            $crate::typenum::operator_aliases::Or<
                $crate::typenum::operator_aliases::Or<
                    $crate::wasm_uint!($b0),
                    $crate::typenum::operator_aliases::Shleft<
                        $crate::wasm_uint!($b1),
                        $crate::wasm_uint!(8),
                    >,
                >,
                $crate::typenum::operator_aliases::Or<
                    $crate::typenum::operator_aliases::Shleft<
                        $crate::wasm_uint!($b2),
                        $crate::wasm_uint!(16),
                    >,
                    $crate::typenum::operator_aliases::Shleft<
                        $crate::wasm_uint!($b3),
                        $crate::wasm_uint!(24),
                    >,
                >,
            >,
            $crate::typenum::operator_aliases::Or<
                $crate::typenum::operator_aliases::Or<
                    $crate::typenum::operator_aliases::Shleft<
                        $crate::wasm_uint!($b4),
                        $crate::wasm_uint!(32),
                    >,
                    $crate::typenum::operator_aliases::Shleft<
                        $crate::wasm_uint!($b5),
                        $crate::wasm_uint!(40),
                    >,
                >,
                $crate::typenum::operator_aliases::Or<
                    $crate::typenum::operator_aliases::Shleft<
                        $crate::wasm_uint!($b6),
                        $crate::wasm_uint!(48),
                    >,
                    $crate::typenum::operator_aliases::Shleft<
                        $crate::wasm_uint!($b7),
                        $crate::wasm_uint!(56),
                    >,
                >,
            >,
        >
    };
}

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
    CheckedStep, EmptyHostEnv, EmptyState, InstantiateModule, InvokeExport, InvokeExportChecked,
    InvokeExportCheckedWithEnv, InvokeExportWithEnv, InvokeFunc, InvokeFuncChecked,
    InvokeFuncCheckedWithEnv, InvokeFuncWithEnv, ModuleProgramRun, ModuleProgramRunChecked, Run,
    RunChecked, RunCheckedWasm, RunWasm, StateBranches, StateExportGlobal, StateExportMemory,
    StateExportTable, StateGlobals, StateLocals, StateMemory, StateProgram, StateStack,
    StateStore, StateTables, TrapCallIndirectNull, TrapCallIndirectTableOob,
    TrapCallIndirectTypeMismatch, TrapMemoryOob, TrapUnreachable, WasmDone, WasmTrap,
};
#[doc(hidden)]
pub use state::MemoryCell;
#[doc(hidden)]
pub use state::{NullFuncRef, TableEntry, WasmGlobal, WasmStore, WasmTable};
pub use state::{WasmMemory, WasmState};
pub use typelude_col::{TArr, TTerm};
pub use typelude_str;
pub use typenum;
pub use value::{
    WasmF32, WasmF32Type, WasmF64, WasmF64Type, WasmI32, WasmI32Type, WasmI64, WasmI64Type,
};

#[doc(hidden)]
pub mod twat_prelude {
    pub use typelude_col::{TArr, TTerm, tarr};
    pub use typelude_str::tstr;

    pub use crate::{
        ExportFunc, ExportGlobal, ExportMemory, ExportTable, GlobalConst, GlobalMut, ImportFunc,
        ImportGlobal, ImportMemory, ImportTable, NoLimit, NoMemoryDecl, NoStart, StartFunc,
        WasmConstExpr, WasmDataSegment, WasmElemSegment, WasmExport, WasmF32Type, WasmF64Type,
        WasmFunc, WasmFuncSpace, WasmFuncType, WasmGlobalDecl, WasmI32Type, WasmI64Type,
        WasmImport, WasmMemArg, WasmMemoryDecl, WasmModuleMemory, WasmModuleTables, WasmTableDecl,
        opcode::*, wasm_u32_bits_le, wasm_u64_bits_le, wasm_u64_uint, wasm_uint,
    };
}
