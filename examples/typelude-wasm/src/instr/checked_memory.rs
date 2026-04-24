//! memory opcode の checked 意味論。
//!
//! checked memory helper は成功時に `CheckedMemoryRead` /
//! `CheckedMemoryWrite`、 範囲外時に `MemoryAccessOutOfBounds`
//! を返します。このファイルはそれらを `WasmDone<State>` または
//! `WasmTrap<TrapMemoryOob>` へ写像します。 `memory.grow` は WASM 仕様通り trap
//! ではなく値で失敗を表すため、infallible 側で扱います。

use core::ops::Add;

use typelude_col::TArr;
use typelude_std::core::Eval;
use typenum::operator_aliases::Sum;

use crate::{
    helpers::memory::{
        CheckedMemoryRead, CheckedMemoryWrite, DecodeI64From1, EncodeI32, EncodeI64, LowByte,
        MemoryAccessOutOfBounds, MemoryReadByteChecked, MemoryReadI32Checked,
        MemoryReadI32Low16Checked, MemoryReadI64Checked, MemoryReadI64Low16Checked,
        MemoryReadI64Low32Checked, MemoryWriteByteChecked, MemoryWriteI32Checked,
        MemoryWriteI32Low16Checked, MemoryWriteI64Checked, MemoryWriteI64Low16Checked,
        MemoryWriteI64Low32Checked, SignExtend8ToI32, SignExtend8ToI64, SignExtend16ToI32,
        SignExtend16ToI64, SignExtend32ToI64,
    },
    instr::memory::ResolveMemArg,
    opcode::{
        OpI32Load, OpI32Load8S, OpI32Load8U, OpI32Load16S, OpI32Load16U, OpI32Store, OpI32Store8,
        OpI32Store16, OpI64Load, OpI64Load8S, OpI64Load8U, OpI64Load16S, OpI64Load16U,
        OpI64Load32S, OpI64Load32U, OpI64Store, OpI64Store8, OpI64Store16, OpI64Store32,
    },
    run::{CheckedStep, TrapMemoryOob, WasmDone, WasmTrap},
    state::{WasmMemory, WasmState, WasmStore},
    value::{WasmI32, WasmI64},
};

/// checked `i32.load` 系の読出し結果を outcome へ変換する bridge trait。
pub trait CheckedLoadI32Outcome<
    Module,
    Memory,
    Tables,
    Globals,
    Tail,
    Locals,
    Frames,
    Branches,
    Rest,
>
{
    type Output;
}

impl<Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    CheckedLoadI32Outcome<Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    for MemoryAccessOutOfBounds
{
    type Output = WasmTrap<TrapMemoryOob>;
}

impl<ValueT, Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    CheckedLoadI32Outcome<Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    for CheckedMemoryRead<ValueT>
{
    type Output = WasmDone<
        WasmState<
            Module,
            WasmStore<Memory, Tables, Globals>,
            TArr<WasmI32<ValueT>, Tail>,
            Locals,
            Frames,
            Branches,
            Rest,
        >,
    >;
}

/// checked `i64.load` 系の読出し結果を outcome へ変換する bridge trait。
pub trait CheckedLoadI64Outcome<
    Module,
    Memory,
    Tables,
    Globals,
    Tail,
    Locals,
    Frames,
    Branches,
    Rest,
>
{
    type Output;
}

impl<Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    CheckedLoadI64Outcome<Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    for MemoryAccessOutOfBounds
{
    type Output = WasmTrap<TrapMemoryOob>;
}

impl<ValueT, Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    CheckedLoadI64Outcome<Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    for CheckedMemoryRead<ValueT>
{
    type Output = WasmDone<
        WasmState<
            Module,
            WasmStore<Memory, Tables, Globals>,
            TArr<WasmI64<ValueT>, Tail>,
            Locals,
            Frames,
            Branches,
            Rest,
        >,
    >;
}

/// checked `i32.load8_s` の読出し結果を outcome へ変換する bridge trait。
pub trait CheckedLoadI32SignExtend8Outcome<
    Module,
    Memory,
    Tables,
    Globals,
    Tail,
    Locals,
    Frames,
    Branches,
    Rest,
>
{
    type Output;
}

impl<Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    CheckedLoadI32SignExtend8Outcome<
        Module,
        Memory,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    > for MemoryAccessOutOfBounds
{
    type Output = WasmTrap<TrapMemoryOob>;
}

impl<ValueT, Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    CheckedLoadI32SignExtend8Outcome<
        Module,
        Memory,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    > for CheckedMemoryRead<ValueT>
where
    ValueT: SignExtend8ToI32,
{
    type Output = WasmDone<
        WasmState<
            Module,
            WasmStore<Memory, Tables, Globals>,
            TArr<WasmI32<<ValueT as SignExtend8ToI32>::Output>, Tail>,
            Locals,
            Frames,
            Branches,
            Rest,
        >,
    >;
}

/// checked `i32.load16_s` の読出し結果を outcome へ変換する bridge trait。
pub trait CheckedLoadI32SignExtend16Outcome<
    Module,
    Memory,
    Tables,
    Globals,
    Tail,
    Locals,
    Frames,
    Branches,
    Rest,
>
{
    type Output;
}

impl<Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    CheckedLoadI32SignExtend16Outcome<
        Module,
        Memory,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    > for MemoryAccessOutOfBounds
{
    type Output = WasmTrap<TrapMemoryOob>;
}

impl<ValueT, Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    CheckedLoadI32SignExtend16Outcome<
        Module,
        Memory,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    > for CheckedMemoryRead<ValueT>
where
    ValueT: SignExtend16ToI32,
{
    type Output = WasmDone<
        WasmState<
            Module,
            WasmStore<Memory, Tables, Globals>,
            TArr<WasmI32<<ValueT as SignExtend16ToI32>::Output>, Tail>,
            Locals,
            Frames,
            Branches,
            Rest,
        >,
    >;
}

/// checked `i64.load8_s` の読出し結果を outcome へ変換する bridge trait。
pub trait CheckedLoadI64SignExtend8Outcome<
    Module,
    Memory,
    Tables,
    Globals,
    Tail,
    Locals,
    Frames,
    Branches,
    Rest,
>
{
    type Output;
}

impl<Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    CheckedLoadI64SignExtend8Outcome<
        Module,
        Memory,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    > for MemoryAccessOutOfBounds
{
    type Output = WasmTrap<TrapMemoryOob>;
}

impl<ValueT, Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    CheckedLoadI64SignExtend8Outcome<
        Module,
        Memory,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    > for CheckedMemoryRead<ValueT>
where
    ValueT: SignExtend8ToI64,
{
    type Output = WasmDone<
        WasmState<
            Module,
            WasmStore<Memory, Tables, Globals>,
            TArr<WasmI64<<ValueT as SignExtend8ToI64>::Output>, Tail>,
            Locals,
            Frames,
            Branches,
            Rest,
        >,
    >;
}

/// checked `i64.load8_u` の読出し結果を outcome へ変換する bridge trait。
pub trait CheckedLoadI64ZeroExtend8Outcome<
    Module,
    Memory,
    Tables,
    Globals,
    Tail,
    Locals,
    Frames,
    Branches,
    Rest,
>
{
    type Output;
}

impl<Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    CheckedLoadI64ZeroExtend8Outcome<
        Module,
        Memory,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    > for MemoryAccessOutOfBounds
{
    type Output = WasmTrap<TrapMemoryOob>;
}

impl<ValueT, Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    CheckedLoadI64ZeroExtend8Outcome<
        Module,
        Memory,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    > for CheckedMemoryRead<ValueT>
where
    ValueT: DecodeI64From1,
{
    type Output = WasmDone<
        WasmState<
            Module,
            WasmStore<Memory, Tables, Globals>,
            TArr<WasmI64<<ValueT as DecodeI64From1>::Output>, Tail>,
            Locals,
            Frames,
            Branches,
            Rest,
        >,
    >;
}

/// checked `i64.load16_s` の読出し結果を outcome へ変換する bridge trait。
pub trait CheckedLoadI64SignExtend16Outcome<
    Module,
    Memory,
    Tables,
    Globals,
    Tail,
    Locals,
    Frames,
    Branches,
    Rest,
>
{
    type Output;
}

impl<Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    CheckedLoadI64SignExtend16Outcome<
        Module,
        Memory,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    > for MemoryAccessOutOfBounds
{
    type Output = WasmTrap<TrapMemoryOob>;
}

impl<ValueT, Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    CheckedLoadI64SignExtend16Outcome<
        Module,
        Memory,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    > for CheckedMemoryRead<ValueT>
where
    ValueT: SignExtend16ToI64,
{
    type Output = WasmDone<
        WasmState<
            Module,
            WasmStore<Memory, Tables, Globals>,
            TArr<WasmI64<<ValueT as SignExtend16ToI64>::Output>, Tail>,
            Locals,
            Frames,
            Branches,
            Rest,
        >,
    >;
}

/// checked `i64.load32_s` の読出し結果を outcome へ変換する bridge trait。
pub trait CheckedLoadI64SignExtend32Outcome<
    Module,
    Memory,
    Tables,
    Globals,
    Tail,
    Locals,
    Frames,
    Branches,
    Rest,
>
{
    type Output;
}

impl<Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    CheckedLoadI64SignExtend32Outcome<
        Module,
        Memory,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    > for MemoryAccessOutOfBounds
{
    type Output = WasmTrap<TrapMemoryOob>;
}

impl<ValueT, Module, Memory, Tables, Globals, Tail, Locals, Frames, Branches, Rest>
    CheckedLoadI64SignExtend32Outcome<
        Module,
        Memory,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    > for CheckedMemoryRead<ValueT>
where
    ValueT: SignExtend32ToI64,
{
    type Output = WasmDone<
        WasmState<
            Module,
            WasmStore<Memory, Tables, Globals>,
            TArr<WasmI64<<ValueT as SignExtend32ToI64>::Output>, Tail>,
            Locals,
            Frames,
            Branches,
            Rest,
        >,
    >;
}

/// checked store 系の書込み結果を outcome へ変換する bridge trait。
pub trait CheckedStoreOutcome<Module, Tables, Globals, Stack, Locals, Frames, Branches, Rest> {
    type Output;
}

impl<Module, Tables, Globals, Stack, Locals, Frames, Branches, Rest>
    CheckedStoreOutcome<Module, Tables, Globals, Stack, Locals, Frames, Branches, Rest>
    for MemoryAccessOutOfBounds
{
    type Output = WasmTrap<TrapMemoryOob>;
}

impl<NewMemory, Module, Tables, Globals, Stack, Locals, Frames, Branches, Rest>
    CheckedStoreOutcome<Module, Tables, Globals, Stack, Locals, Frames, Branches, Rest>
    for CheckedMemoryWrite<NewMemory>
{
    type Output = WasmDone<
        WasmState<
            Module,
            WasmStore<NewMemory, Tables, Globals>,
            Stack,
            Locals,
            Frames,
            Branches,
            Rest,
        >,
    >;
}

macro_rules! impl_checked_load_i32 {
    ($op:ident, $helper:ident, $outcome:ident $(, $extra_bound:ty : $extra_trait:ident < $extra_arg:ty > )? ) => {
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
            for CheckedStep<
                WasmState<
                    Module,
                    WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
                    TArr<WasmI32<Addr>, Tail>,
                    Locals,
                    Frames,
                    Branches,
                    TArr<$op<MemArg>, Rest>,
                >,
            >
        where
            MemArg: ResolveMemArg,
            Addr: Add<<MemArg as ResolveMemArg>::Offset>,
            $( $extra_bound: $extra_trait<$extra_arg>, )?
            WasmMemory<Pages, MaxPages, Cells>:
                $helper<Sum<Addr, <MemArg as ResolveMemArg>::Offset>>,
            <WasmMemory<Pages, MaxPages, Cells> as $helper<
                Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
            >>::Output:
                $outcome<Module, WasmMemory<Pages, MaxPages, Cells>, Tables, Globals, Tail, Locals, Frames, Branches, Rest>,
        {
            type Output = <<WasmMemory<Pages, MaxPages, Cells> as $helper<
                Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
            >>::Output as $outcome<
                Module,
                WasmMemory<Pages, MaxPages, Cells>,
                Tables,
                Globals,
                Tail,
                Locals,
                Frames,
                Branches,
                Rest,
            >>::Output;
        }
    };
}

macro_rules! impl_checked_load_i64 {
    ($op:ident, $helper:ident, $outcome:ident $(, $extra_bound:ty : $extra_trait:ident < $extra_arg:ty > )? ) => {
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
            for CheckedStep<
                WasmState<
                    Module,
                    WasmStore<WasmMemory<Pages, MaxPages, Cells>, Tables, Globals>,
                    TArr<WasmI32<Addr>, Tail>,
                    Locals,
                    Frames,
                    Branches,
                    TArr<$op<MemArg>, Rest>,
                >,
            >
        where
            MemArg: ResolveMemArg,
            Addr: Add<<MemArg as ResolveMemArg>::Offset>,
            $( $extra_bound: $extra_trait<$extra_arg>, )?
            WasmMemory<Pages, MaxPages, Cells>:
                $helper<Sum<Addr, <MemArg as ResolveMemArg>::Offset>>,
            <WasmMemory<Pages, MaxPages, Cells> as $helper<
                Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
            >>::Output:
                $outcome<Module, WasmMemory<Pages, MaxPages, Cells>, Tables, Globals, Tail, Locals, Frames, Branches, Rest>,
        {
            type Output = <<WasmMemory<Pages, MaxPages, Cells> as $helper<
                Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
            >>::Output as $outcome<
                Module,
                WasmMemory<Pages, MaxPages, Cells>,
                Tables,
                Globals,
                Tail,
                Locals,
                Frames,
                Branches,
                Rest,
            >>::Output;
        }
    };
}

impl_checked_load_i32!(
    OpI32Load,
    MemoryReadI32Checked,
    CheckedLoadI32Outcome,
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<typenum::U3>
);
impl_checked_load_i32!(OpI32Load8U, MemoryReadByteChecked, CheckedLoadI32Outcome);
impl_checked_load_i32!(OpI32Load8S, MemoryReadByteChecked, CheckedLoadI32SignExtend8Outcome);
impl_checked_load_i32!(
    OpI32Load16U,
    MemoryReadI32Low16Checked,
    CheckedLoadI32Outcome,
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<typenum::U1>
);
impl_checked_load_i32!(
    OpI32Load16S,
    MemoryReadI32Low16Checked,
    CheckedLoadI32SignExtend16Outcome,
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<typenum::U1>
);

impl_checked_load_i64!(
    OpI64Load,
    MemoryReadI64Checked,
    CheckedLoadI64Outcome,
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<typenum::U7>
);
impl_checked_load_i64!(OpI64Load8S, MemoryReadByteChecked, CheckedLoadI64SignExtend8Outcome);
impl_checked_load_i64!(OpI64Load8U, MemoryReadByteChecked, CheckedLoadI64ZeroExtend8Outcome);
impl_checked_load_i64!(
    OpI64Load16U,
    MemoryReadI64Low16Checked,
    CheckedLoadI64Outcome,
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<typenum::U1>
);
impl_checked_load_i64!(
    OpI64Load16S,
    MemoryReadI64Low16Checked,
    CheckedLoadI64SignExtend16Outcome,
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<typenum::U1>
);
impl_checked_load_i64!(
    OpI64Load32U,
    MemoryReadI64Low32Checked,
    CheckedLoadI64Outcome,
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<typenum::U3>
);
impl_checked_load_i64!(
    OpI64Load32S,
    MemoryReadI64Low32Checked,
    CheckedLoadI64SignExtend32Outcome,
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<typenum::U3>
);

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
    for CheckedStep<
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
    WasmMemory<Pages, MaxPages, Cells>: MemoryWriteByteChecked<
            Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
            <<ValueT as EncodeI32>::Output as LowByte>::Output,
        >,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteByteChecked<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
        <<ValueT as EncodeI32>::Output as LowByte>::Output,
    >>::Output: CheckedStoreOutcome<Module, Tables, Globals, Tail, Locals, Frames, Branches, Rest>,
{
    type Output = <<WasmMemory<Pages, MaxPages, Cells> as MemoryWriteByteChecked<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
        <<ValueT as EncodeI32>::Output as LowByte>::Output,
    >>::Output as CheckedStoreOutcome<
        Module,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
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
    for CheckedStep<
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
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<typenum::U1>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryWriteI32Low16Checked<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, ValueT>,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteI32Low16Checked<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
        ValueT,
    >>::Output: CheckedStoreOutcome<Module, Tables, Globals, Tail, Locals, Frames, Branches, Rest>,
{
    type Output = <<WasmMemory<Pages, MaxPages, Cells> as MemoryWriteI32Low16Checked<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
        ValueT,
    >>::Output as CheckedStoreOutcome<
        Module,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
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
    for CheckedStep<
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
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<typenum::U3>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryWriteI32Checked<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, ValueT>,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteI32Checked<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
        ValueT,
    >>::Output: CheckedStoreOutcome<Module, Tables, Globals, Tail, Locals, Frames, Branches, Rest>,
{
    type Output = <<WasmMemory<Pages, MaxPages, Cells> as MemoryWriteI32Checked<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
        ValueT,
    >>::Output as CheckedStoreOutcome<
        Module,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
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
    for CheckedStep<
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
    ValueT: EncodeI64,
    <ValueT as EncodeI64>::Output: LowByte,
    WasmMemory<Pages, MaxPages, Cells>: MemoryWriteByteChecked<
            Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
            <<ValueT as EncodeI64>::Output as LowByte>::Output,
        >,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteByteChecked<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
        <<ValueT as EncodeI64>::Output as LowByte>::Output,
    >>::Output: CheckedStoreOutcome<Module, Tables, Globals, Tail, Locals, Frames, Branches, Rest>,
{
    type Output = <<WasmMemory<Pages, MaxPages, Cells> as MemoryWriteByteChecked<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
        <<ValueT as EncodeI64>::Output as LowByte>::Output,
    >>::Output as CheckedStoreOutcome<
        Module,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
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
    for CheckedStep<
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
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<typenum::U1>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryWriteI64Low16Checked<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, ValueT>,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteI64Low16Checked<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
        ValueT,
    >>::Output: CheckedStoreOutcome<Module, Tables, Globals, Tail, Locals, Frames, Branches, Rest>,
{
    type Output = <<WasmMemory<Pages, MaxPages, Cells> as MemoryWriteI64Low16Checked<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
        ValueT,
    >>::Output as CheckedStoreOutcome<
        Module,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
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
    for CheckedStep<
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
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<typenum::U3>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryWriteI64Low32Checked<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, ValueT>,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteI64Low32Checked<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
        ValueT,
    >>::Output: CheckedStoreOutcome<Module, Tables, Globals, Tail, Locals, Frames, Branches, Rest>,
{
    type Output = <<WasmMemory<Pages, MaxPages, Cells> as MemoryWriteI64Low32Checked<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
        ValueT,
    >>::Output as CheckedStoreOutcome<
        Module,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
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
    for CheckedStep<
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
    Sum<Addr, <MemArg as ResolveMemArg>::Offset>: Add<typenum::U7>,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryWriteI64Checked<Sum<Addr, <MemArg as ResolveMemArg>::Offset>, ValueT>,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteI64Checked<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
        ValueT,
    >>::Output: CheckedStoreOutcome<Module, Tables, Globals, Tail, Locals, Frames, Branches, Rest>,
{
    type Output = <<WasmMemory<Pages, MaxPages, Cells> as MemoryWriteI64Checked<
        Sum<Addr, <MemArg as ResolveMemArg>::Offset>,
        ValueT,
    >>::Output as CheckedStoreOutcome<
        Module,
        Tables,
        Globals,
        Tail,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
}
