use core::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};

use typenum::{
    B0, B1, Const, IsEqual, IsGreater, IsLess, ToUInt, U0, Unsigned,
    operator_aliases::{And, Diff, Mod, Or, Prod, Quot, Shleft, Shright, Sum, Xor},
};

pub type U31 = <Const<31> as ToUInt>::Output;
pub type U32 = <Const<32> as ToUInt>::Output;
pub type U128 = <Const<128> as ToUInt>::Output;
pub type U127 = <Const<127> as ToUInt>::Output;
pub type U255 = <Const<255> as ToUInt>::Output;
pub type U32767 = <Const<32767> as ToUInt>::Output;
pub type U65535 = <Const<65535> as ToUInt>::Output;
pub type U32768 = <Const<32768> as ToUInt>::Output;
pub type U2147483647 = <Const<2147483647> as ToUInt>::Output;
pub type U2147483648 = <Const<2147483648> as ToUInt>::Output;
pub type U4294967295 = <Const<4294967295> as ToUInt>::Output;
pub type U4294967296 = <Const<4294967296> as ToUInt>::Output;

pub trait TrueBit {}

impl TrueBit for B1 {}

pub trait FalseBit {}

impl FalseBit for B0 {}

pub trait BoolToI32 {
    type Output;
}

impl BoolToI32 for B0 {
    type Output = U0;
}

impl BoolToI32 for B1 {
    type Output = typenum::U1;
}

pub trait BoolNot {
    type Output;
}

impl BoolNot for B0 {
    type Output = B1;
}

impl BoolNot for B1 {
    type Output = B0;
}

pub trait WrapI32 {
    type Output;
}

impl<ValueT> WrapI32 for ValueT
where
    ValueT: Unsigned,
    ValueT: BitAnd<U4294967295>,
{
    type Output = And<ValueT, U4294967295>;
}

pub trait I32Add<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32Add<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Lhs: Add<Rhs>,
    Sum<Lhs, Rhs>: WrapI32,
{
    type Output = <Sum<Lhs, Rhs> as WrapI32>::Output;
}

pub trait I32Neg {
    type Output;
}

pub trait I32NegHelper<ValueT> {
    type Output;
}

impl<ValueT> I32NegHelper<ValueT> for B1 {
    type Output = U0;
}

impl<ValueT> I32NegHelper<ValueT> for B0
where
    U4294967296: Sub<ValueT>,
{
    type Output = Diff<U4294967296, ValueT>;
}

impl<ValueT> I32Neg for ValueT
where
    ValueT: Unsigned,
    ValueT: IsEqual<U0>,
    <ValueT as IsEqual<U0>>::Output: I32NegHelper<ValueT>,
{
    type Output = <<ValueT as IsEqual<U0>>::Output as I32NegHelper<ValueT>>::Output;
}

pub trait I32Sub<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32Sub<Rhs> for Lhs
where
    Rhs: I32Neg,
    Lhs: I32Add<<Rhs as I32Neg>::Output>,
{
    type Output = <Lhs as I32Add<<Rhs as I32Neg>::Output>>::Output;
}

pub trait I32Mul<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32Mul<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Lhs: Mul<Rhs>,
    Prod<Lhs, Rhs>: WrapI32,
{
    type Output = <Prod<Lhs, Rhs> as WrapI32>::Output;
}

pub trait I32UnsignedLt<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32UnsignedLt<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Lhs: IsLess<Rhs>,
{
    type Output = <Lhs as IsLess<Rhs>>::Output;
}

pub trait I32UnsignedGt<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32UnsignedGt<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Lhs: IsGreater<Rhs>,
{
    type Output = <Lhs as IsGreater<Rhs>>::Output;
}

pub trait I32IsNegative {
    type Output;
}

pub trait I32SignBitIsZero {
    type Output;
}

pub trait I32SignBitIsZeroHelper<SignBits> {
    type Output;
}

impl<ValueT, SignBits> I32SignBitIsZeroHelper<SignBits> for ValueT
where
    SignBits: IsEqual<U0>,
{
    type Output = <SignBits as IsEqual<U0>>::Output;
}

impl<ValueT, SignBits> I32SignBitIsZero for ValueT
where
    ValueT: Unsigned + BitAnd<U2147483648, Output = SignBits>,
    ValueT: I32SignBitIsZeroHelper<SignBits>,
{
    type Output = <ValueT as I32SignBitIsZeroHelper<SignBits>>::Output;
}

pub trait I32NegativeFromLt {
    type Output;
}

impl I32NegativeFromLt for B1 {
    type Output = B0;
}

impl I32NegativeFromLt for B0 {
    type Output = B1;
}

impl<ValueT> I32IsNegative for ValueT
where
    ValueT: Unsigned,
    ValueT: I32SignBitIsZero,
    <ValueT as I32SignBitIsZero>::Output: I32NegativeFromLt,
{
    type Output = <<ValueT as I32SignBitIsZero>::Output as I32NegativeFromLt>::Output;
}

pub trait I32Magnitude {
    type Output;
}

pub trait I32MagnitudeHelper<ValueT> {
    type Output;
}

impl<ValueT> I32MagnitudeHelper<ValueT> for B0 {
    type Output = ValueT;
}

impl<ValueT> I32MagnitudeHelper<ValueT> for B1
where
    U4294967296: Sub<ValueT>,
{
    type Output = Diff<U4294967296, ValueT>;
}

impl<ValueT> I32Magnitude for ValueT
where
    ValueT: Unsigned,
    ValueT: I32IsNegative,
    <ValueT as I32IsNegative>::Output: I32MagnitudeHelper<ValueT>,
{
    type Output = <<ValueT as I32IsNegative>::Output as I32MagnitudeHelper<ValueT>>::Output;
}

pub trait I32SignedLt<Rhs> {
    type Output;
}

pub trait I32SignedLtHelper<Rhs, LhsSign, RhsSign> {
    type Output;
}

impl<Lhs, Rhs> I32SignedLtHelper<Rhs, B0, B1> for Lhs {
    type Output = B0;
}

impl<Lhs, Rhs> I32SignedLtHelper<Rhs, B1, B0> for Lhs {
    type Output = B1;
}

impl<Lhs, Rhs> I32SignedLtHelper<Rhs, B0, B0> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Lhs: IsLess<Rhs>,
{
    type Output = <Lhs as IsLess<Rhs>>::Output;
}

impl<Lhs, Rhs> I32SignedLtHelper<Rhs, B1, B1> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Lhs: IsLess<Rhs>,
{
    type Output = <Lhs as IsLess<Rhs>>::Output;
}

impl<Lhs, Rhs> I32SignedLt<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Lhs: I32IsNegative,
    Rhs: I32IsNegative,
    Lhs: I32SignedLtHelper<Rhs, <Lhs as I32IsNegative>::Output, <Rhs as I32IsNegative>::Output>,
{
    type Output = <Lhs as I32SignedLtHelper<
        Rhs,
        <Lhs as I32IsNegative>::Output,
        <Rhs as I32IsNegative>::Output,
    >>::Output;
}

pub trait I32EqValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32EqValue<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Lhs: IsEqual<Rhs>,
{
    type Output = <Lhs as IsEqual<Rhs>>::Output;
}

pub trait I32BoolOr<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32BoolOr<Rhs> for Lhs
where
    Lhs: BitOr<Rhs>,
{
    type Output = Or<Lhs, Rhs>;
}

pub trait I32BoolAnd<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32BoolAnd<Rhs> for Lhs
where
    Lhs: BitAnd<Rhs>,
{
    type Output = And<Lhs, Rhs>;
}

pub trait I32SignedGt<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32SignedGt<Rhs> for Lhs
where
    Rhs: I32SignedLt<Lhs>,
{
    type Output = <Rhs as I32SignedLt<Lhs>>::Output;
}

pub trait I32SignedLe<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32SignedLe<Rhs> for Lhs
where
    Lhs: I32SignedGt<Rhs>,
    <Lhs as I32SignedGt<Rhs>>::Output: BoolNot,
{
    type Output = <<Lhs as I32SignedGt<Rhs>>::Output as BoolNot>::Output;
}

pub trait I32SignedGe<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32SignedGe<Rhs> for Lhs
where
    Lhs: I32SignedLt<Rhs>,
    <Lhs as I32SignedLt<Rhs>>::Output: BoolNot,
{
    type Output = <<Lhs as I32SignedLt<Rhs>>::Output as BoolNot>::Output;
}

pub trait I32UnsignedLe<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32UnsignedLe<Rhs> for Lhs
where
    Lhs: I32UnsignedGt<Rhs>,
    <Lhs as I32UnsignedGt<Rhs>>::Output: BoolNot,
{
    type Output = <<Lhs as I32UnsignedGt<Rhs>>::Output as BoolNot>::Output;
}

pub trait I32UnsignedGe<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32UnsignedGe<Rhs> for Lhs
where
    Lhs: I32UnsignedLt<Rhs>,
    <Lhs as I32UnsignedLt<Rhs>>::Output: BoolNot,
{
    type Output = <<Lhs as I32UnsignedLt<Rhs>>::Output as BoolNot>::Output;
}

pub trait I32DivU<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32DivU<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Lhs: Div<Rhs>,
{
    type Output = Quot<Lhs, Rhs>;
}

pub trait I32RemU<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32RemU<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Lhs: Rem<Rhs>,
{
    type Output = Mod<Lhs, Rhs>;
}

pub trait I32SignedMagToBits<SignBit> {
    type Output;
}

pub trait I32PositiveMagToBits {
    type Output;
}

pub trait I32PositiveMagToBitsHelper<MagIsZero> {
    type Output;
}

impl<Mag> I32PositiveMagToBitsHelper<B1> for Mag {
    type Output = U0;
}

impl<Mag> I32PositiveMagToBitsHelper<B0> for Mag
where
    Mag: Unsigned,
    Mag: I32SignBitIsZero,
    <Mag as I32SignBitIsZero>::Output: TrueBit,
{
    type Output = Mag;
}

impl<Mag> I32PositiveMagToBits for Mag
where
    Mag: Unsigned,
    Mag: IsEqual<U0>,
    Mag: I32PositiveMagToBitsHelper<<Mag as IsEqual<U0>>::Output>,
{
    type Output = <Mag as I32PositiveMagToBitsHelper<<Mag as IsEqual<U0>>::Output>>::Output;
}

pub trait I32NegativeMagToBits {
    type Output;
}

pub trait I32NegativeMagToBitsHelper<MagIsZero> {
    type Output;
}

impl<Mag> I32NegativeMagToBitsHelper<B1> for Mag {
    type Output = U0;
}

impl<Mag> I32NegativeMagToBitsHelper<B0> for Mag
where
    U4294967296: Sub<Mag>,
{
    type Output = Diff<U4294967296, Mag>;
}

impl<Mag> I32NegativeMagToBits for Mag
where
    Mag: Unsigned,
    Mag: IsEqual<U0>,
    Mag: I32NegativeMagToBitsHelper<<Mag as IsEqual<U0>>::Output>,
{
    type Output = <Mag as I32NegativeMagToBitsHelper<<Mag as IsEqual<U0>>::Output>>::Output;
}

impl<Mag> I32SignedMagToBits<B0> for Mag
where
    Mag: I32PositiveMagToBits,
{
    type Output = <Mag as I32PositiveMagToBits>::Output;
}

impl<Mag> I32SignedMagToBits<B1> for Mag
where
    Mag: I32NegativeMagToBits,
{
    type Output = <Mag as I32NegativeMagToBits>::Output;
}

pub trait I32DivS<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32DivS<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Lhs: I32Magnitude + I32IsNegative,
    Rhs: I32Magnitude + I32IsNegative,
    <Lhs as I32Magnitude>::Output: Div<<Rhs as I32Magnitude>::Output>,
    <Lhs as I32IsNegative>::Output: BitXor<<Rhs as I32IsNegative>::Output>,
    Quot<<Lhs as I32Magnitude>::Output, <Rhs as I32Magnitude>::Output>:
        I32SignedMagToBits<Xor<<Lhs as I32IsNegative>::Output, <Rhs as I32IsNegative>::Output>>,
{
    type Output = <Quot<<Lhs as I32Magnitude>::Output, <Rhs as I32Magnitude>::Output> as I32SignedMagToBits<
        Xor<<Lhs as I32IsNegative>::Output, <Rhs as I32IsNegative>::Output>,
    >>::Output;
}

pub trait I32RemS<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32RemS<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Lhs: I32Magnitude + I32IsNegative,
    Rhs: I32Magnitude,
    <Lhs as I32Magnitude>::Output: Rem<<Rhs as I32Magnitude>::Output>,
    Mod<<Lhs as I32Magnitude>::Output, <Rhs as I32Magnitude>::Output>:
        I32SignedMagToBits<<Lhs as I32IsNegative>::Output>,
{
    type Output = <Mod<<Lhs as I32Magnitude>::Output, <Rhs as I32Magnitude>::Output> as I32SignedMagToBits<
        <Lhs as I32IsNegative>::Output,
    >>::Output;
}

pub trait I32AndValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32AndValue<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Lhs: BitAnd<Rhs>,
{
    type Output = And<Lhs, Rhs>;
}

pub trait I32OrValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32OrValue<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Lhs: BitOr<Rhs>,
{
    type Output = Or<Lhs, Rhs>;
}

pub trait I32XorValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32XorValue<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Lhs: BitXor<Rhs>,
{
    type Output = Xor<Lhs, Rhs>;
}

pub trait ShiftAmount {
    type Output;
}

impl<ValueT> ShiftAmount for ValueT
where
    ValueT: Unsigned,
    ValueT: BitAnd<U31>,
{
    type Output = And<ValueT, U31>;
}

pub trait I32ShlValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32ShlValue<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Rhs: ShiftAmount,
    Lhs: Shl<<Rhs as ShiftAmount>::Output>,
    Shleft<Lhs, <Rhs as ShiftAmount>::Output>: WrapI32,
{
    type Output = <Shleft<Lhs, <Rhs as ShiftAmount>::Output> as WrapI32>::Output;
}

pub trait I32ShrUValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32ShrUValue<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Rhs: ShiftAmount,
    Lhs: Shr<<Rhs as ShiftAmount>::Output>,
{
    type Output = Shright<Lhs, <Rhs as ShiftAmount>::Output>;
}

pub trait I32ShrSFill {
    type Output;
}

impl I32ShrSFill for U0 {
    type Output = U0;
}

impl I32ShrSFill for typenum::U1 {
    type Output = Shleft<typenum::U1, U31>;
}

macro_rules! shr_s_fill_impls {
    ($($amount:ty => ($prev:ty, $shift:ty)),* $(,)?) => {
        $(
            impl I32ShrSFill for $amount {
                type Output = Or<
                    <$prev as I32ShrSFill>::Output,
                    Shleft<typenum::U1, $shift>
                >;
            }
        )*
    };
}

shr_s_fill_impls!(
    typenum::U2 => (typenum::U1, typenum::U30),
    typenum::U3 => (typenum::U2, typenum::U29),
    typenum::U4 => (typenum::U3, typenum::U28),
    typenum::U5 => (typenum::U4, typenum::U27),
    typenum::U6 => (typenum::U5, typenum::U26),
    typenum::U7 => (typenum::U6, typenum::U25),
    typenum::U8 => (typenum::U7, typenum::U24),
    typenum::U9 => (typenum::U8, typenum::U23),
    typenum::U10 => (typenum::U9, typenum::U22),
    typenum::U11 => (typenum::U10, typenum::U21),
    typenum::U12 => (typenum::U11, typenum::U20),
    typenum::U13 => (typenum::U12, typenum::U19),
    typenum::U14 => (typenum::U13, typenum::U18),
    typenum::U15 => (typenum::U14, typenum::U17),
    typenum::U16 => (typenum::U15, typenum::U16),
    typenum::U17 => (typenum::U16, typenum::U15),
    typenum::U18 => (typenum::U17, typenum::U14),
    typenum::U19 => (typenum::U18, typenum::U13),
    typenum::U20 => (typenum::U19, typenum::U12),
    typenum::U21 => (typenum::U20, typenum::U11),
    typenum::U22 => (typenum::U21, typenum::U10),
    typenum::U23 => (typenum::U22, typenum::U9),
    typenum::U24 => (typenum::U23, typenum::U8),
    typenum::U25 => (typenum::U24, typenum::U7),
    typenum::U26 => (typenum::U25, typenum::U6),
    typenum::U27 => (typenum::U26, typenum::U5),
    typenum::U28 => (typenum::U27, typenum::U4),
    typenum::U29 => (typenum::U28, typenum::U3),
    typenum::U30 => (typenum::U29, typenum::U2),
    typenum::U31 => (typenum::U30, typenum::U1),
);

pub trait I32ShrSSelect<Amount, Shifted> {
    type Output;
}

impl<Amount, Shifted> I32ShrSSelect<Amount, Shifted> for B0 {
    type Output = Shifted;
}

impl<Amount, Shifted> I32ShrSSelect<Amount, Shifted> for B1
where
    Amount: I32ShrSFill,
    Shifted: BitOr<<Amount as I32ShrSFill>::Output>,
{
    type Output = Or<Shifted, <Amount as I32ShrSFill>::Output>;
}

pub trait I32ShrSValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32ShrSValue<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned,
    Rhs: ShiftAmount,
    Lhs: I32IsNegative + Shr<<Rhs as ShiftAmount>::Output>,
    <Lhs as I32IsNegative>::Output:
        I32ShrSSelect<<Rhs as ShiftAmount>::Output, Shright<Lhs, <Rhs as ShiftAmount>::Output>>,
{
    type Output = <<Lhs as I32IsNegative>::Output as I32ShrSSelect<
        <Rhs as ShiftAmount>::Output,
        Shright<Lhs, <Rhs as ShiftAmount>::Output>,
    >>::Output;
}

pub trait I32Clz {
    type Output;
}

pub trait I32ClzZeroHelper<ValueT, SignZero> {
    type Output;
}

impl<ValueT, SignZero> I32ClzZeroHelper<ValueT, SignZero> for B1 {
    type Output = U32;
}

impl<ValueT> I32ClzZeroHelper<ValueT, B0> for B0 {
    type Output = U0;
}

impl<ValueT> I32ClzZeroHelper<ValueT, B1> for B0
where
    ValueT: I32ShlValue<typenum::U1>,
    <ValueT as I32ShlValue<typenum::U1>>::Output: I32Clz,
    <<ValueT as I32ShlValue<typenum::U1>>::Output as I32Clz>::Output: Add<typenum::U1>,
{
    type Output =
        Sum<<<ValueT as I32ShlValue<typenum::U1>>::Output as I32Clz>::Output, typenum::U1>;
}

impl<ValueT> I32Clz for ValueT
where
    ValueT: Unsigned + IsEqual<U0> + I32SignBitIsZero,
    <ValueT as IsEqual<U0>>::Output:
        I32ClzZeroHelper<ValueT, <ValueT as I32SignBitIsZero>::Output>,
{
    type Output = <<ValueT as IsEqual<U0>>::Output as I32ClzZeroHelper<
        ValueT,
        <ValueT as I32SignBitIsZero>::Output,
    >>::Output;
}

pub trait I32Ctz {
    type Output;
}

pub trait I32CtzZeroHelper<ValueT, LowBit> {
    type Output;
}

impl<ValueT, LowBit> I32CtzZeroHelper<ValueT, LowBit> for B1 {
    type Output = U32;
}

impl<ValueT> I32CtzZeroHelper<ValueT, typenum::U1> for B0 {
    type Output = U0;
}

impl<ValueT> I32CtzZeroHelper<ValueT, U0> for B0
where
    ValueT: Shr<typenum::U1>,
    Shright<ValueT, typenum::U1>: I32Ctz,
    <Shright<ValueT, typenum::U1> as I32Ctz>::Output: Add<typenum::U1>,
{
    type Output = Sum<<Shright<ValueT, typenum::U1> as I32Ctz>::Output, typenum::U1>;
}

impl<ValueT> I32Ctz for ValueT
where
    ValueT: Unsigned + IsEqual<U0> + BitAnd<typenum::U1>,
    <ValueT as IsEqual<U0>>::Output: I32CtzZeroHelper<ValueT, And<ValueT, typenum::U1>>,
{
    type Output = <<ValueT as IsEqual<U0>>::Output as I32CtzZeroHelper<
        ValueT,
        And<ValueT, typenum::U1>,
    >>::Output;
}

pub trait I32Popcnt {
    type Output;
}

pub trait I32PopcntZeroHelper<ValueT> {
    type Output;
}

impl<ValueT> I32PopcntZeroHelper<ValueT> for B1 {
    type Output = U0;
}

impl<ValueT> I32PopcntZeroHelper<ValueT> for B0
where
    ValueT: Shr<typenum::U1> + BitAnd<typenum::U1>,
    Shright<ValueT, typenum::U1>: I32Popcnt,
    <Shright<ValueT, typenum::U1> as I32Popcnt>::Output: Add<And<ValueT, typenum::U1>>,
{
    type Output =
        Sum<<Shright<ValueT, typenum::U1> as I32Popcnt>::Output, And<ValueT, typenum::U1>>;
}

impl<ValueT> I32Popcnt for ValueT
where
    ValueT: Unsigned + IsEqual<U0>,
    <ValueT as IsEqual<U0>>::Output: I32PopcntZeroHelper<ValueT>,
{
    type Output = <<ValueT as IsEqual<U0>>::Output as I32PopcntZeroHelper<ValueT>>::Output;
}

pub trait I32InvRotAmount<Rhs> {
    type Output;
}

pub trait I32InvRotAmountValue {
    type Output;
}

impl I32InvRotAmountValue for U0 {
    type Output = U0;
}

macro_rules! inv_rot_amount_impls_32 {
    ($($amount:ty => $inv:ty),* $(,)?) => {
        $(
            impl I32InvRotAmountValue for $amount {
                type Output = $inv;
            }
        )*
    };
}

inv_rot_amount_impls_32!(
    typenum::U1 => typenum::U31,
    typenum::U2 => typenum::U30,
    typenum::U3 => typenum::U29,
    typenum::U4 => typenum::U28,
    typenum::U5 => typenum::U27,
    typenum::U6 => typenum::U26,
    typenum::U7 => typenum::U25,
    typenum::U8 => typenum::U24,
    typenum::U9 => typenum::U23,
    typenum::U10 => typenum::U22,
    typenum::U11 => typenum::U21,
    typenum::U12 => typenum::U20,
    typenum::U13 => typenum::U19,
    typenum::U14 => typenum::U18,
    typenum::U15 => typenum::U17,
    typenum::U16 => typenum::U16,
    typenum::U17 => typenum::U15,
    typenum::U18 => typenum::U14,
    typenum::U19 => typenum::U13,
    typenum::U20 => typenum::U12,
    typenum::U21 => typenum::U11,
    typenum::U22 => typenum::U10,
    typenum::U23 => typenum::U9,
    typenum::U24 => typenum::U8,
    typenum::U25 => typenum::U7,
    typenum::U26 => typenum::U6,
    typenum::U27 => typenum::U5,
    typenum::U28 => typenum::U4,
    typenum::U29 => typenum::U3,
    typenum::U30 => typenum::U2,
    typenum::U31 => typenum::U1,
);

impl<Lhs, Rhs> I32InvRotAmount<Rhs> for Lhs
where
    Rhs: ShiftAmount,
    <Rhs as ShiftAmount>::Output: I32InvRotAmountValue,
{
    type Output = <<Rhs as ShiftAmount>::Output as I32InvRotAmountValue>::Output;
}

pub trait I32RotlValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32RotlValue<Rhs> for Lhs
where
    Lhs: I32ShlValue<Rhs>
        + I32InvRotAmount<Rhs>
        + I32ShrUValue<<Lhs as I32InvRotAmount<Rhs>>::Output>,
    <Lhs as I32ShlValue<Rhs>>::Output:
        BitOr<<Lhs as I32ShrUValue<<Lhs as I32InvRotAmount<Rhs>>::Output>>::Output>,
{
    type Output = Or<
        <Lhs as I32ShlValue<Rhs>>::Output,
        <Lhs as I32ShrUValue<<Lhs as I32InvRotAmount<Rhs>>::Output>>::Output,
    >;
}

pub trait I32RotrValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I32RotrValue<Rhs> for Lhs
where
    Lhs: I32ShrUValue<Rhs>
        + I32InvRotAmount<Rhs>
        + I32ShlValue<<Lhs as I32InvRotAmount<Rhs>>::Output>,
    <Lhs as I32ShrUValue<Rhs>>::Output:
        BitOr<<Lhs as I32ShlValue<<Lhs as I32InvRotAmount<Rhs>>::Output>>::Output>,
{
    type Output = Or<
        <Lhs as I32ShrUValue<Rhs>>::Output,
        <Lhs as I32ShlValue<<Lhs as I32InvRotAmount<Rhs>>::Output>>::Output,
    >;
}

pub trait I32Extend8S {
    type Output;
}

pub trait Mask8 {
    type Output;
}

impl<ValueT> Mask8 for ValueT
where
    ValueT: BitAnd<U255>,
{
    type Output = And<ValueT, U255>;
}

pub trait SignBit8 {
    type Output;
}

impl<ValueT> SignBit8 for ValueT
where
    ValueT: Mask8,
    <ValueT as Mask8>::Output: BitAnd<U128>,
{
    type Output = And<<ValueT as Mask8>::Output, U128>;
}

pub trait I32Extend8SHelper<LowBits> {
    type Output;
}

impl<LowBits> I32Extend8SHelper<LowBits> for B1 {
    type Output = LowBits;
}

impl<LowBits> I32Extend8SHelper<LowBits> for B0
where
    LowBits: BitOr<Xor<U4294967295, U255>>,
{
    type Output = Or<LowBits, Xor<U4294967295, U255>>;
}

impl<ValueT> I32Extend8S for ValueT
where
    ValueT: Unsigned + Mask8 + SignBit8,
    <ValueT as SignBit8>::Output: IsEqual<U0>,
    <<ValueT as SignBit8>::Output as IsEqual<U0>>::Output:
        I32Extend8SHelper<<ValueT as Mask8>::Output>,
{
    type Output = <<<ValueT as SignBit8>::Output as IsEqual<U0>>::Output as I32Extend8SHelper<
        <ValueT as Mask8>::Output,
    >>::Output;
}

pub trait I32Extend16S {
    type Output;
}

pub trait Mask16 {
    type Output;
}

impl<ValueT> Mask16 for ValueT
where
    ValueT: BitAnd<U65535>,
{
    type Output = And<ValueT, U65535>;
}

pub trait SignBit16 {
    type Output;
}

impl<ValueT> SignBit16 for ValueT
where
    ValueT: Mask16,
    <ValueT as Mask16>::Output: BitAnd<U32768>,
{
    type Output = And<<ValueT as Mask16>::Output, U32768>;
}

pub trait I32Extend16SHelper<LowBits> {
    type Output;
}

impl<LowBits> I32Extend16SHelper<LowBits> for B1 {
    type Output = LowBits;
}

impl<LowBits> I32Extend16SHelper<LowBits> for B0
where
    LowBits: BitOr<Xor<U4294967295, U65535>>,
{
    type Output = Or<LowBits, Xor<U4294967295, U65535>>;
}

impl<ValueT> I32Extend16S for ValueT
where
    ValueT: Unsigned + Mask16 + SignBit16,
    <ValueT as SignBit16>::Output: IsEqual<U0>,
    <<ValueT as SignBit16>::Output as IsEqual<U0>>::Output:
        I32Extend16SHelper<<ValueT as Mask16>::Output>,
{
    type Output = <<<ValueT as SignBit16>::Output as IsEqual<U0>>::Output as I32Extend16SHelper<
        <ValueT as Mask16>::Output,
    >>::Output;
}
