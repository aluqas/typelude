use core::ops::Add;

use typelude_col::TArr;
use typelude_std::core::Eval;
use typenum::{U0, operator_aliases::Sum};

use crate::{
    helpers::memory::{
        EncodeI32, LowByte, MemoryGrow, MemoryReadByte, MemoryReadI32, MemoryReadI64,
        MemoryWriteByte, MemoryWriteI32, MemoryWriteI64,
    },
    module::WasmMemArg,
    opcode::{
        OpI32Load, OpI32Load8U, OpI32Store, OpI32Store8, OpI64Load, OpI64Store, OpMemoryGrow,
        OpMemorySize,
    },
    run::Step,
    state::{WasmMemory, WasmState, WasmStore},
    value::{WasmI32, WasmI64},
};

pub trait ResolveMemArg {
    type Offset;
}

impl<Offset> ResolveMemArg for Offset
where
    Offset: typenum::Unsigned,
{
    type Offset = Offset;
}

impl<Align, Offset> ResolveMemArg for WasmMemArg<U0, Align, Offset> {
    type Offset = Offset;
}

impl<
    Module,
    Pages,
    MaxPages,
    Cells,
    Tables,
    Globals,
    Addr,
    Tail,
    Locals,
    Frames,
    Branches,
    Rest,
    MemArg,
> Eval
    for Step<
        WasmState<
            Module,
            WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
            TArr<WasmI32<Addr>, Tail>,
            Locals,
            Frames,
            Branches,
            TArr<OpI32Load<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadI32<Sum<Addr, <MemArg as ResolveMemArg>::Offset>>,
{
    type Output = WasmState<
        Module,
        WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
        TArr<
            WasmI32<
                <WasmMemory<Pages, MaxPages, Cells> as MemoryReadI32<
                    Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
                >>::Output,
            >,
            Tail,
        >,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<
    Module,
    Pages,
    MaxPages,
    Cells,
    Tables,
    Globals,
    Addr,
    Tail,
    Locals,
    Frames,
    Branches,
    Rest,
    MemArg,
> Eval
    for Step<
        WasmState<
            Module,
            WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
            TArr<WasmI32<Addr>, Tail>,
            Locals,
            Frames,
            Branches,
            TArr<OpI64Load<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadI64<Sum<Addr, <MemArg as ResolveMemArg>::Offset>>,
{
    type Output = WasmState<
        Module,
        WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
        TArr<
            WasmI64<
                <WasmMemory<Pages, MaxPages, Cells> as MemoryReadI64<
                    Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
                >>::Output,
            >,
            Tail,
        >,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<
    Module,
    Pages,
    MaxPages,
    Cells,
    Tables,
    Globals,
    Addr,
    ValueT,
    Tail,
    Locals,
    Frames,
    Branches,
    Rest,
    MemArg,
> Eval
    for Step<
        WasmState<
            Module,
            WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
            TArr<WasmI32<ValueT>, TArr<WasmI32<Addr>, Tail>>,
            Locals,
            Frames,
            Branches,
            TArr<OpI32Store<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryWriteI32<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, ValueT>,
{
    type Output = WasmState<
        Module,
        WasmStore<
            <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteI32<
                Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
                ValueT,
            >>::Output,
            Tables,
            Globals,
        >,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<
    Module,
    Pages,
    MaxPages,
    Cells,
    Tables,
    Globals,
    Addr,
    ValueT,
    Tail,
    Locals,
    Frames,
    Branches,
    Rest,
    MemArg,
> Eval
    for Step<
        WasmState<
            Module,
            WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
            TArr<WasmI64<ValueT>, TArr<WasmI32<Addr>, Tail>>,
            Locals,
            Frames,
            Branches,
            TArr<OpI64Store<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryWriteI64<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, ValueT>,
{
    type Output = WasmState<
        Module,
        WasmStore<
            <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteI64<
                Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
                ValueT,
            >>::Output,
            Tables,
            Globals,
        >,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<
    Module,
    Pages,
    MaxPages,
    Cells,
    Tables,
    Globals,
    Addr,
    Tail,
    Locals,
    Frames,
    Branches,
    Rest,
    MemArg,
> Eval
    for Step<
        WasmState<
            Module,
            WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
            TArr<WasmI32<Addr>, Tail>,
            Locals,
            Frames,
            Branches,
            TArr<OpI32Load8U<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Addr, <MemArg as ResolveMemArg>::Offset>>,
{
    type Output = WasmState<
        Module,
        WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
        TArr<
            WasmI32<
                <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                    Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
                >>::Output,
            >,
            Tail,
        >,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<
    Module,
    Pages,
    MaxPages,
    Cells,
    Tables,
    Globals,
    Addr,
    ValueT,
    Tail,
    Locals,
    Frames,
    Branches,
    Rest,
    MemArg,
> Eval
    for Step<
        WasmState<
            Module,
            WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
            TArr<WasmI32<ValueT>, TArr<WasmI32<Addr>, Tail>>,
            Locals,
            Frames,
            Branches,
            TArr<OpI32Store8<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    ValueT: EncodeI32,
    <ValueT as EncodeI32>::Output: LowByte,
    WasmMemory<Pages, MaxPages, Cells>: MemoryWriteByte<
            Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
            <<ValueT as EncodeI32>::Output as LowByte>::Output,
        >,
{
    type Output = WasmState<
        Module,
        WasmStore<
            <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteByte<
                Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
                <<ValueT as EncodeI32>::Output as LowByte>::Output,
            >>::Output,
            Tables,
            Globals,
        >,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Pages, MaxPages, Cells, Tables, Globals, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
            Stack,
            Locals,
            Frames,
            Branches,
            TArr<OpMemorySize<U0>, Rest>,
        >,
    >
{
    type Output = WasmState<
        Module,
        WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
        TArr<WasmI32<Pages>, Stack>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Pages, MaxPages, Cells, Tables, Globals, Delta, Tail, Locals, Frames, Branches, Rest>
    Eval
    for Step<
        WasmState<
            Module,
            WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
            TArr<WasmI32<Delta>, Tail>,
            Locals,
            Frames,
            Branches,
            TArr<OpMemoryGrow<U0>, Rest>,
        >,
    >
where
    WasmMemory<Pages, MaxPages, Cells>: MemoryGrow<Delta>,
{
    type Output = WasmState<
        Module,
        WasmStore<
            <WasmMemory<Pages, MaxPages, Cells> as MemoryGrow<Delta>>::OutputMemory,
            Tables,
            Globals,
        >,
        TArr<WasmI32<<WasmMemory<Pages, MaxPages, Cells> as MemoryGrow<Delta>>::Result>, Tail>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

#[cfg(test)]
mod tests {
    use crate::tests::support::*;

    type GrowOneToTwoPagesModule = ModuleWithMemoryLimits<TTerm, U1, U2>;
    type PreloadedDataModule = ModuleWithData<
        TTerm,
        U1,
        tarr![WasmDataSegment<WasmConstExpr<tarr![OpI32Const<U2>]>, tarr![U7, U8]>],
    >;

    #[test]
    fn memory_size_reports_initial_page_count() {
        type Program = tarr![OpMemorySize];
        type Final = ModuleProgramRun<OnePageModule, Program>;

        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U1>]);
    }

    #[test]
    fn memory_grow_within_max_returns_old_page_count_and_updates_memory() {
        type Program = tarr![OpI32Const<U1>, OpMemoryGrow, OpMemorySize];
        type Final = ModuleProgramRun<GrowOneToTwoPagesModule, Program>;

        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U2>, WasmI32<U1>]);
    }

    #[test]
    fn store_then_load_round_trips_i32_value() {
        type Program = tarr![
            OpI32Const<U0>,
            OpI32Const<U258>,
            OpI32Store<U0>,
            OpI32Const<U0>,
            OpI32Load<U0>
        ];
        type Final = ModuleProgramRun<OnePageModule, Program>;

        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U258>]);
    }

    #[test]
    fn active_data_segment_preloads_memory_before_first_instruction() {
        type Program = tarr![OpI32Const<U2>, OpI32Load8U<U0>];
        type Final = ModuleProgramRun<PreloadedDataModule, Program>;

        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U7>]);
    }
}
