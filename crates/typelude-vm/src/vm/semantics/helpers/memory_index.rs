//! Memory index resolution helpers.
//!
//! These helpers classify reads and writes as found/missing so runtime-visible
//! trap semantics stay explicit instead of collapsing into trait-resolution
//! failures.

use core::{marker::PhantomData, ops::Sub};

use typelude_std::std::col::array::{Array, IsList, Nil};
use typenum::{B0, B1, NInt, PInt, Sub1, U0, UInt, Unsigned, Z0};

/// Reads a memory slot, classifying missing indices explicitly.
pub trait MemoryGet<Idx> {
    type Output;
}

#[derive(Debug)]
pub struct FoundMemory<Value>(pub PhantomData<Value>);

#[derive(Debug)]
pub struct MissingMemory<Idx>(pub PhantomData<Idx>);

impl<Idx> MemoryGet<Idx> for Nil {
    type Output = MissingMemory<Idx>;
}

impl<Head, Tail> MemoryGet<U0> for Array<Head, Tail>
where
    Tail: IsList,
{
    type Output = FoundMemory<Head>;
}

impl<Head, Tail, N, B> MemoryGet<UInt<N, B>> for Array<Head, Tail>
where
    UInt<N, B>: Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    Tail: MemoryGet<Sub1<UInt<N, B>>> + IsList,
{
    type Output = <Tail as MemoryGet<Sub1<UInt<N, B>>>>::Output;
}

macro_rules! impl_invalid_memory_get {
    ($($idx:ty),+ $(,)?) => {
        $(
            impl<Head, Tail> MemoryGet<$idx> for Array<Head, Tail>
            where
                Tail: IsList,
            {
                type Output = MissingMemory<$idx>;
            }
        )+
    };
}

impl_invalid_memory_get!(B0, B1, Z0);

impl<Head, Tail, U> MemoryGet<PInt<U>> for Array<Head, Tail>
where
    Tail: IsList,
    U: Unsigned + typenum::NonZero,
{
    type Output = MissingMemory<PInt<U>>;
}

impl<Head, Tail, U> MemoryGet<NInt<U>> for Array<Head, Tail>
where
    Tail: IsList,
    U: Unsigned + typenum::NonZero,
{
    type Output = MissingMemory<NInt<U>>;
}

/// Writes a memory slot, classifying missing indices explicitly.
pub trait MemorySet<Idx, Value> {
    type Output;
}

#[derive(Debug)]
pub struct SetMemoryOk<Memory>(pub PhantomData<Memory>);

impl<Idx, Value> MemorySet<Idx, Value> for Nil {
    type Output = MissingMemory<Idx>;
}

impl<Head, Tail, Value> MemorySet<U0, Value> for Array<Head, Tail>
where
    Tail: IsList,
{
    type Output = SetMemoryOk<Array<Value, Tail>>;
}

impl<Head, Tail, N, B, Value> MemorySet<UInt<N, B>, Value> for Array<Head, Tail>
where
    UInt<N, B>: Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    Tail: MemorySet<Sub1<UInt<N, B>>, Value> + IsList,
    <Tail as MemorySet<Sub1<UInt<N, B>>, Value>>::Output: MemorySetTailResult,
{
    type Output =
        <<Tail as MemorySet<Sub1<UInt<N, B>>, Value>>::Output as MemorySetTailResult>::WithHead<
            Head,
        >;
}

macro_rules! impl_invalid_memory_set {
    ($($idx:ty),+ $(,)?) => {
        $(
            impl<Head, Tail, Value> MemorySet<$idx, Value> for Array<Head, Tail>
            where
                Tail: IsList,
            {
                type Output = MissingMemory<$idx>;
            }
        )+
    };
}

impl_invalid_memory_set!(B0, B1, Z0);

impl<Head, Tail, Value, U> MemorySet<PInt<U>, Value> for Array<Head, Tail>
where
    Tail: IsList,
    U: Unsigned + typenum::NonZero,
{
    type Output = MissingMemory<PInt<U>>;
}

impl<Head, Tail, Value, U> MemorySet<NInt<U>, Value> for Array<Head, Tail>
where
    Tail: IsList,
    U: Unsigned + typenum::NonZero,
{
    type Output = MissingMemory<NInt<U>>;
}

pub trait MemorySetTailResult {
    type WithHead<Head>;
}

impl<Memory> MemorySetTailResult for SetMemoryOk<Memory>
where
    Memory: IsList,
{
    type WithHead<Head> = SetMemoryOk<Array<Head, Memory>>;
}

impl<Idx> MemorySetTailResult for MissingMemory<Idx> {
    type WithHead<Head> = MissingMemory<Idx>;
}
