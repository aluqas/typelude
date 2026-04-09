use typelude_col::{TArr, TTerm};
use typelude_std::core::{Append, Concat};
use typenum::{U1, UInt, UTerm};

use crate::value::WasmI32;

pub trait PopArgs<ParamCount> {
    type RemainingStack;
    type Params;
}

impl<Stack> PopArgs<UTerm> for Stack {
    type RemainingStack = Stack;
    type Params = TTerm;
}

impl<ValueT, Tail, N, B> PopArgs<UInt<N, B>> for TArr<WasmI32<ValueT>, Tail>
where
    UInt<N, B>: core::ops::Sub<U1>,
    Tail: PopArgs<<UInt<N, B> as core::ops::Sub<U1>>::Output>,
    <Tail as PopArgs<<UInt<N, B> as core::ops::Sub<U1>>::Output>>::Params: Append<WasmI32<ValueT>>,
{
    type RemainingStack = <Tail as PopArgs<<UInt<N, B> as core::ops::Sub<U1>>::Output>>::RemainingStack;
    type Params =
        <<Tail as PopArgs<<UInt<N, B> as core::ops::Sub<U1>>::Output>>::Params as Append<
            WasmI32<ValueT>,
        >>::Output;
}

pub trait BindLocals<ParamCount, LocalInits>: PopArgs<ParamCount> {
    type Output;
}

impl<Stack, ParamCount, LocalInits> BindLocals<ParamCount, LocalInits> for Stack
where
    Stack: PopArgs<ParamCount>,
    <Stack as PopArgs<ParamCount>>::Params: Concat<LocalInits>,
{
    type Output = <<Stack as PopArgs<ParamCount>>::Params as Concat<LocalInits>>::Output;
}
