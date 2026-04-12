use core::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr};

use typenum::{
    B0, B1, Const, IsEqual, IsGreater, IsLess, ToUInt, U0, U1, Unsigned,
    operator_aliases::{And, Mod, Or, Prod, Quot, Shleft, Shright, Sum, Xor},
};

use crate::helpers::i32::{BoolNot, TrueBit, U2147483648, U4294967295};

pub type U63 = <Const<63> as ToUInt>::Output;
pub type U64 = <Const<64> as ToUInt>::Output;
pub type U255 = <Const<255> as ToUInt>::Output;
pub type U9223372036854775807 = <Const<9223372036854775807> as ToUInt>::Output;
pub type U9223372036854775808 = <Const<9223372036854775808> as ToUInt>::Output;
pub type U18446744073709551615 = Or<U9223372036854775808, U9223372036854775807>;

pub trait WrapI64 {
    type Output;
}

impl<ValueT> WrapI64 for ValueT
where
    ValueT: Unsigned + BitAnd<U18446744073709551615>,
{
    type Output = And<ValueT, U18446744073709551615>;
}

pub trait I64Add<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64Add<Rhs> for Lhs
where
    Lhs: Unsigned + Add<Rhs>,
    Rhs: Unsigned,
    Sum<Lhs, Rhs>: WrapI64,
{
    type Output = <Sum<Lhs, Rhs> as WrapI64>::Output;
}

pub trait I64Not {
    type Output;
}

impl<ValueT> I64Not for ValueT
where
    ValueT: Unsigned + BitXor<U18446744073709551615>,
{
    type Output = Xor<ValueT, U18446744073709551615>;
}

pub trait I64Neg {
    type Output;
}

impl<ValueT> I64Neg for ValueT
where
    ValueT: I64Not,
    <ValueT as I64Not>::Output: Add<U1>,
    Sum<<ValueT as I64Not>::Output, U1>: WrapI64,
{
    type Output = <Sum<<ValueT as I64Not>::Output, U1> as WrapI64>::Output;
}

pub trait I64Sub<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64Sub<Rhs> for Lhs
where
    Rhs: I64Neg,
    Lhs: I64Add<<Rhs as I64Neg>::Output>,
{
    type Output = <Lhs as I64Add<<Rhs as I64Neg>::Output>>::Output;
}

pub trait I64Mul<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64Mul<Rhs> for Lhs
where
    Lhs: Unsigned + Mul<Rhs>,
    Rhs: Unsigned,
    Prod<Lhs, Rhs>: WrapI64,
{
    type Output = <Prod<Lhs, Rhs> as WrapI64>::Output;
}

pub trait I64UnsignedLt<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64UnsignedLt<Rhs> for Lhs
where
    Lhs: Unsigned + IsLess<Rhs>,
    Rhs: Unsigned,
{
    type Output = <Lhs as IsLess<Rhs>>::Output;
}

pub trait I64UnsignedGt<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64UnsignedGt<Rhs> for Lhs
where
    Lhs: Unsigned + IsGreater<Rhs>,
    Rhs: Unsigned,
{
    type Output = <Lhs as IsGreater<Rhs>>::Output;
}

pub trait I64EqValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64EqValue<Rhs> for Lhs
where
    Lhs: Unsigned + IsEqual<Rhs>,
    Rhs: Unsigned,
{
    type Output = <Lhs as IsEqual<Rhs>>::Output;
}

pub trait I64SignBitIsZero {
    type Output;
}

pub trait I64SignBitIsZeroHelper<SignBits> {
    type Output;
}

impl<ValueT, SignBits> I64SignBitIsZeroHelper<SignBits> for ValueT
where
    SignBits: IsEqual<U0>,
{
    type Output = <SignBits as IsEqual<U0>>::Output;
}

impl<ValueT, SignBits> I64SignBitIsZero for ValueT
where
    ValueT: Unsigned + BitAnd<U9223372036854775808, Output = SignBits>,
    ValueT: I64SignBitIsZeroHelper<SignBits>,
{
    type Output = <ValueT as I64SignBitIsZeroHelper<SignBits>>::Output;
}

pub trait I64NegativeFromLt {
    type Output;
}

impl I64NegativeFromLt for B1 {
    type Output = B0;
}

impl I64NegativeFromLt for B0 {
    type Output = B1;
}

pub trait I64IsNegative {
    type Output;
}

impl<ValueT> I64IsNegative for ValueT
where
    ValueT: Unsigned + I64SignBitIsZero,
    <ValueT as I64SignBitIsZero>::Output: I64NegativeFromLt,
{
    type Output = <<ValueT as I64SignBitIsZero>::Output as I64NegativeFromLt>::Output;
}

pub trait I64Magnitude {
    type Output;
}

pub trait I64MagnitudeHelper<ValueT> {
    type Output;
}

impl<ValueT> I64MagnitudeHelper<ValueT> for B0 {
    type Output = ValueT;
}

impl<ValueT> I64MagnitudeHelper<ValueT> for B1
where
    ValueT: I64Neg,
{
    type Output = <ValueT as I64Neg>::Output;
}

impl<ValueT> I64Magnitude for ValueT
where
    ValueT: Unsigned + I64IsNegative,
    <ValueT as I64IsNegative>::Output: I64MagnitudeHelper<ValueT>,
{
    type Output = <<ValueT as I64IsNegative>::Output as I64MagnitudeHelper<ValueT>>::Output;
}

pub trait I64SignedLt<Rhs> {
    type Output;
}

pub trait I64SignedLtHelper<Rhs, LhsSign, RhsSign> {
    type Output;
}

impl<Lhs, Rhs> I64SignedLtHelper<Rhs, B0, B1> for Lhs {
    type Output = B0;
}

impl<Lhs, Rhs> I64SignedLtHelper<Rhs, B1, B0> for Lhs {
    type Output = B1;
}

impl<Lhs, Rhs> I64SignedLtHelper<Rhs, B0, B0> for Lhs
where
    Lhs: Unsigned + IsLess<Rhs>,
    Rhs: Unsigned,
{
    type Output = <Lhs as IsLess<Rhs>>::Output;
}

impl<Lhs, Rhs> I64SignedLtHelper<Rhs, B1, B1> for Lhs
where
    Lhs: Unsigned + IsLess<Rhs>,
    Rhs: Unsigned,
{
    type Output = <Lhs as IsLess<Rhs>>::Output;
}

impl<Lhs, Rhs> I64SignedLt<Rhs> for Lhs
where
    Lhs: Unsigned + I64IsNegative,
    Rhs: Unsigned + I64IsNegative,
    Lhs: I64SignedLtHelper<Rhs, <Lhs as I64IsNegative>::Output, <Rhs as I64IsNegative>::Output>,
{
    type Output = <Lhs as I64SignedLtHelper<
        Rhs,
        <Lhs as I64IsNegative>::Output,
        <Rhs as I64IsNegative>::Output,
    >>::Output;
}

pub trait I64SignedGt<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64SignedGt<Rhs> for Lhs
where
    Rhs: I64SignedLt<Lhs>,
{
    type Output = <Rhs as I64SignedLt<Lhs>>::Output;
}

pub trait I64SignedLe<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64SignedLe<Rhs> for Lhs
where
    Lhs: I64SignedGt<Rhs>,
    <Lhs as I64SignedGt<Rhs>>::Output: BoolNot,
{
    type Output = <<Lhs as I64SignedGt<Rhs>>::Output as BoolNot>::Output;
}

pub trait I64SignedGe<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64SignedGe<Rhs> for Lhs
where
    Lhs: I64SignedLt<Rhs>,
    <Lhs as I64SignedLt<Rhs>>::Output: BoolNot,
{
    type Output = <<Lhs as I64SignedLt<Rhs>>::Output as BoolNot>::Output;
}

pub trait I64UnsignedLe<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64UnsignedLe<Rhs> for Lhs
where
    Lhs: I64UnsignedGt<Rhs>,
    <Lhs as I64UnsignedGt<Rhs>>::Output: BoolNot,
{
    type Output = <<Lhs as I64UnsignedGt<Rhs>>::Output as BoolNot>::Output;
}

pub trait I64UnsignedGe<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64UnsignedGe<Rhs> for Lhs
where
    Lhs: I64UnsignedLt<Rhs>,
    <Lhs as I64UnsignedLt<Rhs>>::Output: BoolNot,
{
    type Output = <<Lhs as I64UnsignedLt<Rhs>>::Output as BoolNot>::Output;
}

pub trait I64DivU<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64DivU<Rhs> for Lhs
where
    Lhs: Unsigned + Div<Rhs>,
    Rhs: Unsigned,
{
    type Output = Quot<Lhs, Rhs>;
}

pub trait I64RemU<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64RemU<Rhs> for Lhs
where
    Lhs: Unsigned + Rem<Rhs>,
    Rhs: Unsigned,
{
    type Output = Mod<Lhs, Rhs>;
}

pub trait I64PositiveMagToBits {
    type Output;
}

pub trait I64PositiveMagToBitsHelper<MagIsZero> {
    type Output;
}

impl<Mag> I64PositiveMagToBitsHelper<B1> for Mag {
    type Output = U0;
}

impl<Mag> I64PositiveMagToBitsHelper<B0> for Mag
where
    Mag: Unsigned + I64SignBitIsZero,
    <Mag as I64SignBitIsZero>::Output: TrueBit,
{
    type Output = Mag;
}

impl<Mag> I64PositiveMagToBits for Mag
where
    Mag: Unsigned + IsEqual<U0>,
    Mag: I64PositiveMagToBitsHelper<<Mag as IsEqual<U0>>::Output>,
{
    type Output = <Mag as I64PositiveMagToBitsHelper<<Mag as IsEqual<U0>>::Output>>::Output;
}

pub trait I64NegativeMagToBits {
    type Output;
}

impl<Mag> I64NegativeMagToBits for Mag
where
    Mag: I64Neg,
{
    type Output = <Mag as I64Neg>::Output;
}

pub trait I64SignedMagToBits<SignBit> {
    type Output;
}

impl<Mag> I64SignedMagToBits<B0> for Mag
where
    Mag: I64PositiveMagToBits,
{
    type Output = <Mag as I64PositiveMagToBits>::Output;
}

impl<Mag> I64SignedMagToBits<B1> for Mag
where
    Mag: I64NegativeMagToBits,
{
    type Output = <Mag as I64NegativeMagToBits>::Output;
}

pub trait I64DivS<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64DivS<Rhs> for Lhs
where
    Lhs: Unsigned + I64Magnitude + I64IsNegative,
    Rhs: Unsigned + I64Magnitude + I64IsNegative,
    <Lhs as I64Magnitude>::Output: Div<<Rhs as I64Magnitude>::Output>,
    <Lhs as I64IsNegative>::Output: BitXor<<Rhs as I64IsNegative>::Output>,
    Quot<<Lhs as I64Magnitude>::Output, <Rhs as I64Magnitude>::Output>:
        I64SignedMagToBits<Xor<<Lhs as I64IsNegative>::Output, <Rhs as I64IsNegative>::Output>>,
{
    type Output = <Quot<<Lhs as I64Magnitude>::Output, <Rhs as I64Magnitude>::Output> as I64SignedMagToBits<
        Xor<<Lhs as I64IsNegative>::Output, <Rhs as I64IsNegative>::Output>,
    >>::Output;
}

pub trait I64RemS<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64RemS<Rhs> for Lhs
where
    Lhs: Unsigned + I64Magnitude + I64IsNegative,
    Rhs: Unsigned + I64Magnitude,
    <Lhs as I64Magnitude>::Output: Rem<<Rhs as I64Magnitude>::Output>,
    Mod<<Lhs as I64Magnitude>::Output, <Rhs as I64Magnitude>::Output>:
        I64SignedMagToBits<<Lhs as I64IsNegative>::Output>,
{
    type Output = <Mod<<Lhs as I64Magnitude>::Output, <Rhs as I64Magnitude>::Output> as I64SignedMagToBits<
        <Lhs as I64IsNegative>::Output,
    >>::Output;
}

pub trait I64AndValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64AndValue<Rhs> for Lhs
where
    Lhs: Unsigned + BitAnd<Rhs>,
    Rhs: Unsigned,
{
    type Output = And<Lhs, Rhs>;
}

pub trait I64OrValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64OrValue<Rhs> for Lhs
where
    Lhs: Unsigned + BitOr<Rhs>,
    Rhs: Unsigned,
{
    type Output = Or<Lhs, Rhs>;
}

pub trait I64XorValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64XorValue<Rhs> for Lhs
where
    Lhs: Unsigned + BitXor<Rhs>,
    Rhs: Unsigned,
{
    type Output = Xor<Lhs, Rhs>;
}

pub trait ShiftAmount {
    type Output;
}

impl<ValueT> ShiftAmount for ValueT
where
    ValueT: Unsigned + BitAnd<U63>,
{
    type Output = And<ValueT, U63>;
}

pub trait I64ShlValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64ShlValue<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned + ShiftAmount,
    Lhs: Shl<<Rhs as ShiftAmount>::Output>,
    Shleft<Lhs, <Rhs as ShiftAmount>::Output>: WrapI64,
{
    type Output = <Shleft<Lhs, <Rhs as ShiftAmount>::Output> as WrapI64>::Output;
}

pub trait I64ShrUValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64ShrUValue<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned + ShiftAmount,
    Lhs: Shr<<Rhs as ShiftAmount>::Output>,
{
    type Output = Shright<Lhs, <Rhs as ShiftAmount>::Output>;
}

pub trait I64ShrSFill {
    type Output;
}

impl I64ShrSFill for U0 {
    type Output = U0;
}

impl I64ShrSFill for U1 {
    type Output = Shleft<U1, U63>;
}

macro_rules! shr_s_fill_impls {
    ($($amount:ty => ($prev:ty, $shift:ty)),* $(,)?) => {
        $(
            impl I64ShrSFill for $amount {
                type Output = Or<
                    <$prev as I64ShrSFill>::Output,
                    Shleft<U1, $shift>
                >;
            }
        )*
    };
}

shr_s_fill_impls!(
    typenum::U2 => (typenum::U1, typenum::U62),
    typenum::U3 => (typenum::U2, typenum::U61),
    typenum::U4 => (typenum::U3, typenum::U60),
    typenum::U5 => (typenum::U4, typenum::U59),
    typenum::U6 => (typenum::U5, typenum::U58),
    typenum::U7 => (typenum::U6, typenum::U57),
    typenum::U8 => (typenum::U7, typenum::U56),
    typenum::U9 => (typenum::U8, typenum::U55),
    typenum::U10 => (typenum::U9, typenum::U54),
    typenum::U11 => (typenum::U10, typenum::U53),
    typenum::U12 => (typenum::U11, typenum::U52),
    typenum::U13 => (typenum::U12, typenum::U51),
    typenum::U14 => (typenum::U13, typenum::U50),
    typenum::U15 => (typenum::U14, typenum::U49),
    typenum::U16 => (typenum::U15, typenum::U48),
    typenum::U17 => (typenum::U16, typenum::U47),
    typenum::U18 => (typenum::U17, typenum::U46),
    typenum::U19 => (typenum::U18, typenum::U45),
    typenum::U20 => (typenum::U19, typenum::U44),
    typenum::U21 => (typenum::U20, typenum::U43),
    typenum::U22 => (typenum::U21, typenum::U42),
    typenum::U23 => (typenum::U22, typenum::U41),
    typenum::U24 => (typenum::U23, typenum::U40),
    typenum::U25 => (typenum::U24, typenum::U39),
    typenum::U26 => (typenum::U25, typenum::U38),
    typenum::U27 => (typenum::U26, typenum::U37),
    typenum::U28 => (typenum::U27, typenum::U36),
    typenum::U29 => (typenum::U28, typenum::U35),
    typenum::U30 => (typenum::U29, typenum::U34),
    typenum::U31 => (typenum::U30, typenum::U33),
    typenum::U32 => (typenum::U31, typenum::U32),
    typenum::U33 => (typenum::U32, typenum::U31),
    typenum::U34 => (typenum::U33, typenum::U30),
    typenum::U35 => (typenum::U34, typenum::U29),
    typenum::U36 => (typenum::U35, typenum::U28),
    typenum::U37 => (typenum::U36, typenum::U27),
    typenum::U38 => (typenum::U37, typenum::U26),
    typenum::U39 => (typenum::U38, typenum::U25),
    typenum::U40 => (typenum::U39, typenum::U24),
    typenum::U41 => (typenum::U40, typenum::U23),
    typenum::U42 => (typenum::U41, typenum::U22),
    typenum::U43 => (typenum::U42, typenum::U21),
    typenum::U44 => (typenum::U43, typenum::U20),
    typenum::U45 => (typenum::U44, typenum::U19),
    typenum::U46 => (typenum::U45, typenum::U18),
    typenum::U47 => (typenum::U46, typenum::U17),
    typenum::U48 => (typenum::U47, typenum::U16),
    typenum::U49 => (typenum::U48, typenum::U15),
    typenum::U50 => (typenum::U49, typenum::U14),
    typenum::U51 => (typenum::U50, typenum::U13),
    typenum::U52 => (typenum::U51, typenum::U12),
    typenum::U53 => (typenum::U52, typenum::U11),
    typenum::U54 => (typenum::U53, typenum::U10),
    typenum::U55 => (typenum::U54, typenum::U9),
    typenum::U56 => (typenum::U55, typenum::U8),
    typenum::U57 => (typenum::U56, typenum::U7),
    typenum::U58 => (typenum::U57, typenum::U6),
    typenum::U59 => (typenum::U58, typenum::U5),
    typenum::U60 => (typenum::U59, typenum::U4),
    typenum::U61 => (typenum::U60, typenum::U3),
    typenum::U62 => (typenum::U61, typenum::U2),
    typenum::U63 => (typenum::U62, typenum::U1),
);

pub trait I64ShrSSelect<Amount, Shifted> {
    type Output;
}

impl<Amount, Shifted> I64ShrSSelect<Amount, Shifted> for B0 {
    type Output = Shifted;
}

impl<Amount, Shifted> I64ShrSSelect<Amount, Shifted> for B1
where
    Amount: I64ShrSFill,
    Shifted: BitOr<<Amount as I64ShrSFill>::Output>,
{
    type Output = Or<Shifted, <Amount as I64ShrSFill>::Output>;
}

pub trait I64ShrSValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64ShrSValue<Rhs> for Lhs
where
    Lhs: Unsigned,
    Rhs: Unsigned + ShiftAmount,
    Lhs: I64IsNegative + Shr<<Rhs as ShiftAmount>::Output>,
    <Lhs as I64IsNegative>::Output:
        I64ShrSSelect<<Rhs as ShiftAmount>::Output, Shright<Lhs, <Rhs as ShiftAmount>::Output>>,
{
    type Output = <<Lhs as I64IsNegative>::Output as I64ShrSSelect<
        <Rhs as ShiftAmount>::Output,
        Shright<Lhs, <Rhs as ShiftAmount>::Output>,
    >>::Output;
}

pub trait I64Clz {
    type Output;
}

pub trait I64ClzZeroHelper<ValueT, SignZero> {
    type Output;
}

impl<ValueT, SignZero> I64ClzZeroHelper<ValueT, SignZero> for B1 {
    type Output = U64;
}

impl<ValueT> I64ClzZeroHelper<ValueT, B0> for B0 {
    type Output = U0;
}

impl<ValueT> I64ClzZeroHelper<ValueT, B1> for B0
where
    ValueT: I64ShlValue<U1>,
    <ValueT as I64ShlValue<U1>>::Output: I64Clz,
    <<ValueT as I64ShlValue<U1>>::Output as I64Clz>::Output: Add<U1>,
{
    type Output = Sum<<<ValueT as I64ShlValue<U1>>::Output as I64Clz>::Output, U1>;
}

impl<ValueT> I64Clz for ValueT
where
    ValueT: Unsigned + IsEqual<U0> + I64SignBitIsZero,
    <ValueT as IsEqual<U0>>::Output:
        I64ClzZeroHelper<ValueT, <ValueT as I64SignBitIsZero>::Output>,
{
    type Output = <<ValueT as IsEqual<U0>>::Output as I64ClzZeroHelper<
        ValueT,
        <ValueT as I64SignBitIsZero>::Output,
    >>::Output;
}

pub trait I64Ctz {
    type Output;
}

pub trait I64CtzZeroHelper<ValueT, LowBit> {
    type Output;
}

impl<ValueT, LowBit> I64CtzZeroHelper<ValueT, LowBit> for B1 {
    type Output = U64;
}

impl<ValueT> I64CtzZeroHelper<ValueT, U1> for B0 {
    type Output = U0;
}

impl<ValueT> I64CtzZeroHelper<ValueT, U0> for B0
where
    ValueT: Shr<U1>,
    Shright<ValueT, U1>: I64Ctz,
    <Shright<ValueT, U1> as I64Ctz>::Output: Add<U1>,
{
    type Output = Sum<<Shright<ValueT, U1> as I64Ctz>::Output, U1>;
}

impl<ValueT> I64Ctz for ValueT
where
    ValueT: Unsigned + IsEqual<U0> + BitAnd<U1>,
    <ValueT as IsEqual<U0>>::Output: I64CtzZeroHelper<ValueT, And<ValueT, U1>>,
{
    type Output =
        <<ValueT as IsEqual<U0>>::Output as I64CtzZeroHelper<ValueT, And<ValueT, U1>>>::Output;
}

pub trait I64Popcnt {
    type Output;
}

pub trait I64PopcntZeroHelper<ValueT> {
    type Output;
}

impl<ValueT> I64PopcntZeroHelper<ValueT> for B1 {
    type Output = U0;
}

impl<ValueT> I64PopcntZeroHelper<ValueT> for B0
where
    ValueT: Shr<U1> + BitAnd<U1>,
    Shright<ValueT, U1>: I64Popcnt,
    <Shright<ValueT, U1> as I64Popcnt>::Output: Add<And<ValueT, U1>>,
{
    type Output = Sum<<Shright<ValueT, U1> as I64Popcnt>::Output, And<ValueT, U1>>;
}

impl<ValueT> I64Popcnt for ValueT
where
    ValueT: Unsigned + IsEqual<U0>,
    <ValueT as IsEqual<U0>>::Output: I64PopcntZeroHelper<ValueT>,
{
    type Output = <<ValueT as IsEqual<U0>>::Output as I64PopcntZeroHelper<ValueT>>::Output;
}

pub trait I64InvRotAmount<Rhs> {
    type Output;
}

pub trait I64InvRotAmountValue {
    type Output;
}

impl I64InvRotAmountValue for U0 {
    type Output = U0;
}

macro_rules! inv_rot_amount_impls_64 {
    ($($amount:ty => $inv:ty),* $(,)?) => {
        $(
            impl I64InvRotAmountValue for $amount {
                type Output = $inv;
            }
        )*
    };
}

inv_rot_amount_impls_64!(
    typenum::U1 => typenum::U63,
    typenum::U2 => typenum::U62,
    typenum::U3 => typenum::U61,
    typenum::U4 => typenum::U60,
    typenum::U5 => typenum::U59,
    typenum::U6 => typenum::U58,
    typenum::U7 => typenum::U57,
    typenum::U8 => typenum::U56,
    typenum::U9 => typenum::U55,
    typenum::U10 => typenum::U54,
    typenum::U11 => typenum::U53,
    typenum::U12 => typenum::U52,
    typenum::U13 => typenum::U51,
    typenum::U14 => typenum::U50,
    typenum::U15 => typenum::U49,
    typenum::U16 => typenum::U48,
    typenum::U17 => typenum::U47,
    typenum::U18 => typenum::U46,
    typenum::U19 => typenum::U45,
    typenum::U20 => typenum::U44,
    typenum::U21 => typenum::U43,
    typenum::U22 => typenum::U42,
    typenum::U23 => typenum::U41,
    typenum::U24 => typenum::U40,
    typenum::U25 => typenum::U39,
    typenum::U26 => typenum::U38,
    typenum::U27 => typenum::U37,
    typenum::U28 => typenum::U36,
    typenum::U29 => typenum::U35,
    typenum::U30 => typenum::U34,
    typenum::U31 => typenum::U33,
    typenum::U32 => typenum::U32,
    typenum::U33 => typenum::U31,
    typenum::U34 => typenum::U30,
    typenum::U35 => typenum::U29,
    typenum::U36 => typenum::U28,
    typenum::U37 => typenum::U27,
    typenum::U38 => typenum::U26,
    typenum::U39 => typenum::U25,
    typenum::U40 => typenum::U24,
    typenum::U41 => typenum::U23,
    typenum::U42 => typenum::U22,
    typenum::U43 => typenum::U21,
    typenum::U44 => typenum::U20,
    typenum::U45 => typenum::U19,
    typenum::U46 => typenum::U18,
    typenum::U47 => typenum::U17,
    typenum::U48 => typenum::U16,
    typenum::U49 => typenum::U15,
    typenum::U50 => typenum::U14,
    typenum::U51 => typenum::U13,
    typenum::U52 => typenum::U12,
    typenum::U53 => typenum::U11,
    typenum::U54 => typenum::U10,
    typenum::U55 => typenum::U9,
    typenum::U56 => typenum::U8,
    typenum::U57 => typenum::U7,
    typenum::U58 => typenum::U6,
    typenum::U59 => typenum::U5,
    typenum::U60 => typenum::U4,
    typenum::U61 => typenum::U3,
    typenum::U62 => typenum::U2,
    typenum::U63 => typenum::U1,
);

impl<Lhs, Rhs> I64InvRotAmount<Rhs> for Lhs
where
    Rhs: ShiftAmount,
    <Rhs as ShiftAmount>::Output: I64InvRotAmountValue,
{
    type Output = <<Rhs as ShiftAmount>::Output as I64InvRotAmountValue>::Output;
}

pub trait I64RotlValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64RotlValue<Rhs> for Lhs
where
    Lhs: I64ShlValue<Rhs>
        + I64InvRotAmount<Rhs>
        + I64ShrUValue<<Lhs as I64InvRotAmount<Rhs>>::Output>,
    <Lhs as I64ShlValue<Rhs>>::Output:
        BitOr<<Lhs as I64ShrUValue<<Lhs as I64InvRotAmount<Rhs>>::Output>>::Output>,
{
    type Output = Or<
        <Lhs as I64ShlValue<Rhs>>::Output,
        <Lhs as I64ShrUValue<<Lhs as I64InvRotAmount<Rhs>>::Output>>::Output,
    >;
}

pub trait I64RotrValue<Rhs> {
    type Output;
}

impl<Lhs, Rhs> I64RotrValue<Rhs> for Lhs
where
    Lhs: I64ShrUValue<Rhs>
        + I64InvRotAmount<Rhs>
        + I64ShlValue<<Lhs as I64InvRotAmount<Rhs>>::Output>,
    <Lhs as I64ShrUValue<Rhs>>::Output:
        BitOr<<Lhs as I64ShlValue<<Lhs as I64InvRotAmount<Rhs>>::Output>>::Output>,
{
    type Output = Or<
        <Lhs as I64ShrUValue<Rhs>>::Output,
        <Lhs as I64ShlValue<<Lhs as I64InvRotAmount<Rhs>>::Output>>::Output,
    >;
}

pub trait I64ExtendI32U {
    type Output;
}

pub trait Mask32 {
    type Output;
}

impl<ValueT> Mask32 for ValueT
where
    ValueT: BitAnd<U4294967295>,
{
    type Output = And<ValueT, U4294967295>;
}

impl<ValueT> I64ExtendI32U for ValueT
where
    ValueT: Unsigned + Mask32,
{
    type Output = <ValueT as Mask32>::Output;
}

pub trait I64ExtendI32S {
    type Output;
}

pub trait SignBit32 {
    type Output;
}

impl<ValueT> SignBit32 for ValueT
where
    ValueT: Mask32,
    <ValueT as Mask32>::Output: BitAnd<crate::helpers::i32::U2147483648>,
{
    type Output = And<<ValueT as Mask32>::Output, crate::helpers::i32::U2147483648>;
}

pub trait I64ExtendI32SHelper<LowBits> {
    type Output;
}

impl<LowBits> I64ExtendI32SHelper<LowBits> for B1 {
    type Output = LowBits;
}

impl<LowBits> I64ExtendI32SHelper<LowBits> for B0
where
    LowBits: BitOr<Xor<U18446744073709551615, U4294967295>>,
{
    type Output = Or<LowBits, Xor<U18446744073709551615, U4294967295>>;
}

impl<ValueT> I64ExtendI32S for ValueT
where
    ValueT: Unsigned + Mask32 + SignBit32,
    <ValueT as SignBit32>::Output: IsEqual<U0>,
    <<ValueT as SignBit32>::Output as IsEqual<U0>>::Output:
        I64ExtendI32SHelper<<ValueT as Mask32>::Output>,
{
    type Output =
        <<<ValueT as SignBit32>::Output as IsEqual<U0>>::Output as I64ExtendI32SHelper<
            <ValueT as Mask32>::Output,
        >>::Output;
}
