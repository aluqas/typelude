use core::marker::PhantomData;

use typelude_std::core::Value;

#[derive(Debug, Default)]
pub struct WasmState<Module, Store, Stack, Locals, Frames, Branches, Program>(
    pub PhantomData<(Module, Store, Stack, Locals, Frames, Branches, Program)>,
);

impl<Module, Store, Stack, Locals, Frames, Branches, Program> Value
    for WasmState<Module, Store, Stack, Locals, Frames, Branches, Program>
{
}

#[derive(Debug, Default)]
pub struct WasmStore<Memory, Tables, Globals>(PhantomData<(Memory, Tables, Globals)>);

impl<Memory, Tables, Globals> Value for WasmStore<Memory, Tables, Globals> {}

#[derive(Debug, Default)]
pub struct WasmMemory<Pages, MaxPages, Cells>(PhantomData<(Pages, MaxPages, Cells)>);

impl<Pages, MaxPages, Cells> Value for WasmMemory<Pages, MaxPages, Cells> {}

#[doc(hidden)]
#[derive(Debug, Default)]
pub struct MemoryCell<Addr, Byte>(PhantomData<(Addr, Byte)>);

impl<Addr, Byte> Value for MemoryCell<Addr, Byte> {}

#[derive(Debug, Default)]
pub struct WasmTable<Min, Max, Entries>(PhantomData<(Min, Max, Entries)>);

impl<Min, Max, Entries> Value for WasmTable<Min, Max, Entries> {}

#[doc(hidden)]
#[derive(Debug, Default)]
pub struct TableEntry<SlotIdx, FuncIdx>(PhantomData<(SlotIdx, FuncIdx)>);

impl<SlotIdx, FuncIdx> Value for TableEntry<SlotIdx, FuncIdx> {}

#[derive(Debug, Default)]
pub struct NullFuncRef;

impl Value for NullFuncRef {}

#[derive(Debug, Default)]
pub struct WasmGlobal<Mutability, ValueT>(PhantomData<(Mutability, ValueT)>);

impl<Mutability, ValueT> Value for WasmGlobal<Mutability, ValueT> {}
