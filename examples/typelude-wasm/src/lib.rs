//! WebAssembly の型レベル実行系。
//!
//! この crate は WebAssembly モジュール、インスタンス化、operand stack、
//! 線形メモリ、テーブル、グローバル、制御フローを Rust の型として表現し、
//! `Eval` を通じて small-step に評価します。`twat!` は WAT 文字列を
//! `WasmModule<...>` に lower し、この crate の runtime surface
//! がそれを実行します。
//!
//! ## レイヤ構成
//!
//! - `module`: import / function / memory / table / global / export
//!   を持つモジュール定義
//! - `state`: 実行中の store, operand stack, locals, frames, branches, program
//! - `opcode`: WASM 命令そのものを表すマーカー型
//! - `helpers`: index 解決、import/export 解決、call/memory/table
//!   操作などの型レベル補助
//! - `instr`: `Step` / `CheckedStep` に対する opcode の意味論
//! - `run`: インスタンス化と実行の入口、および結果取り出し用 alias / trait
//!
//! ## Runtime Surface
//!
//! - `Run` / `Invoke*`: 従来の success-only runtime
//! - `RunChecked` / `Invoke*Checked`: trap-aware runtime
//!
//! success-only runtime では、一部の失敗は trait 未解決や compile-fail
//! として表面化します。 checked runtime では `WasmDone<State>` または
//! `WasmTrap<Reason>` を返し、 `unreachable`, `call_indirect`, memory OOB
//! などの失敗を型レベル outcome として扱います。
//!
//! ## `twat!` との関係
//!
//! `typelude-macros::twat!` は WAT を parse / validate / lower し、
//! この crate の `WasmModule`, `WasmFunc`, `WasmMemArg`, opcode
//! 群へ変換します。 frontend 側の validation で single-memory や memarg
//! 制約を締め、 runtime 側ではその前提のもとで命令意味論を評価します。
//!
//! ## 評価フロー
//!
//! 典型的な実行は次の順序で進みます。
//!
//! 1. `WasmModule<...>` を `InstantiateModule<Module, Env>` で `WasmInstance`
//!    にする
//! 2. `BuildInvokeState` または `BuildProgramState` で初期 `WasmState` を作る
//! 3. `RunWasm` または `RunCheckedWasm` が `Program` の先頭 opcode を 1
//!    つずつ評価する
//! 4. 各 opcode の意味論は `instr::*` の `Step<WasmState<...>>` /
//!    `CheckedStep<...>` に委譲される
//! 5. `Program = TTerm` になった時点で、legacy runtime は final `WasmState`、
//!    checked runtime は `WasmDone<WasmState<...>>` を返す
//!
//! `helpers::*` はこの流れの補助層です。例えば `helpers::instance` は
//! import 解決・data segment 適用・elem segment 適用を担当し、
//! `helpers::call` は引数 pop と locals 構築、`helpers::memory` は byte-level
//! memory access、`helpers::table` は `call_indirect` 用の table lookup
//! を担当します。
//!
//! ## 失敗モードの考え方
//!
//! この実験実装には 2 種類の失敗表現があります。
//!
//! - 型レベル前提を満たさないものは trait 未解決として compile-time に失敗する
//! - WASM runtime trap として扱いたいものは checked runtime で
//!   `WasmTrap<Reason>` になる
//!
//! `Run` / `Invoke*` は前者を多く含む success-only surface です。
//! `RunChecked` / `Invoke*Checked` は `unreachable`, `call_indirect` の
//! null/OOB/type mismatch, memory load/store の OOB を明示的な outcome
//! として返します。

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
