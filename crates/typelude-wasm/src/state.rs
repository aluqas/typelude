use core::marker::PhantomData;

use typelude_std::core::Value;

#[derive(Debug, Default)]
pub struct WasmState<Module, Stack, Locals, Memory, Frames, Branches, Program>(
    PhantomData<(Module, Stack, Locals, Memory, Frames, Branches, Program)>,
);

impl<Module, Stack, Locals, Memory, Frames, Branches, Program> Value
    for WasmState<Module, Stack, Locals, Memory, Frames, Branches, Program>
{
}

#[derive(Debug, Default)]
pub struct WasmMemory<Pages, Cells>(pub PhantomData<(Pages, Cells)>);

impl<Pages, Cells> Value for WasmMemory<Pages, Cells> {}

#[doc(hidden)]
#[derive(Debug, Default)]
pub struct MemoryCell<Addr, Byte>(pub PhantomData<(Addr, Byte)>);

impl<Addr, Byte> Value for MemoryCell<Addr, Byte> {}
