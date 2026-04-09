//! `typenum` interoperability for the shared core traits.

use core::ops::{Add as StdAdd, Div as StdDiv, Mul as StdMul, Sub as StdSub};

use typenum::{
    B0, B1, IsEqual, IsGreater, IsLess, IsLessOrEqual, NInt, NonZero, PInt, UInt, UTerm, Unsigned,
    Z0,
};

use super::{Add, And, Div, Eq, Gt, Le, Lt, Mul, Nand, Neq, Not, Or, Sub, Xor};

macro_rules! impl_typenum_arithmetic {
    ($lhs:ty) => {
        impl<Rhs> Add<Rhs> for $lhs
        where
            $lhs: StdAdd<Rhs>,
        {
            type Output = <$lhs as StdAdd<Rhs>>::Output;
        }

        impl<Rhs> Sub<Rhs> for $lhs
        where
            $lhs: StdSub<Rhs>,
        {
            type Output = <$lhs as StdSub<Rhs>>::Output;
        }

        impl<Rhs> Mul<Rhs> for $lhs
        where
            $lhs: StdMul<Rhs>,
        {
            type Output = <$lhs as StdMul<Rhs>>::Output;
        }

        impl<Rhs> Div<Rhs> for $lhs
        where
            $lhs: StdDiv<Rhs>,
        {
            type Output = <$lhs as StdDiv<Rhs>>::Output;
        }
    };
}

macro_rules! impl_typenum_compare {
    ($lhs:ty) => {
        impl<Rhs> Eq<Rhs> for $lhs
        where
            $lhs: IsEqual<Rhs>,
        {
            type Output = <$lhs as IsEqual<Rhs>>::Output;
        }

        impl<Rhs> Lt<Rhs> for $lhs
        where
            $lhs: IsLess<Rhs>,
        {
            type Output = <$lhs as IsLess<Rhs>>::Output;
        }

        impl<Rhs> Le<Rhs> for $lhs
        where
            $lhs: IsLessOrEqual<Rhs>,
        {
            type Output = <$lhs as IsLessOrEqual<Rhs>>::Output;
        }

        impl<Rhs> Gt<Rhs> for $lhs
        where
            $lhs: IsGreater<Rhs>,
        {
            type Output = <$lhs as IsGreater<Rhs>>::Output;
        }

        impl<Rhs> Neq<Rhs> for $lhs
        where
            $lhs: IsEqual<Rhs>,
            <$lhs as IsEqual<Rhs>>::Output: Not,
        {
            type Output = <<$lhs as IsEqual<Rhs>>::Output as Not>::Output;
        }
    };
}

impl Not for B0 {
    type Output = B1;
}

impl Not for B1 {
    type Output = B0;
}

impl And<B0> for B0 {
    type Output = B0;
}

impl And<B1> for B0 {
    type Output = B0;
}

impl And<B0> for B1 {
    type Output = B0;
}

impl And<B1> for B1 {
    type Output = B1;
}

impl Or<B0> for B0 {
    type Output = B0;
}

impl Or<B1> for B0 {
    type Output = B1;
}

impl Or<B0> for B1 {
    type Output = B1;
}

impl Or<B1> for B1 {
    type Output = B1;
}

impl Xor<B0> for B0 {
    type Output = B0;
}

impl Xor<B1> for B0 {
    type Output = B1;
}

impl Xor<B0> for B1 {
    type Output = B1;
}

impl Xor<B1> for B1 {
    type Output = B0;
}

impl Nand<B0> for B0 {
    type Output = B1;
}

impl Nand<B1> for B0 {
    type Output = B1;
}

impl Nand<B0> for B1 {
    type Output = B1;
}

impl Nand<B1> for B1 {
    type Output = B0;
}

impl_typenum_arithmetic!(UTerm);
impl_typenum_compare!(UTerm);

impl<N, B, Rhs> Add<Rhs> for UInt<N, B>
where
    UInt<N, B>: StdAdd<Rhs>,
{
    type Output = <UInt<N, B> as StdAdd<Rhs>>::Output;
}

impl<N, B, Rhs> Sub<Rhs> for UInt<N, B>
where
    UInt<N, B>: StdSub<Rhs>,
{
    type Output = <UInt<N, B> as StdSub<Rhs>>::Output;
}

impl<N, B, Rhs> Mul<Rhs> for UInt<N, B>
where
    UInt<N, B>: StdMul<Rhs>,
{
    type Output = <UInt<N, B> as StdMul<Rhs>>::Output;
}

impl<N, B, Rhs> Div<Rhs> for UInt<N, B>
where
    UInt<N, B>: StdDiv<Rhs>,
{
    type Output = <UInt<N, B> as StdDiv<Rhs>>::Output;
}

impl<N, B, Rhs> Eq<Rhs> for UInt<N, B>
where
    UInt<N, B>: IsEqual<Rhs>,
{
    type Output = <UInt<N, B> as IsEqual<Rhs>>::Output;
}

impl<N, B, Rhs> Neq<Rhs> for UInt<N, B>
where
    UInt<N, B>: IsEqual<Rhs>,
    <UInt<N, B> as IsEqual<Rhs>>::Output: Not,
{
    type Output = <<UInt<N, B> as IsEqual<Rhs>>::Output as Not>::Output;
}

impl<N, B, Rhs> Lt<Rhs> for UInt<N, B>
where
    UInt<N, B>: IsLess<Rhs>,
{
    type Output = <UInt<N, B> as IsLess<Rhs>>::Output;
}

impl<N, B, Rhs> Le<Rhs> for UInt<N, B>
where
    UInt<N, B>: IsLessOrEqual<Rhs>,
{
    type Output = <UInt<N, B> as IsLessOrEqual<Rhs>>::Output;
}

impl<N, B, Rhs> Gt<Rhs> for UInt<N, B>
where
    UInt<N, B>: IsGreater<Rhs>,
{
    type Output = <UInt<N, B> as IsGreater<Rhs>>::Output;
}

impl<U, Rhs> Add<Rhs> for PInt<U>
where
    U: Unsigned + NonZero,
    PInt<U>: StdAdd<Rhs>,
{
    type Output = <PInt<U> as StdAdd<Rhs>>::Output;
}

impl<U, Rhs> Sub<Rhs> for PInt<U>
where
    U: Unsigned + NonZero,
    PInt<U>: StdSub<Rhs>,
{
    type Output = <PInt<U> as StdSub<Rhs>>::Output;
}

impl<U, Rhs> Mul<Rhs> for PInt<U>
where
    U: Unsigned + NonZero,
    PInt<U>: StdMul<Rhs>,
{
    type Output = <PInt<U> as StdMul<Rhs>>::Output;
}

impl<U, Rhs> Div<Rhs> for PInt<U>
where
    U: Unsigned + NonZero,
    PInt<U>: StdDiv<Rhs>,
{
    type Output = <PInt<U> as StdDiv<Rhs>>::Output;
}

impl<U, Rhs> Eq<Rhs> for PInt<U>
where
    U: Unsigned + NonZero,
    PInt<U>: IsEqual<Rhs>,
{
    type Output = <PInt<U> as IsEqual<Rhs>>::Output;
}

impl<U, Rhs> Neq<Rhs> for PInt<U>
where
    U: Unsigned + NonZero,
    PInt<U>: IsEqual<Rhs>,
    <PInt<U> as IsEqual<Rhs>>::Output: Not,
{
    type Output = <<PInt<U> as IsEqual<Rhs>>::Output as Not>::Output;
}

impl<U, Rhs> Lt<Rhs> for PInt<U>
where
    U: Unsigned + NonZero,
    PInt<U>: IsLess<Rhs>,
{
    type Output = <PInt<U> as IsLess<Rhs>>::Output;
}

impl<U, Rhs> Le<Rhs> for PInt<U>
where
    U: Unsigned + NonZero,
    PInt<U>: IsLessOrEqual<Rhs>,
{
    type Output = <PInt<U> as IsLessOrEqual<Rhs>>::Output;
}

impl<U, Rhs> Gt<Rhs> for PInt<U>
where
    U: Unsigned + NonZero,
    PInt<U>: IsGreater<Rhs>,
{
    type Output = <PInt<U> as IsGreater<Rhs>>::Output;
}

impl<U, Rhs> Add<Rhs> for NInt<U>
where
    U: Unsigned + NonZero,
    NInt<U>: StdAdd<Rhs>,
{
    type Output = <NInt<U> as StdAdd<Rhs>>::Output;
}

impl<U, Rhs> Sub<Rhs> for NInt<U>
where
    U: Unsigned + NonZero,
    NInt<U>: StdSub<Rhs>,
{
    type Output = <NInt<U> as StdSub<Rhs>>::Output;
}

impl<U, Rhs> Mul<Rhs> for NInt<U>
where
    U: Unsigned + NonZero,
    NInt<U>: StdMul<Rhs>,
{
    type Output = <NInt<U> as StdMul<Rhs>>::Output;
}

impl<U, Rhs> Div<Rhs> for NInt<U>
where
    U: Unsigned + NonZero,
    NInt<U>: StdDiv<Rhs>,
{
    type Output = <NInt<U> as StdDiv<Rhs>>::Output;
}

impl<U, Rhs> Eq<Rhs> for NInt<U>
where
    U: Unsigned + NonZero,
    NInt<U>: IsEqual<Rhs>,
{
    type Output = <NInt<U> as IsEqual<Rhs>>::Output;
}

impl<U, Rhs> Neq<Rhs> for NInt<U>
where
    U: Unsigned + NonZero,
    NInt<U>: IsEqual<Rhs>,
    <NInt<U> as IsEqual<Rhs>>::Output: Not,
{
    type Output = <<NInt<U> as IsEqual<Rhs>>::Output as Not>::Output;
}

impl<U, Rhs> Lt<Rhs> for NInt<U>
where
    U: Unsigned + NonZero,
    NInt<U>: IsLess<Rhs>,
{
    type Output = <NInt<U> as IsLess<Rhs>>::Output;
}

impl<U, Rhs> Le<Rhs> for NInt<U>
where
    U: Unsigned + NonZero,
    NInt<U>: IsLessOrEqual<Rhs>,
{
    type Output = <NInt<U> as IsLessOrEqual<Rhs>>::Output;
}

impl<U, Rhs> Gt<Rhs> for NInt<U>
where
    U: Unsigned + NonZero,
    NInt<U>: IsGreater<Rhs>,
{
    type Output = <NInt<U> as IsGreater<Rhs>>::Output;
}

impl_typenum_arithmetic!(Z0);
impl_typenum_compare!(Z0);
