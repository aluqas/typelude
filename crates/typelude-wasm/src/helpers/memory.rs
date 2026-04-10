use core::{
    marker::PhantomData,
    ops::{Add, BitAnd, BitOr, Mul, Shl, Shr},
};

use typelude_col::{TArr, TTerm};
use typelude_std::core::{Eq, Gt, Lt, Value};
use typenum::{
    B0, B1, U1, U2, U3, U8, U16, U24, U65536,
    operator_aliases::{And, Or, Prod, Shleft, Shright, Sum},
};

use crate::{
    helpers::i32::{TrueBit, U255, U4294967295},
    state::{MemoryCell, WasmMemory},
};

pub trait FindByte<Addr> {
    type Output;
}

impl<Addr> FindByte<Addr> for TTerm {
    type Output = typenum::U0;
}

pub trait FindByteHelper<Addr, Byte, Tail> {
    type Output;
}

impl<Addr, Byte, Tail> FindByteHelper<Addr, Byte, Tail> for B1 {
    type Output = Byte;
}

impl<Addr, Byte, Tail> FindByteHelper<Addr, Byte, Tail> for B0
where
    Tail: FindByte<Addr>,
{
    type Output = <Tail as FindByte<Addr>>::Output;
}

impl<QueryAddr, CellAddr, Byte, Tail> FindByte<QueryAddr> for TArr<MemoryCell<CellAddr, Byte>, Tail>
where
    QueryAddr: Eq<CellAddr>,
    <QueryAddr as Eq<CellAddr>>::Output: FindByteHelper<QueryAddr, Byte, Tail>,
{
    type Output =
        <<QueryAddr as Eq<CellAddr>>::Output as FindByteHelper<QueryAddr, Byte, Tail>>::Output;
}

pub trait MemoryReadByte<Addr> {
    type Output;
}

impl<Pages, MaxPages, Cells, Addr> MemoryReadByte<Addr> for WasmMemory<Pages, MaxPages, Cells>
where
    Pages: Mul<U65536>,
    Addr: Lt<Prod<Pages, U65536>>,
    <Addr as Lt<Prod<Pages, U65536>>>::Output: TrueBit,
    Cells: FindByte<Addr>,
{
    type Output = <Cells as FindByte<Addr>>::Output;
}

pub trait MemoryWriteByte<Addr, Byte> {
    type Output;
}

impl<Pages, MaxPages, Cells, Addr, Byte> MemoryWriteByte<Addr, Byte> for WasmMemory<Pages, MaxPages, Cells>
where
    Pages: Mul<U65536>,
    Addr: Lt<Prod<Pages, U65536>>,
    <Addr as Lt<Prod<Pages, U65536>>>::Output: TrueBit,
{
    type Output = WasmMemory<Pages, MaxPages, TArr<MemoryCell<Addr, Byte>, Cells>>;
}

pub trait MemoryWriteBytes<Addr, Bytes> {
    type Output;
}

impl<Memory, Addr> MemoryWriteBytes<Addr, TTerm> for Memory {
    type Output = Memory;
}

impl<Memory, Addr, Byte, Tail> MemoryWriteBytes<Addr, TArr<Byte, Tail>> for Memory
where
    Memory: MemoryWriteByte<Addr, Byte>,
    Addr: Add<U1>,
    <Memory as MemoryWriteByte<Addr, Byte>>::Output: MemoryWriteBytes<Sum<Addr, U1>, Tail>,
{
    type Output =
        <<Memory as MemoryWriteByte<Addr, Byte>>::Output as MemoryWriteBytes<Sum<Addr, U1>, Tail>>::Output;
}

#[doc(hidden)]
pub struct EncodedI32<B0V, B1V, B2V, B3V>(pub PhantomData<(B0V, B1V, B2V, B3V)>);

impl<B0V, B1V, B2V, B3V> Value for EncodedI32<B0V, B1V, B2V, B3V> {}

pub trait EncodeI32 {
    type Output;
}

impl<ValueT> EncodeI32 for ValueT
where
    ValueT: BitAnd<U255> + Shr<U8> + Shr<U16> + Shr<U24>,
    Shright<ValueT, U8>: BitAnd<U255>,
    Shright<ValueT, U16>: BitAnd<U255>,
    Shright<ValueT, U24>: BitAnd<U255>,
{
    type Output = EncodedI32<
        And<ValueT, U255>,
        And<Shright<ValueT, U8>, U255>,
        And<Shright<ValueT, U16>, U255>,
        And<Shright<ValueT, U24>, U255>,
    >;
}

pub trait LowByte {
    type Output;
}

impl<B0V, B1V, B2V, B3V> LowByte for EncodedI32<B0V, B1V, B2V, B3V> {
    type Output = B0V;
}

pub trait DecodeI32 {
    type Output;
}

impl<B0V, B1V, B2V, B3V> DecodeI32 for EncodedI32<B0V, B1V, B2V, B3V>
where
    B1V: Shl<U8>,
    B2V: Shl<U16>,
    B3V: Shl<U24>,
    B0V: BitOr<Shleft<B1V, U8>>,
    Shleft<B2V, U16>: BitOr<Shleft<B3V, U24>>,
    Or<B0V, Shleft<B1V, U8>>: BitOr<Or<Shleft<B2V, U16>, Shleft<B3V, U24>>>,
    Or<Or<B0V, Shleft<B1V, U8>>, Or<Shleft<B2V, U16>, Shleft<B3V, U24>>>: BitAnd<U4294967295>,
{
    type Output = And<
        Or<Or<B0V, Shleft<B1V, U8>>, Or<Shleft<B2V, U16>, Shleft<B3V, U24>>>,
        U4294967295,
    >;
}

pub trait MemoryReadI32<Addr> {
    type Output;
}

impl<Pages, MaxPages, Cells, Addr> MemoryReadI32<Addr> for WasmMemory<Pages, MaxPages, Cells>
where
    Addr: Add<U1> + Add<U2> + Add<U3>,
    WasmMemory<Pages, MaxPages, Cells>: MemoryReadByte<Addr>,
    WasmMemory<Pages, MaxPages, Cells>: MemoryReadByte<Sum<Addr, U1>>,
    WasmMemory<Pages, MaxPages, Cells>: MemoryReadByte<Sum<Addr, U2>>,
    WasmMemory<Pages, MaxPages, Cells>: MemoryReadByte<Sum<Addr, U3>>,
    EncodedI32<
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Addr>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U1>>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U2>>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U3>>>::Output,
    >: DecodeI32,
{
    type Output = <EncodedI32<
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Addr>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U1>>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U2>>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U3>>>::Output,
    > as DecodeI32>::Output;
}

pub trait MemoryWriteI32<Addr, ValueT> {
    type Output;
}

impl<Pages, MaxPages, Cells, Addr, ValueT> MemoryWriteI32<Addr, ValueT> for WasmMemory<Pages, MaxPages, Cells>
where
    Addr: Add<U1> + Add<U2> + Add<U3>,
    ValueT: EncodeI32,
    <ValueT as EncodeI32>::Output: WriteEncodedI32<WasmMemory<Pages, MaxPages, Cells>, Addr>,
{
    type Output = <<ValueT as EncodeI32>::Output as WriteEncodedI32<
        WasmMemory<Pages, MaxPages, Cells>,
        Addr,
    >>::Output;
}

pub trait WriteEncodedI32<Memory, Addr> {
    type Output;
}

impl<Memory, Addr, B0V, B1V, B2V, B3V> WriteEncodedI32<Memory, Addr> for EncodedI32<B0V, B1V, B2V, B3V>
where
    Addr: Add<U1> + Add<U2> + Add<U3>,
    Memory: MemoryWriteByte<Addr, B0V>,
    <Memory as MemoryWriteByte<Addr, B0V>>::Output: MemoryWriteByte<Sum<Addr, U1>, B1V>,
    <<Memory as MemoryWriteByte<Addr, B0V>>::Output as MemoryWriteByte<Sum<Addr, U1>, B1V>>::Output:
        MemoryWriteByte<Sum<Addr, U2>, B2V>,
    <<<Memory as MemoryWriteByte<Addr, B0V>>::Output as MemoryWriteByte<Sum<Addr, U1>, B1V>>::Output as MemoryWriteByte<Sum<Addr, U2>, B2V>>::Output:
        MemoryWriteByte<Sum<Addr, U3>, B3V>,
{
    type Output = <<<<Memory as MemoryWriteByte<Addr, B0V>>::Output as MemoryWriteByte<
        Sum<Addr, U1>,
        B1V,
    >>::Output as MemoryWriteByte<Sum<Addr, U2>, B2V>>::Output as MemoryWriteByte<
        Sum<Addr, U3>,
        B3V,
    >>::Output;
}

pub trait MemoryGrow<Delta> {
    type OutputMemory;
    type Result;
}

pub trait MemoryGrowWithinMax<Pages, MaxPages, Cells, NewPages> {
    type OutputMemory;
    type Result;
}

impl<Pages, MaxPages, Cells, NewPages> MemoryGrowWithinMax<Pages, MaxPages, Cells, NewPages> for B1 {
    type OutputMemory = WasmMemory<Pages, MaxPages, Cells>;
    type Result = U4294967295;
}

impl<Pages, MaxPages, Cells, NewPages> MemoryGrowWithinMax<Pages, MaxPages, Cells, NewPages> for B0 {
    type OutputMemory = WasmMemory<NewPages, MaxPages, Cells>;
    type Result = Pages;
}

impl<Pages, MaxPages, Cells, Delta> MemoryGrow<Delta> for WasmMemory<Pages, MaxPages, Cells>
where
    Pages: Add<Delta>,
    Sum<Pages, Delta>: Gt<MaxPages>,
    <Sum<Pages, Delta> as Gt<MaxPages>>::Output: MemoryGrowWithinMax<Pages, MaxPages, Cells, Sum<Pages, Delta>>,
{
    type OutputMemory =
        <<Sum<Pages, Delta> as Gt<MaxPages>>::Output as MemoryGrowWithinMax<
            Pages,
            MaxPages,
            Cells,
            Sum<Pages, Delta>,
        >>::OutputMemory;
    type Result = <<Sum<Pages, Delta> as Gt<MaxPages>>::Output as MemoryGrowWithinMax<
        Pages,
        MaxPages,
        Cells,
        Sum<Pages, Delta>,
    >>::Result;
}
