use core::marker::PhantomData;

use typelude_std::core::Value;

/// 実行中の WebAssembly 仮想機械状態。
///
/// `Module` は解決済みモジュール、`Store` は memory/table/global、
/// `Stack` は operand stack、`Locals` は現在フレームの local 群、
/// `Frames` は return frame stack、`Branches` は branch target stack、
/// `Program` は残り命令列です。
///
/// `Step<WasmState<...>>` は常に `Program` の先頭 opcode を 1 つ消費し、
/// stack や store を更新した新しい `WasmState` を返します。`Program = TTerm`
/// になった状態が runtime の正常終了点です。
#[derive(Debug, Default)]
pub struct WasmState<Module, Store, Stack, Locals, Frames, Branches, Program>(
    pub PhantomData<(Module, Store, Stack, Locals, Frames, Branches, Program)>,
);

impl<Module, Store, Stack, Locals, Frames, Branches, Program> Value
    for WasmState<Module, Store, Stack, Locals, Frames, Branches, Program>
{
}

/// runtime store。
///
/// 線形メモリ、テーブル群、グローバル群をまとめて保持します。
/// WASM の store に相当する runtime 側の可変領域ですが、この実装では
/// 更新後の store 型を新しく作ることで状態遷移を表します。
#[derive(Debug, Default)]
pub struct WasmStore<Memory, Tables, Globals>(PhantomData<(Memory, Tables, Globals)>);

impl<Memory, Tables, Globals> Value for WasmStore<Memory, Tables, Globals> {}

/// 線形メモリの実体。
///
/// `Pages` は現在サイズ、`MaxPages` は上限、`Cells` は byte 単位の疎表現です。
/// 未書き込みのアドレスは `0` として読まれます。success-only memory access は
/// 範囲外アクセスを trait 制約で失敗させ、checked access は `TrapMemoryOob`
/// へ変換します。
#[derive(Debug, Default)]
pub struct WasmMemory<Pages, MaxPages, Cells>(PhantomData<(Pages, MaxPages, Cells)>);

impl<Pages, MaxPages, Cells> Value for WasmMemory<Pages, MaxPages, Cells> {}

#[doc(hidden)]
#[derive(Debug, Default)]
/// 疎な memory cell。
///
/// `Addr` は byte address、`Byte` は 0..255 の typenum 値です。
pub struct MemoryCell<Addr, Byte>(PhantomData<(Addr, Byte)>);

impl<Addr, Byte> Value for MemoryCell<Addr, Byte> {}

/// テーブル実体。
///
/// `Min` / `Max` は table limits、`Entries` は `TableEntry<SlotIdx, FuncIdx>`
/// の疎なリストです。未書き込み slot は `NullFuncRef` として扱われます。
#[derive(Debug, Default)]
pub struct WasmTable<Min, Max, Entries>(PhantomData<(Min, Max, Entries)>);

impl<Min, Max, Entries> Value for WasmTable<Min, Max, Entries> {}

#[doc(hidden)]
#[derive(Debug, Default)]
/// table entry。
///
/// `SlotIdx` に `FuncIdx` の funcref が入っていることを表します。
pub struct TableEntry<SlotIdx, FuncIdx>(PhantomData<(SlotIdx, FuncIdx)>);

impl<SlotIdx, FuncIdx> Value for TableEntry<SlotIdx, FuncIdx> {}

/// null funcref。
///
/// `call_indirect` の checked runtime ではこれに到達すると
/// `TrapCallIndirectNull` になります。
#[derive(Debug, Default)]
pub struct NullFuncRef;

impl Value for NullFuncRef {}

/// グローバル変数の実体。
///
/// `Mutability` は `GlobalConst` または `GlobalMut`、`ValueT` は `WasmI32<_>`
/// などの値型です。 `global.set` は `GlobalMut` に対してのみ実装されます。
#[derive(Debug, Default)]
pub struct WasmGlobal<Mutability, ValueT>(PhantomData<(Mutability, ValueT)>);

impl<Mutability, ValueT> Value for WasmGlobal<Mutability, ValueT> {}
