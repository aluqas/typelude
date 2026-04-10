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
pub struct WasmStore<Memory, Tables, Globals>(pub PhantomData<(Memory, Tables, Globals)>);

impl<Memory, Tables, Globals> Value for WasmStore<Memory, Tables, Globals> {}

#[derive(Debug, Default)]
pub struct WasmMemory<Pages, MaxPages, Cells>(pub PhantomData<(Pages, MaxPages, Cells)>);

impl<Pages, MaxPages, Cells> Value for WasmMemory<Pages, MaxPages, Cells> {}

#[doc(hidden)]
#[derive(Debug, Default)]
pub struct MemoryCell<Addr, Byte>(pub PhantomData<(Addr, Byte)>);

impl<Addr, Byte> Value for MemoryCell<Addr, Byte> {}

#[derive(Debug, Default)]
pub struct WasmTable<Min, Max, Entries>(pub PhantomData<(Min, Max, Entries)>);

impl<Min, Max, Entries> Value for WasmTable<Min, Max, Entries> {}

#[doc(hidden)]
#[derive(Debug, Default)]
pub struct TableEntry<SlotIdx, FuncIdx>(pub PhantomData<(SlotIdx, FuncIdx)>);

impl<SlotIdx, FuncIdx> Value for TableEntry<SlotIdx, FuncIdx> {}

#[derive(Debug, Default)]
pub struct NullFuncRef;

impl Value for NullFuncRef {}

#[derive(Debug, Default)]
pub struct WasmGlobal<Mutability, ValueT>(pub PhantomData<(Mutability, ValueT)>);

impl<Mutability, ValueT> Value for WasmGlobal<Mutability, ValueT> {}
