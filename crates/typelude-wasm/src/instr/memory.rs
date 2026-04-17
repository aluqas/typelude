use core::ops::Add;

use typelude_col::TArr;
use typelude_std::core::Eval;
use typenum::{U0, U1, U2, U3, operator_aliases::Sum};

use crate::{
    helpers::memory::{
        DecodeI32From2, DecodeI64From1, DecodeI64From2, DecodeI64From4, EncodeI32, LowByte,
        MemoryGrow, MemoryReadByte, MemoryReadI32, MemoryReadI64, MemoryWriteByte, MemoryWriteI32,
        MemoryWriteI32Low16, MemoryWriteI64, MemoryWriteI64Low8, MemoryWriteI64Low16,
        MemoryWriteI64Low32, SignExtend8ToI32, SignExtend8ToI64, SignExtend16ToI32,
        SignExtend16ToI64, SignExtend32ToI64,
    },
    module::WasmMemArg,
    opcode::{
        OpI32Load, OpI32Load8S, OpI32Load8U, OpI32Load16S, OpI32Load16U, OpI32Store, OpI32Store8,
        OpI32Store16, OpI64Load, OpI64Load8S, OpI64Load8U, OpI64Load16S, OpI64Load16U,
        OpI64Load32S, OpI64Load32U, OpI64Store, OpI64Store8, OpI64Store16, OpI64Store32,
        OpMemoryGrow, OpMemorySize,
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
            TArr<OpI64Load8S<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Addr, <MemArg as ResolveMemArg>::Offset>>,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
    >>::Output: SignExtend8ToI64,
{
    type Output = WasmState<
        Module,
        WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
        TArr<
            WasmI64<
                <<WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                    Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
                >>::Output as SignExtend8ToI64>::Output,
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
            TArr<OpI64Load8U<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Addr, <MemArg as ResolveMemArg>::Offset>>,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
    >>::Output: DecodeI64From1,
{
    type Output = WasmState<
        Module,
        WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
        TArr<
            WasmI64<
                <<WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                    Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
                >>::Output as DecodeI64From1>::Output,
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
            TArr<OpI64Load16S<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<U1>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Addr, <MemArg as ResolveMemArg>::Offset>>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>>,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
    >>::Output: DecodeI64From2<
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
            Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>,
        >>::Output,
    >,
    <<WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
    >>::Output as DecodeI64From2<
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
            Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>,
        >>::Output,
    >>::Output: SignExtend16ToI64,
{
    type Output = WasmState<
        Module,
        WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
        TArr<
            WasmI64<
                <<<WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                    Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
                >>::Output as DecodeI64From2<
                    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                        Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>,
                    >>::Output,
                >>::Output as SignExtend16ToI64>::Output,
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
            TArr<OpI64Load16U<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<U1>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Addr, <MemArg as ResolveMemArg>::Offset>>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>>,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
    >>::Output: DecodeI64From2<
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
            Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>,
        >>::Output,
    >,
{
    type Output = WasmState<
        Module,
        WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
        TArr<
            WasmI64<
                <<WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                    Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
                >>::Output as DecodeI64From2<
                    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                        Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>,
                    >>::Output,
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
            TArr<OpI64Load32S<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<U1> + Add<U2> + Add<U3>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Addr, <MemArg as ResolveMemArg>::Offset>>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U2>>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U3>>,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
    >>::Output: DecodeI64From4<
            <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>,
            >>::Output,
            <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U2>,
            >>::Output,
            <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U3>,
            >>::Output,
        >,
    <<WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
    >>::Output as DecodeI64From4<
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
            Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>,
        >>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
            Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U2>,
        >>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
            Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U3>,
        >>::Output,
    >>::Output: SignExtend32ToI64,
{
    type Output = WasmState<
        Module,
        WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
        TArr<
            WasmI64<
                <<<WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                    Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
                >>::Output as DecodeI64From4<
                    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                        Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>,
                    >>::Output,
                    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                        Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U2>,
                    >>::Output,
                    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                        Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U3>,
                    >>::Output,
                >>::Output as SignExtend32ToI64>::Output,
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
            TArr<OpI64Load32U<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<U1> + Add<U2> + Add<U3>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Addr, <MemArg as ResolveMemArg>::Offset>>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U2>>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U3>>,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
    >>::Output: DecodeI64From4<
            <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>,
            >>::Output,
            <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U2>,
            >>::Output,
            <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U3>,
            >>::Output,
        >,
{
    type Output = WasmState<
        Module,
        WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
        TArr<
            WasmI64<
                <<WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                    Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
                >>::Output as DecodeI64From4<
                    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                        Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>,
                    >>::Output,
                    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                        Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U2>,
                    >>::Output,
                    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                        Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U3>,
                    >>::Output,
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
            TArr<OpI32Load8S<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Addr, <MemArg as ResolveMemArg>::Offset>>,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
    >>::Output: SignExtend8ToI32,
{
    type Output = WasmState<
        Module,
        WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
        TArr<
            WasmI32<
                <<WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                    Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
                >>::Output as SignExtend8ToI32>::Output,
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
            TArr<OpI32Load16S<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<U1>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Addr, <MemArg as ResolveMemArg>::Offset>>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>>,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
    >>::Output: DecodeI32From2<
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
            Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>,
        >>::Output,
    >,
    <<WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
    >>::Output as DecodeI32From2<
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
            Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>,
        >>::Output,
    >>::Output: SignExtend16ToI32,
{
    type Output = WasmState<
        Module,
        WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
        TArr<
            WasmI32<
                <<<WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                    Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
                >>::Output as DecodeI32From2<
                    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                        Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>,
                    >>::Output,
                >>::Output as SignExtend16ToI32>::Output,
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
            TArr<OpI32Load16U<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<U1>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Addr, <MemArg as ResolveMemArg>::Offset>>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryReadByte<Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>>,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
    >>::Output: DecodeI32From2<
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
            Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>,
        >>::Output,
    >,
{
    type Output = WasmState<
        Module,
        WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
        TArr<
            WasmI32<
                <<WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                    Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
                >>::Output as DecodeI32From2<
                    <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<
                        Sum<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, U1>,
                    >>::Output,
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
            TArr<OpI32Store16<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryWriteI32Low16<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, ValueT>,
{
    type Output = WasmState<
        Module,
        WasmStore<
            <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteI32Low16<
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
            TArr<OpI64Store8<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryWriteI64Low8<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, ValueT>,
{
    type Output = WasmState<
        Module,
        WasmStore<
            <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteI64Low8<
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
            TArr<OpI64Store16<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryWriteI64Low16<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, ValueT>,
{
    type Output = WasmState<
        Module,
        WasmStore<
            <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteI64Low16<
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
            TArr<OpI64Store32<MemArg>, Rest>,
        >,
    >
where
    MemArg: ResolveMemArg,
    Addr: Add<<MemArg as ResolveMemArg>::Offset>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryWriteI64Low32<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, ValueT>,
{
    type Output = WasmState<
        Module,
        WasmStore<
            <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteI64Low32<
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
        type Program =
            tarr![OpI32Const<U0>, OpI32Const<U258>, OpI32Store<U0>, OpI32Const<U0>, OpI32Load<U0>];
        type Final = ModuleProgramRun<OnePageModule, Program>;

        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U258>]);
    }

    #[test]
    fn active_data_segment_preloads_memory_before_first_instruction() {
        type Program = tarr![OpI32Const<U2>, OpI32Load8U<U0>];
        type Final = ModuleProgramRun<PreloadedDataModule, Program>;

        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U7>]);
    }

    #[test]
    fn i32_narrow_loads_and_store16_work() {
        type U128T = <typenum::Const<128> as typenum::ToUInt>::Output;
        type U32768T = <typenum::Const<32768> as typenum::ToUInt>::Output;
        type U4294967168 =
            typenum::operator_aliases::Diff<crate::helpers::i32::U4294967296, crate::helpers::i32::U128>;
        type U4294934528 =
            typenum::operator_aliases::Diff<crate::helpers::i32::U4294967296, crate::helpers::i32::U32768>;

        type Load8SProgram =
            tarr![OpI32Const<U0>, OpI32Const<U128T>, OpI32Store8, OpI32Const<U0>, OpI32Load8S];
        type Load8SFinal = ModuleProgramRun<OnePageModule, Load8SProgram>;
        type Load16SProgram =
            tarr![OpI32Const<U0>, OpI32Const<U32768T>, OpI32Store16, OpI32Const<U0>, OpI32Load16S];
        type Load16SFinal = ModuleProgramRun<OnePageModule, Load16SProgram>;
        type Load16UProgram =
            tarr![OpI32Const<U0>, OpI32Const<U258>, OpI32Store16, OpI32Const<U0>, OpI32Load16U];
        type Load16UFinal = ModuleProgramRun<OnePageModule, Load16UProgram>;

        assert_type_eq_all!(<Load8SFinal as StateStack>::Output, tarr![WasmI32<U4294967168>]);
        assert_type_eq_all!(<Load16SFinal as StateStack>::Output, tarr![WasmI32<U4294934528>]);
        assert_type_eq_all!(<Load16UFinal as StateStack>::Output, tarr![WasmI32<U258>]);
    }

    #[test]
    fn i64_narrow_loads_and_stores_work() {
        type U128T = <typenum::Const<128> as typenum::ToUInt>::Output;
        type U32768T = <typenum::Const<32768> as typenum::ToUInt>::Output;
        type U65535T = <typenum::Const<65535> as typenum::ToUInt>::Output;
        type U4294967168 =
            typenum::operator_aliases::Diff<crate::helpers::i32::U4294967296, crate::helpers::i32::U128>;
        type U18446744073709518848 =
            typenum::operator_aliases::Diff<crate::helpers::i64::U18446744073709551615, crate::helpers::i32::U32767>;
        type U18446744073709551488 =
            typenum::operator_aliases::Diff<crate::helpers::i64::U18446744073709551615, crate::helpers::i32::U127>;
        type U18446744073709551615T = crate::helpers::i64::U18446744073709551615;

        type Load8SProgram =
            tarr![OpI32Const<U0>, OpI64Const<U128T>, OpI64Store8, OpI32Const<U0>, OpI64Load8S];
        type Load8SFinal = ModuleProgramRun<OnePageModule, Load8SProgram>;
        type Load8UProgram =
            tarr![OpI32Const<U0>, OpI64Const<U258>, OpI64Store8, OpI32Const<U0>, OpI64Load8U];
        type Load8UFinal = ModuleProgramRun<OnePageModule, Load8UProgram>;
        type Load16SProgram =
            tarr![OpI32Const<U0>, OpI64Const<U32768T>, OpI64Store16, OpI32Const<U0>, OpI64Load16S];
        type Load16SFinal = ModuleProgramRun<OnePageModule, Load16SProgram>;
        type Load16UProgram =
            tarr![OpI32Const<U0>, OpI64Const<U65535T>, OpI64Store16, OpI32Const<U0>, OpI64Load16U];
        type Load16UFinal = ModuleProgramRun<OnePageModule, Load16UProgram>;
        type Load32SProgram = tarr![
            OpI32Const<U0>,
            OpI64Const<U4294967168>,
            OpI64Store32,
            OpI32Const<U0>,
            OpI64Load32S
        ];
        type Load32SFinal = ModuleProgramRun<OnePageModule, Load32SProgram>;
        type Load32UProgram = tarr![
            OpI32Const<U0>,
            OpI64Const<U18446744073709551615T>,
            OpI64Store32,
            OpI32Const<U0>,
            OpI64Load32U
        ];
        type Load32UFinal = ModuleProgramRun<OnePageModule, Load32UProgram>;

        assert_type_eq_all!(<Load8SFinal as StateStack>::Output, tarr![
            WasmI64<U18446744073709551488>
        ]);
        assert_type_eq_all!(<Load8UFinal as StateStack>::Output, tarr![WasmI64<U2>]);
        assert_type_eq_all!(<Load16SFinal as StateStack>::Output, tarr![
            WasmI64<U18446744073709518848>
        ]);
        assert_type_eq_all!(<Load16UFinal as StateStack>::Output, tarr![WasmI64<U65535T>]);
        assert_type_eq_all!(<Load32SFinal as StateStack>::Output, tarr![
            WasmI64<U18446744073709551488>
        ]);
        assert_type_eq_all!(<Load32UFinal as StateStack>::Output, tarr![
            WasmI64<typenum::U4294967295>
        ]);
    }

    #[test]
    fn narrow_memory_ops_support_offsets_and_little_endian_layout() {
        type U287454020 = <typenum::Const<287454020usize> as typenum::ToUInt>::Output;
        type Program = tarr![
            OpI32Const<U0>,
            OpI64Const<U287454020>,
            OpI64Store32<MemArg0<U1>>,
            OpI32Const<U2>,
            OpI32Load8U,
            OpI32Const<U1>,
            OpI64Load32U<MemArg0<U1>>
        ];
        type Final = ModuleProgramRun<OnePageModule, Program>;

        assert_type_eq_all!(<Final as StateStack>::Output, tarr![
            WasmI64<U287454020>,
            WasmI32<typenum::U51>
        ]);
    }
}
