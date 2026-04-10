use core::marker::PhantomData;

use typelude_col::{TArr, TTerm};
use typelude_num::peano::{Succ, Zero};
use typelude_std::core::{Concat, Get, Len};

use crate::{
    func::WasmFunc,
    module::{WasmFuncType, WasmHostFunc, WasmResolvedModule},
    value::WasmI32,
};

pub trait PopArgs<ParamCount> {
    type RemainingStack;
    type Params;
}

impl<Stack> PopArgs<typenum::U0> for Stack {
    type RemainingStack = Stack;
    type Params = TTerm;
}

impl<Stack> PopArgs<Zero> for Stack {
    type RemainingStack = Stack;
    type Params = TTerm;
}

impl<ValueT, Tail, N, B> PopArgs<typenum::UInt<N, B>> for TArr<WasmI32<ValueT>, Tail>
where
    typenum::UInt<N, B>: core::ops::Sub<typenum::U1>,
    Tail: PopArgs<<typenum::UInt<N, B> as core::ops::Sub<typenum::U1>>::Output>,
    <Tail as PopArgs<<typenum::UInt<N, B> as core::ops::Sub<typenum::U1>>::Output>>::Params:
        typelude_std::core::Append<WasmI32<ValueT>>,
{
    type RemainingStack =
        <Tail as PopArgs<<typenum::UInt<N, B> as core::ops::Sub<typenum::U1>>::Output>>::RemainingStack;
    type Params = <<Tail as PopArgs<<typenum::UInt<N, B> as core::ops::Sub<typenum::U1>>::Output>>::Params as typelude_std::core::Append<WasmI32<ValueT>>>::Output;
}

impl<ValueT, Tail, N> PopArgs<Succ<N>> for TArr<WasmI32<ValueT>, Tail>
where
    Tail: PopArgs<N>,
    <Tail as PopArgs<N>>::Params: typelude_std::core::Append<WasmI32<ValueT>>,
{
    type RemainingStack = <Tail as PopArgs<N>>::RemainingStack;
    type Params =
        <<Tail as PopArgs<N>>::Params as typelude_std::core::Append<WasmI32<ValueT>>>::Output;
}

pub trait ParamCount {
    type Output;
}

impl<Params, Results> ParamCount for WasmFuncType<Params, Results>
where
    Params: Len,
{
    type Output = <Params as Len>::Output;
}

pub trait BindLocals<FuncType, LocalInits>: PopArgs<<FuncType as ParamCount>::Output>
where
    FuncType: ParamCount,
{
    type Output;
}

impl<Stack, FuncType, LocalInits> BindLocals<FuncType, LocalInits> for Stack
where
    FuncType: ParamCount,
    Stack: PopArgs<<FuncType as ParamCount>::Output>,
    <Stack as PopArgs<<FuncType as ParamCount>::Output>>::Params: Concat<LocalInits>,
{
    type Output =
        <<Stack as PopArgs<<FuncType as ParamCount>::Output>>::Params as Concat<LocalInits>>::Output;
}

pub trait ModuleFuncLookup<FuncIdx> {
    type Output;
}

impl<Funcs, Types, Exports, FuncIdx> ModuleFuncLookup<FuncIdx> for WasmResolvedModule<Funcs, Types, Exports>
where
    Funcs: Get<FuncIdx>,
{
    type Output = <Funcs as Get<FuncIdx>>::Output;
}

pub trait FuncSignature {
    type Output;
}

impl<FuncType, LocalInits, Program> FuncSignature for WasmFunc<FuncType, LocalInits, Program> {
    type Output = FuncType;
}

impl<FuncType, Host> FuncSignature for WasmHostFunc<FuncType, Host> {
    type Output = FuncType;
}

#[derive(Debug, Default)]
pub struct HostCallResult<Store, Results>(pub PhantomData<(Store, Results)>);

pub trait HostCall<FuncType, Store, Args> {
    type Output;
}
