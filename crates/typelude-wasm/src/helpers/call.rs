use core::marker::PhantomData;

use typelude_col::{TArr, TTerm};
use typelude_std::core::{Append, Concat, Get};

use crate::{
    func::WasmFunc,
    module::{WasmFuncType, WasmHostFunc, WasmResolvedModule},
    value::{WasmI32, WasmI32Type, WasmI64, WasmI64Type},
};

pub trait ReverseList {
    type Output;
}

impl ReverseList for TTerm {
    type Output = TTerm;
}

impl<Head, Tail> ReverseList for TArr<Head, Tail>
where
    Tail: ReverseList,
    <Tail as ReverseList>::Output: Append<Head>,
{
    type Output = <<Tail as ReverseList>::Output as Append<Head>>::Output;
}

pub trait PopArg<ExpectedType> {
    type RemainingStack;
    type Value;
}

impl<ValueT, Tail> PopArg<WasmI32Type> for TArr<WasmI32<ValueT>, Tail> {
    type RemainingStack = Tail;
    type Value = WasmI32<ValueT>;
}

impl<ValueT, Tail> PopArg<WasmI64Type> for TArr<WasmI64<ValueT>, Tail> {
    type RemainingStack = Tail;
    type Value = WasmI64<ValueT>;
}

pub trait PopArgs<ParamTypes> {
    type RemainingStack;
    type Params;
}

impl<Stack> PopArgs<TTerm> for Stack {
    type RemainingStack = Stack;
    type Params = TTerm;
}

impl<Stack, ParamType, TailTypes> PopArgs<TArr<ParamType, TailTypes>> for Stack
where
    Stack: PopArg<ParamType>,
    <Stack as PopArg<ParamType>>::RemainingStack: PopArgs<TailTypes>,
    <<Stack as PopArg<ParamType>>::RemainingStack as PopArgs<TailTypes>>::Params:
        Append<<Stack as PopArg<ParamType>>::Value>,
{
    type RemainingStack =
        <<Stack as PopArg<ParamType>>::RemainingStack as PopArgs<TailTypes>>::RemainingStack;
    type Params = <<<Stack as PopArg<ParamType>>::RemainingStack as PopArgs<TailTypes>>::Params as Append<
        <Stack as PopArg<ParamType>>::Value,
    >>::Output;
}

pub trait ParamTypes {
    type Output;
}

impl<Params, Results> ParamTypes for WasmFuncType<Params, Results> {
    type Output = Params;
}

pub trait BindLocals<FuncType, LocalInits> {
    type Output;
}

impl<Stack, FuncType, LocalInits> BindLocals<FuncType, LocalInits> for Stack
where
    FuncType: ParamTypes,
    <FuncType as ParamTypes>::Output: ReverseList,
    Stack: PopArgs<<<FuncType as ParamTypes>::Output as ReverseList>::Output>,
    <Stack as PopArgs<<<FuncType as ParamTypes>::Output as ReverseList>::Output>>::Params:
        Concat<LocalInits>,
{
    type Output = <<Stack as PopArgs<<<FuncType as ParamTypes>::Output as ReverseList>::Output>>::Params as Concat<
        LocalInits,
    >>::Output;
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
