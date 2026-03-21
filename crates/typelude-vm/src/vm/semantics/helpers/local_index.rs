//! Local index resolution helpers.
//!
//! These helpers never fail with a trait-resolution error for invalid user
//! indices. They classify lookups and updates as found/missing so the VM
//! semantics can turn them into `BadLocalIndex`.

use core::{marker::PhantomData, ops::Sub};

use typelude_std::std::col::array::{Array, IsList, Nil};
use typenum::{B0, B1, NInt, PInt, Sub1, U0, UInt, Unsigned, Z0};

/// Reads a local slot, classifying missing indices explicitly.
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

macro_rules! impl_invalid_get_at {
    ($($idx:ty),+ $(,)?) => {
        $(
            impl<Head, Tail> GetAt<$idx> for Array<Head, Tail>
            where
                Tail: IsList,
            {
                type Output = MissingLocal<$idx>;
            }
        )+
    };
}

impl_invalid_get_at!(B0, B1, Z0);

impl<Head, Tail, U> GetAt<PInt<U>> for Array<Head, Tail>
where
    Tail: IsList,
    U: Unsigned + typenum::NonZero,
{
    type Output = MissingLocal<PInt<U>>;
}

impl<Head, Tail, U> GetAt<NInt<U>> for Array<Head, Tail>
where
    Tail: IsList,
    U: Unsigned + typenum::NonZero,
{
    type Output = MissingLocal<NInt<U>>;
}

/// Writes a local slot, classifying missing indices explicitly.
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

macro_rules! impl_invalid_set_at {
    ($($idx:ty),+ $(,)?) => {
        $(
            impl<Head, Tail, Value> SetAt<$idx, Value> for Array<Head, Tail>
            where
                Tail: IsList,
            {
                type Output = MissingLocal<$idx>;
            }
        )+
    };
}

impl_invalid_set_at!(B0, B1, Z0);

impl<Head, Tail, Value, U> SetAt<PInt<U>, Value> for Array<Head, Tail>
where
    Tail: IsList,
    U: Unsigned + typenum::NonZero,
{
    type Output = MissingLocal<PInt<U>>;
}

impl<Head, Tail, Value, U> SetAt<NInt<U>, Value> for Array<Head, Tail>
where
    Tail: IsList,
    U: Unsigned + typenum::NonZero,
{
    type Output = MissingLocal<NInt<U>>;
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
