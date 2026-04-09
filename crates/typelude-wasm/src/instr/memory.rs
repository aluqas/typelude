use typelude_col::TArr;
use typelude_std::core::Eval;

use crate::{
    helpers::memory::{
        EncodeI32, LowByte, MemoryReadByte, MemoryReadI32, MemoryWriteByte, MemoryWriteI32,
    },
    opcode::{OpI32Load, OpI32Load8U, OpI32Store, OpI32Store8, OpMemorySize},
    run::Step,
    state::{WasmMemory, WasmState},
    value::WasmI32,
};

impl<Module, Addr, Stack, Locals, Pages, Cells, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            TArr<WasmI32<Addr>, Stack>,
            Locals,
            WasmMemory<Pages, Cells>,
            Frames,
            Branches,
            TArr<OpI32Load8U, Rest>,
        >,
    >
where
    WasmMemory<Pages, Cells>: MemoryReadByte<Addr>,
{
    type Output = WasmState<
        Module,
        TArr<WasmI32<<WasmMemory<Pages, Cells> as MemoryReadByte<Addr>>::Output>, Stack>,
        Locals,
        WasmMemory<Pages, Cells>,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, ValueT, Addr, Stack, Locals, Pages, Cells, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            TArr<WasmI32<ValueT>, TArr<WasmI32<Addr>, Stack>>,
            Locals,
            WasmMemory<Pages, Cells>,
            Frames,
            Branches,
            TArr<OpI32Store8, Rest>,
        >,
    >
where
    ValueT: EncodeI32,
    <ValueT as EncodeI32>::Output: LowByte,
    WasmMemory<Pages, Cells>:
        MemoryWriteByte<Addr, <<ValueT as EncodeI32>::Output as LowByte>::Output>,
{
    type Output = WasmState<
        Module,
        Stack,
        Locals,
        <WasmMemory<Pages, Cells> as MemoryWriteByte<
            Addr,
            <<ValueT as EncodeI32>::Output as LowByte>::Output,
        >>::Output,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Addr, Stack, Locals, Pages, Cells, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            TArr<WasmI32<Addr>, Stack>,
            Locals,
            WasmMemory<Pages, Cells>,
            Frames,
            Branches,
            TArr<OpI32Load, Rest>,
        >,
    >
where
    WasmMemory<Pages, Cells>: MemoryReadI32<Addr>,
{
    type Output = WasmState<
        Module,
        TArr<WasmI32<<WasmMemory<Pages, Cells> as MemoryReadI32<Addr>>::Output>, Stack>,
        Locals,
        WasmMemory<Pages, Cells>,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, ValueT, Addr, Stack, Locals, Pages, Cells, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            TArr<WasmI32<ValueT>, TArr<WasmI32<Addr>, Stack>>,
            Locals,
            WasmMemory<Pages, Cells>,
            Frames,
            Branches,
            TArr<OpI32Store, Rest>,
        >,
    >
where
    WasmMemory<Pages, Cells>: MemoryWriteI32<Addr, ValueT>,
{
    type Output = WasmState<
        Module,
        Stack,
        Locals,
        <WasmMemory<Pages, Cells> as MemoryWriteI32<Addr, ValueT>>::Output,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Stack, Locals, Pages, Cells, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Stack,
            Locals,
            WasmMemory<Pages, Cells>,
            Frames,
            Branches,
            TArr<OpMemorySize, Rest>,
        >,
    >
{
    type Output = WasmState<
        Module,
        TArr<WasmI32<Pages>, Stack>,
        Locals,
        WasmMemory<Pages, Cells>,
        Frames,
        Branches,
        Rest,
    >;
}
