use typelude_col::TArr;
use typelude_std::core::{Eval, Get, Set};

use crate::{
    module::GlobalMut,
    opcode::{OpGlobalGet, OpGlobalSet},
    run::Step,
    state::{WasmGlobal, WasmState, WasmStore},
};

pub trait GlobalGetValue {
    type Output;
}

impl<Mutability, ValueT> GlobalGetValue for WasmGlobal<Mutability, ValueT> {
    type Output = ValueT;
}

pub trait MutableGlobalWith<ValueT> {
    type Output;
}

impl<ValueT, OldValue> MutableGlobalWith<ValueT> for WasmGlobal<GlobalMut, OldValue> {
    type Output = WasmGlobal<GlobalMut, ValueT>;
}

impl<Module, Memory, Tables, Globals, Idx, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            WasmStore<Memory, Tables, Globals>,
            Stack,
            Locals,
            Frames,
            Branches,
            TArr<OpGlobalGet<Idx>, Rest>,
        >,
    >
where
    Globals: Get<Idx>,
    <Globals as Get<Idx>>::Output: GlobalGetValue,
{
    type Output = WasmState<
        Module,
        WasmStore<Memory, Tables, Globals>,
        TArr<<<Globals as Get<Idx>>::Output as GlobalGetValue>::Output, Stack>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, Memory, Tables, Globals, Idx, ValueT, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            WasmStore<Memory, Tables, Globals>,
            TArr<ValueT, Stack>,
            Locals,
            Frames,
            Branches,
            TArr<OpGlobalSet<Idx>, Rest>,
        >,
    >
where
    Globals: Get<Idx>,
    <Globals as Get<Idx>>::Output: MutableGlobalWith<ValueT>,
    Globals: Set<Idx, <<Globals as Get<Idx>>::Output as MutableGlobalWith<ValueT>>::Output>,
{
    type Output = WasmState<
        Module,
        WasmStore<
            Memory,
            Tables,
            <Globals as Set<Idx, <<Globals as Get<Idx>>::Output as MutableGlobalWith<ValueT>>::Output>>::Output,
        >,
        Stack,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}
