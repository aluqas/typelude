use core::{
    marker::PhantomData,
    ops::{Add, BitAnd, BitOr, Mul, Shl, Shr},
};

use typelude_col::{TArr, TTerm};
use typelude_std::core::{Eq, Gt, Lt, Value};
use typenum::{
    B0, B1, U1, U2, U3, U4, U5, U6, U7, U8, U16, U24, U32, U40, U48, U56, U65536,
    operator_aliases::{And, Or, Prod, Shleft, Shright, Sum},
};

use crate::{
    helpers::{
        i32::{TrueBit, U255, U4294967295},
        i64::U18446744073709551615,
    },
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

impl<QueryAddr, CellAddr, Byte, Tail> FindByte<QueryAddr>
    for TArr<MemoryCell<CellAddr, Byte>, Tail>
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

impl<Pages, MaxPages, Cells, Addr, Byte> MemoryWriteByte<Addr, Byte>
    for WasmMemory<Pages, MaxPages, Cells>
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
    type Output = <<Memory as MemoryWriteByte<Addr, Byte>>::Output as MemoryWriteBytes<
        Sum<Addr, U1>,
        Tail,
    >>::Output;
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
    type Output =
        And<Or<Or<B0V, Shleft<B1V, U8>>, Or<Shleft<B2V, U16>, Shleft<B3V, U24>>>, U4294967295>;
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

impl<Pages, MaxPages, Cells, Addr, ValueT> MemoryWriteI32<Addr, ValueT>
    for WasmMemory<Pages, MaxPages, Cells>
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

#[doc(hidden)]
pub struct EncodedI64<B0V, B1V, B2V, B3V, B4V, B5V, B6V, B7V>(
    pub PhantomData<(B0V, B1V, B2V, B3V, B4V, B5V, B6V, B7V)>,
);

impl<B0V, B1V, B2V, B3V, B4V, B5V, B6V, B7V> Value
    for EncodedI64<B0V, B1V, B2V, B3V, B4V, B5V, B6V, B7V>
{
}

pub trait EncodeI64 {
    type Output;
}

impl<ValueT> EncodeI64 for ValueT
where
    ValueT:
        BitAnd<U255> + Shr<U8> + Shr<U16> + Shr<U24> + Shr<U32> + Shr<U40> + Shr<U48> + Shr<U56>,
    Shright<ValueT, U8>: BitAnd<U255>,
    Shright<ValueT, U16>: BitAnd<U255>,
    Shright<ValueT, U24>: BitAnd<U255>,
    Shright<ValueT, U32>: BitAnd<U255>,
    Shright<ValueT, U40>: BitAnd<U255>,
    Shright<ValueT, U48>: BitAnd<U255>,
    Shright<ValueT, U56>: BitAnd<U255>,
{
    type Output = EncodedI64<
        And<ValueT, U255>,
        And<Shright<ValueT, U8>, U255>,
        And<Shright<ValueT, U16>, U255>,
        And<Shright<ValueT, U24>, U255>,
        And<Shright<ValueT, U32>, U255>,
        And<Shright<ValueT, U40>, U255>,
        And<Shright<ValueT, U48>, U255>,
        And<Shright<ValueT, U56>, U255>,
    >;
}

pub trait DecodeI64 {
    type Output;
}

impl<B0V, B1V, B2V, B3V, B4V, B5V, B6V, B7V> DecodeI64
    for EncodedI64<B0V, B1V, B2V, B3V, B4V, B5V, B6V, B7V>
where
    B1V: Shl<U8>,
    B2V: Shl<U16>,
    B3V: Shl<U24>,
    B4V: Shl<U32>,
    B5V: Shl<U40>,
    B6V: Shl<U48>,
    B7V: Shl<U56>,
    B0V: BitOr<Shleft<B1V, U8>>,
    Or<B0V, Shleft<B1V, U8>>: BitOr<Shleft<B2V, U16>>,
    Or<Or<B0V, Shleft<B1V, U8>>, Shleft<B2V, U16>>: BitOr<Shleft<B3V, U24>>,
    Or<Or<Or<B0V, Shleft<B1V, U8>>, Shleft<B2V, U16>>, Shleft<B3V, U24>>: BitOr<Shleft<B4V, U32>>,
    Or<Or<Or<Or<B0V, Shleft<B1V, U8>>, Shleft<B2V, U16>>, Shleft<B3V, U24>>, Shleft<B4V, U32>>:
        BitOr<Shleft<B5V, U40>>,
    Or<
        Or<Or<Or<Or<B0V, Shleft<B1V, U8>>, Shleft<B2V, U16>>, Shleft<B3V, U24>>, Shleft<B4V, U32>>,
        Shleft<B5V, U40>,
    >: BitOr<Shleft<B6V, U48>>,
    Or<
        Or<
            Or<
                Or<Or<Or<B0V, Shleft<B1V, U8>>, Shleft<B2V, U16>>, Shleft<B3V, U24>>,
                Shleft<B4V, U32>,
            >,
            Shleft<B5V, U40>,
        >,
        Shleft<B6V, U48>,
    >: BitOr<Shleft<B7V, U56>>,
    Or<
        Or<
            Or<
                Or<
                    Or<Or<Or<B0V, Shleft<B1V, U8>>, Shleft<B2V, U16>>, Shleft<B3V, U24>>,
                    Shleft<B4V, U32>,
                >,
                Shleft<B5V, U40>,
            >,
            Shleft<B6V, U48>,
        >,
        Shleft<B7V, U56>,
    >: BitAnd<U18446744073709551615>,
{
    type Output = And<
        Or<
            Or<
                Or<
                    Or<
                        Or<Or<Or<B0V, Shleft<B1V, U8>>, Shleft<B2V, U16>>, Shleft<B3V, U24>>,
                        Shleft<B4V, U32>,
                    >,
                    Shleft<B5V, U40>,
                >,
                Shleft<B6V, U48>,
            >,
            Shleft<B7V, U56>,
        >,
        U18446744073709551615,
    >;
}

pub trait MemoryReadI64<Addr> {
    type Output;
}

impl<Pages, MaxPages, Cells, Addr> MemoryReadI64<Addr> for WasmMemory<Pages, MaxPages, Cells>
where
    Addr: Add<U1> + Add<U2> + Add<U3> + Add<U4> + Add<U5> + Add<U6> + Add<U7>,
    WasmMemory<Pages, MaxPages, Cells>: MemoryReadByte<Addr>,
    WasmMemory<Pages, MaxPages, Cells>: MemoryReadByte<Sum<Addr, U1>>,
    WasmMemory<Pages, MaxPages, Cells>: MemoryReadByte<Sum<Addr, U2>>,
    WasmMemory<Pages, MaxPages, Cells>: MemoryReadByte<Sum<Addr, U3>>,
    WasmMemory<Pages, MaxPages, Cells>: MemoryReadByte<Sum<Addr, U4>>,
    WasmMemory<Pages, MaxPages, Cells>: MemoryReadByte<Sum<Addr, U5>>,
    WasmMemory<Pages, MaxPages, Cells>: MemoryReadByte<Sum<Addr, U6>>,
    WasmMemory<Pages, MaxPages, Cells>: MemoryReadByte<Sum<Addr, U7>>,
    EncodedI64<
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Addr>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U1>>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U2>>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U3>>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U4>>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U5>>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U6>>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U7>>>::Output,
    >: DecodeI64,
{
    type Output = <EncodedI64<
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Addr>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U1>>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U2>>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U3>>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U4>>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U5>>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U6>>>::Output,
        <WasmMemory<Pages, MaxPages, Cells> as MemoryReadByte<Sum<Addr, U7>>>::Output,
    > as DecodeI64>::Output;
}

pub trait MemoryWriteI64<Addr, ValueT> {
    type Output;
}

impl<Pages, MaxPages, Cells, Addr, ValueT> MemoryWriteI64<Addr, ValueT>
    for WasmMemory<Pages, MaxPages, Cells>
where
    Addr: Add<U1> + Add<U2> + Add<U3> + Add<U4> + Add<U5> + Add<U6> + Add<U7>,
    ValueT: EncodeI64,
    <ValueT as EncodeI64>::Output: WriteEncodedI64<WasmMemory<Pages, MaxPages, Cells>, Addr>,
{
    type Output = <<ValueT as EncodeI64>::Output as WriteEncodedI64<
        WasmMemory<Pages, MaxPages, Cells>,
        Addr,
    >>::Output;
}

pub trait WriteEncodedI64<Memory, Addr> {
    type Output;
}

impl<Memory, Addr, B0V, B1V, B2V, B3V, B4V, B5V, B6V, B7V> WriteEncodedI64<Memory, Addr>
    for EncodedI64<B0V, B1V, B2V, B3V, B4V, B5V, B6V, B7V>
where
    Addr: Add<U1> + Add<U2> + Add<U3> + Add<U4> + Add<U5> + Add<U6> + Add<U7>,
    Memory: MemoryWriteByte<Addr, B0V>,
    <Memory as MemoryWriteByte<Addr, B0V>>::Output: MemoryWriteByte<Sum<Addr, U1>, B1V>,
    <<Memory as MemoryWriteByte<Addr, B0V>>::Output as MemoryWriteByte<Sum<Addr, U1>, B1V>>::Output:
        MemoryWriteByte<Sum<Addr, U2>, B2V>,
    <<<Memory as MemoryWriteByte<Addr, B0V>>::Output as MemoryWriteByte<Sum<Addr, U1>, B1V>>::Output as MemoryWriteByte<Sum<Addr, U2>, B2V>>::Output:
        MemoryWriteByte<Sum<Addr, U3>, B3V>,
    <<<<Memory as MemoryWriteByte<Addr, B0V>>::Output as MemoryWriteByte<Sum<Addr, U1>, B1V>>::Output as MemoryWriteByte<Sum<Addr, U2>, B2V>>::Output as MemoryWriteByte<Sum<Addr, U3>, B3V>>::Output:
        MemoryWriteByte<Sum<Addr, U4>, B4V>,
    <<<<<Memory as MemoryWriteByte<Addr, B0V>>::Output as MemoryWriteByte<Sum<Addr, U1>, B1V>>::Output as MemoryWriteByte<Sum<Addr, U2>, B2V>>::Output as MemoryWriteByte<Sum<Addr, U3>, B3V>>::Output as MemoryWriteByte<Sum<Addr, U4>, B4V>>::Output:
        MemoryWriteByte<Sum<Addr, U5>, B5V>,
    <<<<<<Memory as MemoryWriteByte<Addr, B0V>>::Output as MemoryWriteByte<Sum<Addr, U1>, B1V>>::Output as MemoryWriteByte<Sum<Addr, U2>, B2V>>::Output as MemoryWriteByte<Sum<Addr, U3>, B3V>>::Output as MemoryWriteByte<Sum<Addr, U4>, B4V>>::Output as MemoryWriteByte<Sum<Addr, U5>, B5V>>::Output:
        MemoryWriteByte<Sum<Addr, U6>, B6V>,
    <<<<<<<Memory as MemoryWriteByte<Addr, B0V>>::Output as MemoryWriteByte<Sum<Addr, U1>, B1V>>::Output as MemoryWriteByte<Sum<Addr, U2>, B2V>>::Output as MemoryWriteByte<Sum<Addr, U3>, B3V>>::Output as MemoryWriteByte<Sum<Addr, U4>, B4V>>::Output as MemoryWriteByte<Sum<Addr, U5>, B5V>>::Output as MemoryWriteByte<Sum<Addr, U6>, B6V>>::Output:
        MemoryWriteByte<Sum<Addr, U7>, B7V>,
{
    type Output = <<<<<<<<Memory as MemoryWriteByte<Addr, B0V>>::Output as MemoryWriteByte<
        Sum<Addr, U1>,
        B1V,
    >>::Output as MemoryWriteByte<Sum<Addr, U2>, B2V>>::Output as MemoryWriteByte<
        Sum<Addr, U3>,
        B3V,
    >>::Output as MemoryWriteByte<Sum<Addr, U4>, B4V>>::Output as MemoryWriteByte<
        Sum<Addr, U5>,
        B5V,
    >>::Output as MemoryWriteByte<Sum<Addr, U6>, B6V>>::Output as MemoryWriteByte<
        Sum<Addr, U7>,
        B7V,
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

impl<Pages, MaxPages, Cells, NewPages> MemoryGrowWithinMax<Pages, MaxPages, Cells, NewPages>
    for B1
{
    type OutputMemory = WasmMemory<Pages, MaxPages, Cells>;
    type Result = U4294967295;
}

impl<Pages, MaxPages, Cells, NewPages> MemoryGrowWithinMax<Pages, MaxPages, Cells, NewPages>
    for B0
{
    type OutputMemory = WasmMemory<NewPages, MaxPages, Cells>;
    type Result = Pages;
}

impl<Pages, MaxPages, Cells, Delta> MemoryGrow<Delta> for WasmMemory<Pages, MaxPages, Cells>
where
    Pages: Add<Delta>,
    Sum<Pages, Delta>: Gt<MaxPages>,
    <Sum<Pages, Delta> as Gt<MaxPages>>::Output:
        MemoryGrowWithinMax<Pages, MaxPages, Cells, Sum<Pages, Delta>>,
{
    type OutputMemory = <<Sum<Pages, Delta> as Gt<MaxPages>>::Output as MemoryGrowWithinMax<
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
