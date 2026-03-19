use core::marker::PhantomData;
use core::ops::Sub;

use typelude_std::std::col::array::{Array, IsList, Nil};
use typenum::{B1, Sub1, U0, UInt, Unsigned};

pub trait GetAt<Idx> {
    type Output;
}

#[derive(Debug)]
pub struct FoundLocal<Value>(pub PhantomData<Value>);

#[derive(Debug)]
pub struct MissingLocal<Idx>(pub PhantomData<Idx>);

impl<Idx> GetAt<Idx> for Nil {
    type Output = MissingLocal<Idx>;
}

impl<Head, Tail> GetAt<U0> for Array<Head, Tail>
where
    Tail: IsList,
{
    type Output = FoundLocal<Head>;
}

impl<Head, Tail, N, B> GetAt<UInt<N, B>> for Array<Head, Tail>
where
    UInt<N, B>: Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    Tail: GetAt<Sub1<UInt<N, B>>> + IsList,
{
    type Output = <Tail as GetAt<Sub1<UInt<N, B>>>>::Output;
}

pub trait SetAt<Idx, Value> {
    type Output;
}

#[derive(Debug)]
pub struct SetLocalOk<Locals>(pub PhantomData<Locals>);

impl<Idx, Value> SetAt<Idx, Value> for Nil {
    type Output = MissingLocal<Idx>;
}

impl<Head, Tail, Value> SetAt<U0, Value> for Array<Head, Tail>
where
    Tail: IsList,
{
    type Output = SetLocalOk<Array<Value, Tail>>;
}

impl<Head, Tail, N, B, Value> SetAt<UInt<N, B>, Value> for Array<Head, Tail>
where
    UInt<N, B>: Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    Tail: SetAt<Sub1<UInt<N, B>>, Value> + IsList,
    <Tail as SetAt<Sub1<UInt<N, B>>, Value>>::Output: SetAtTailResult,
{
    type Output =
        <<Tail as SetAt<Sub1<UInt<N, B>>, Value>>::Output as SetAtTailResult>::WithHead<Head>;
}

pub trait SetAtTailResult {
    type WithHead<Head>;
}

impl<Locals> SetAtTailResult for SetLocalOk<Locals>
where
    Locals: IsList,
{
    type WithHead<Head> = SetLocalOk<Array<Head, Locals>>;
}

impl<Idx> SetAtTailResult for MissingLocal<Idx> {
    type WithHead<Head> = MissingLocal<Idx>;
}
