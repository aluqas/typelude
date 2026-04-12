use core::{
    marker::PhantomData,
    ops::{Add, BitAnd, BitOr, Mul, Shl, Shr},
};

use typelude_col::{TArr, TTerm};
use typelude_std::core::{Eq, Gt, Lt, Value};
use typenum::{
    B0, B1, IsGreater, U0, U1, U2, U3, U4, U5, U6, U7, U8, U16, U24, U32, U40, U48, U56, U65536,
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

pub trait SecondByte {
    type Output;
}

impl<B0V, B1V, B2V, B3V> SecondByte for EncodedI32<B0V, B1V, B2V, B3V> {
    type Output = B1V;
}

pub trait ThirdByte {
    type Output;
}

impl<B0V, B1V, B2V, B3V> ThirdByte for EncodedI32<B0V, B1V, B2V, B3V> {
    type Output = B2V;
}

pub trait FourthByte {
    type Output;
}

impl<B0V, B1V, B2V, B3V> FourthByte for EncodedI32<B0V, B1V, B2V, B3V> {
    type Output = B3V;
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

impl<B0V, B1V, B2V, B3V, B4V, B5V, B6V, B7V> LowByte
    for EncodedI64<B0V, B1V, B2V, B3V, B4V, B5V, B6V, B7V>
{
    type Output = B0V;
}

impl<B0V, B1V, B2V, B3V, B4V, B5V, B6V, B7V> SecondByte
    for EncodedI64<B0V, B1V, B2V, B3V, B4V, B5V, B6V, B7V>
{
    type Output = B1V;
}

impl<B0V, B1V, B2V, B3V, B4V, B5V, B6V, B7V> ThirdByte
    for EncodedI64<B0V, B1V, B2V, B3V, B4V, B5V, B6V, B7V>
{
    type Output = B2V;
}

impl<B0V, B1V, B2V, B3V, B4V, B5V, B6V, B7V> FourthByte
    for EncodedI64<B0V, B1V, B2V, B3V, B4V, B5V, B6V, B7V>
{
    type Output = B3V;
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

pub trait DecodeI32From2<B1V> {
    type Output;
}

impl<B0V, B1V> DecodeI32From2<B1V> for B0V
where
    B1V: Shl<U8>,
    B0V: BitOr<Shleft<B1V, U8>>,
    Or<B0V, Shleft<B1V, U8>>: BitAnd<U4294967295>,
{
    type Output = And<Or<B0V, Shleft<B1V, U8>>, U4294967295>;
}

pub trait DecodeI64From1 {
    type Output;
}

impl<B0V> DecodeI64From1 for B0V
where
    B0V: BitAnd<U18446744073709551615>,
{
    type Output = And<B0V, U18446744073709551615>;
}

pub trait DecodeI64From2<B1V> {
    type Output;
}

impl<B0V, B1V> DecodeI64From2<B1V> for B0V
where
    B1V: Shl<U8>,
    B0V: BitOr<Shleft<B1V, U8>>,
    Or<B0V, Shleft<B1V, U8>>: BitAnd<U18446744073709551615>,
{
    type Output = And<Or<B0V, Shleft<B1V, U8>>, U18446744073709551615>;
}

pub trait DecodeI64From4<B1V, B2V, B3V> {
    type Output;
}

impl<B0V, B1V, B2V, B3V> DecodeI64From4<B1V, B2V, B3V> for B0V
where
    B1V: Shl<U8>,
    B2V: Shl<U16>,
    B3V: Shl<U24>,
    B0V: BitOr<Shleft<B1V, U8>>,
    Or<B0V, Shleft<B1V, U8>>: BitOr<Shleft<B2V, U16>>,
    Or<Or<B0V, Shleft<B1V, U8>>, Shleft<B2V, U16>>: BitOr<Shleft<B3V, U24>>,
    Or<Or<Or<B0V, Shleft<B1V, U8>>, Shleft<B2V, U16>>, Shleft<B3V, U24>>:
        BitAnd<U18446744073709551615>,
{
    type Output = And<
        Or<Or<Or<B0V, Shleft<B1V, U8>>, Shleft<B2V, U16>>, Shleft<B3V, U24>>,
        U18446744073709551615,
    >;
}

pub trait SignExtend8ToI32 {
    type Output;
}

impl<ValueT> SignExtend8ToI32 for ValueT
where
    ValueT: crate::helpers::i32::I32Extend8S,
{
    type Output = <ValueT as crate::helpers::i32::I32Extend8S>::Output;
}

pub trait SignExtend16ToI32 {
    type Output;
}

impl<ValueT> SignExtend16ToI32 for ValueT
where
    ValueT: crate::helpers::i32::I32Extend16S,
{
    type Output = <ValueT as crate::helpers::i32::I32Extend16S>::Output;
}

pub trait SignExtend8ToI64 {
    type Output;
}

pub trait MaskByte {
    type Output;
}

impl<ValueT> MaskByte for ValueT
where
    ValueT: BitAnd<U255>,
{
    type Output = And<ValueT, U255>;
}

pub trait SignBitByte {
    type Output;
}

impl<ValueT> SignBitByte for ValueT
where
    ValueT: MaskByte,
    <ValueT as MaskByte>::Output: BitAnd<crate::helpers::i32::U128>,
{
    type Output = And<<ValueT as MaskByte>::Output, crate::helpers::i32::U128>;
}

pub trait SignExtend8ToI64Helper<LowBits> {
    type Output;
}

impl<LowBits> SignExtend8ToI64Helper<LowBits> for B1 {
    type Output = LowBits;
}

impl<LowBits> SignExtend8ToI64Helper<LowBits> for B0
where
    LowBits: BitOr<typenum::operator_aliases::Xor<U18446744073709551615, U255>>,
{
    type Output = Or<LowBits, typenum::operator_aliases::Xor<U18446744073709551615, U255>>;
}

impl<ValueT> SignExtend8ToI64 for ValueT
where
    ValueT: MaskByte + SignBitByte,
    <ValueT as SignBitByte>::Output: Eq<U0>,
    <<ValueT as SignBitByte>::Output as Eq<U0>>::Output:
        SignExtend8ToI64Helper<<ValueT as MaskByte>::Output>,
{
    type Output =
        <<<ValueT as SignBitByte>::Output as Eq<U0>>::Output as SignExtend8ToI64Helper<
            <ValueT as MaskByte>::Output,
        >>::Output;
}

pub trait SignExtend16ToI64 {
    type Output;
}

pub trait MaskWord {
    type Output;
}

impl<ValueT> MaskWord for ValueT
where
    ValueT: BitAnd<crate::helpers::i32::U65535>,
{
    type Output = And<ValueT, crate::helpers::i32::U65535>;
}

pub trait SignBitWord {
    type Output;
}

impl<ValueT> SignBitWord for ValueT
where
    ValueT: MaskWord,
    <ValueT as MaskWord>::Output: BitAnd<crate::helpers::i32::U32768>,
{
    type Output = And<<ValueT as MaskWord>::Output, crate::helpers::i32::U32768>;
}

pub trait SignExtend16ToI64Helper<LowBits> {
    type Output;
}

impl<LowBits> SignExtend16ToI64Helper<LowBits> for B1 {
    type Output = LowBits;
}

impl<LowBits> SignExtend16ToI64Helper<LowBits> for B0
where
    LowBits:
        BitOr<typenum::operator_aliases::Xor<U18446744073709551615, crate::helpers::i32::U65535>>,
{
    type Output = Or<
        LowBits,
        typenum::operator_aliases::Xor<U18446744073709551615, crate::helpers::i32::U65535>,
    >;
}

impl<ValueT> SignExtend16ToI64 for ValueT
where
    ValueT: MaskWord + SignBitWord,
    <ValueT as SignBitWord>::Output: Eq<U0>,
    <<ValueT as SignBitWord>::Output as Eq<U0>>::Output:
        SignExtend16ToI64Helper<<ValueT as MaskWord>::Output>,
{
    type Output =
        <<<ValueT as SignBitWord>::Output as Eq<U0>>::Output as SignExtend16ToI64Helper<
            <ValueT as MaskWord>::Output,
        >>::Output;
}

pub trait SignExtend32ToI64 {
    type Output;
}

impl<ValueT> SignExtend32ToI64 for ValueT
where
    ValueT: crate::helpers::i64::I64ExtendI32S,
{
    type Output = <ValueT as crate::helpers::i64::I64ExtendI32S>::Output;
}

pub trait MemoryWriteI32Low16<Addr, ValueT> {
    type Output;
}

impl<Pages, MaxPages, Cells, Addr, ValueT> MemoryWriteI32Low16<Addr, ValueT>
    for WasmMemory<Pages, MaxPages, Cells>
where
    Addr: Add<U1>,
    ValueT: EncodeI32,
    <ValueT as EncodeI32>::Output: LowByte + SecondByte,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryWriteByte<Addr, <<ValueT as EncodeI32>::Output as LowByte>::Output>,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteByte<
        Addr,
        <<ValueT as EncodeI32>::Output as LowByte>::Output,
    >>::Output:
        MemoryWriteByte<Sum<Addr, U1>, <<ValueT as EncodeI32>::Output as SecondByte>::Output>,
{
    type Output = <<WasmMemory<Pages, MaxPages, Cells> as MemoryWriteByte<
        Addr,
        <<ValueT as EncodeI32>::Output as LowByte>::Output,
    >>::Output as MemoryWriteByte<
        Sum<Addr, U1>,
        <<ValueT as EncodeI32>::Output as SecondByte>::Output,
    >>::Output;
}

pub trait MemoryWriteI64Low8<Addr, ValueT> {
    type Output;
}

impl<Pages, MaxPages, Cells, Addr, ValueT> MemoryWriteI64Low8<Addr, ValueT>
    for WasmMemory<Pages, MaxPages, Cells>
where
    ValueT: EncodeI64,
    <ValueT as EncodeI64>::Output: LowByte,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryWriteByte<Addr, <<ValueT as EncodeI64>::Output as LowByte>::Output>,
{
    type Output = <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteByte<
        Addr,
        <<ValueT as EncodeI64>::Output as LowByte>::Output,
    >>::Output;
}

pub trait MemoryWriteI64Low16<Addr, ValueT> {
    type Output;
}

impl<Pages, MaxPages, Cells, Addr, ValueT> MemoryWriteI64Low16<Addr, ValueT>
    for WasmMemory<Pages, MaxPages, Cells>
where
    Addr: Add<U1>,
    ValueT: EncodeI64,
    <ValueT as EncodeI64>::Output: LowByte + SecondByte,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryWriteByte<Addr, <<ValueT as EncodeI64>::Output as LowByte>::Output>,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteByte<
        Addr,
        <<ValueT as EncodeI64>::Output as LowByte>::Output,
    >>::Output:
        MemoryWriteByte<Sum<Addr, U1>, <<ValueT as EncodeI64>::Output as SecondByte>::Output>,
{
    type Output = <<WasmMemory<Pages, MaxPages, Cells> as MemoryWriteByte<
        Addr,
        <<ValueT as EncodeI64>::Output as LowByte>::Output,
    >>::Output as MemoryWriteByte<
        Sum<Addr, U1>,
        <<ValueT as EncodeI64>::Output as SecondByte>::Output,
    >>::Output;
}

pub trait MemoryWriteI64Low32<Addr, ValueT> {
    type Output;
}

impl<Pages, MaxPages, Cells, Addr, ValueT> MemoryWriteI64Low32<Addr, ValueT>
    for WasmMemory<Pages, MaxPages, Cells>
where
    Addr: Add<U1> + Add<U2> + Add<U3>,
    ValueT: EncodeI64,
    <ValueT as EncodeI64>::Output: LowByte + SecondByte + ThirdByte + FourthByte,
    WasmMemory<Pages, MaxPages, Cells>:
        MemoryWriteByte<Addr, <<ValueT as EncodeI64>::Output as LowByte>::Output>,
    <WasmMemory<Pages, MaxPages, Cells> as MemoryWriteByte<
        Addr,
        <<ValueT as EncodeI64>::Output as LowByte>::Output,
    >>::Output:
        MemoryWriteByte<Sum<Addr, U1>, <<ValueT as EncodeI64>::Output as SecondByte>::Output>,
    <<WasmMemory<Pages, MaxPages, Cells> as MemoryWriteByte<
        Addr,
        <<ValueT as EncodeI64>::Output as LowByte>::Output,
    >>::Output as MemoryWriteByte<
        Sum<Addr, U1>,
        <<ValueT as EncodeI64>::Output as SecondByte>::Output,
    >>::Output:
        MemoryWriteByte<Sum<Addr, U2>, <<ValueT as EncodeI64>::Output as ThirdByte>::Output>,
    <<<WasmMemory<Pages, MaxPages, Cells> as MemoryWriteByte<
        Addr,
        <<ValueT as EncodeI64>::Output as LowByte>::Output,
    >>::Output as MemoryWriteByte<
        Sum<Addr, U1>,
        <<ValueT as EncodeI64>::Output as SecondByte>::Output,
    >>::Output as MemoryWriteByte<
        Sum<Addr, U2>,
        <<ValueT as EncodeI64>::Output as ThirdByte>::Output,
    >>::Output:
        MemoryWriteByte<Sum<Addr, U3>, <<ValueT as EncodeI64>::Output as FourthByte>::Output>,
{
    type Output = <<<<WasmMemory<Pages, MaxPages, Cells> as MemoryWriteByte<
        Addr,
        <<ValueT as EncodeI64>::Output as LowByte>::Output,
    >>::Output as MemoryWriteByte<
        Sum<Addr, U1>,
        <<ValueT as EncodeI64>::Output as SecondByte>::Output,
    >>::Output as MemoryWriteByte<
        Sum<Addr, U2>,
        <<ValueT as EncodeI64>::Output as ThirdByte>::Output,
    >>::Output as MemoryWriteByte<
        Sum<Addr, U3>,
        <<ValueT as EncodeI64>::Output as FourthByte>::Output,
    >>::Output;
}
