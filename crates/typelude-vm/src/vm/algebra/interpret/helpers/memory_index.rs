use core::marker::PhantomData;
use core::ops::Sub;

use typelude_std::std::col::array::{Array, IsList, Nil};
use typenum::{B1, Sub1, U0, UInt, Unsigned};

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
