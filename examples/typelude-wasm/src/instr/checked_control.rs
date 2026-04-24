//! control opcode の checked 意味論。
//!
//! success-only runtime で trait 未解決になりやすい selected failure を
//! `WasmTrap<Reason>` に変換します。現在は `unreachable` と `call_indirect`
//! の null table slot / table OOB / function type mismatch を扱います。

use typelude_col::TArr;
use typelude_std::core::{Eval, Get};

use crate::{
    helpers::{
        call::{FuncSignature, FuncTypeEq, ModuleFuncLookup},
        table::{TableReadRefChecked, TableSlotOob},
    },
    instr::control::InvokeCall,
    module::WasmResolvedModule,
    opcode::{OpCallIndirect, OpUnreachable},
    run::{
        CheckedStep, TrapCallIndirectNull, TrapCallIndirectTableOob, TrapCallIndirectTypeMismatch,
        TrapUnreachable, WasmDone, WasmTrap,
    },
    state::{NullFuncRef, WasmState, WasmStore},
    value::WasmI32,
};

impl<Module, Store, Stack, Locals, Frames, Branches, Rest> Eval
    for CheckedStep<
        WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<OpUnreachable, Rest>>,
    >
{
    type Output = WasmTrap<TrapUnreachable>;
}

/// checked `call_indirect` の table 参照結果を最終 outcome へ変換する bridge
/// trait。
pub trait CheckedCallIndirectResolved<
    TypeIdx,
    Module,
    Store,
    Stack,
    Locals,
    Frames,
    Branches,
    Rest,
>
{
    type Output;
}

impl<TypeIdx, Module, Store, Stack, Locals, Frames, Branches, Rest>
    CheckedCallIndirectResolved<TypeIdx, Module, Store, Stack, Locals, Frames, Branches, Rest>
    for TableSlotOob
{
    type Output = WasmTrap<TrapCallIndirectTableOob>;
}

impl<TypeIdx, Module, Store, Stack, Locals, Frames, Branches, Rest>
    CheckedCallIndirectResolved<TypeIdx, Module, Store, Stack, Locals, Frames, Branches, Rest>
    for NullFuncRef
{
    type Output = WasmTrap<TrapCallIndirectNull>;
}

impl<TypeIdx, FuncIdx, Module, Store, Stack, Locals, Frames, Branches, Rest>
    CheckedCallIndirectResolved<TypeIdx, Module, Store, Stack, Locals, Frames, Branches, Rest>
    for FuncIdx
where
    FuncIdx: typenum::Unsigned,
    Module:
        CheckedCallIndirectTarget<TypeIdx, FuncIdx, Store, Stack, Locals, Frames, Branches, Rest>,
{
    type Output = <Module as CheckedCallIndirectTarget<
        TypeIdx,
        FuncIdx,
        Store,
        Stack,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
}

/// checked `call_indirect` の関数型一致結果を outcome へ変換する bridge trait。
pub trait CheckedCallIndirectTypeMatch<Func, Module, Store, Stack, Locals, Frames, Branches, Rest>
{
    type Output;
}

impl<Func, Module, Store, Stack, Locals, Frames, Branches, Rest>
    CheckedCallIndirectTypeMatch<Func, Module, Store, Stack, Locals, Frames, Branches, Rest>
    for typenum::B0
{
    type Output = WasmTrap<TrapCallIndirectTypeMismatch>;
}

impl<Func, Module, Store, Stack, Locals, Frames, Branches, Rest>
    CheckedCallIndirectTypeMatch<Func, Module, Store, Stack, Locals, Frames, Branches, Rest>
    for typenum::B1
where
    Func: InvokeCall<Module, Store, Stack, Locals, Frames, Branches, Rest>,
{
    type Output = WasmDone<
        <Func as InvokeCall<Module, Store, Stack, Locals, Frames, Branches, Rest>>::Output,
    >;
}

/// checked `call_indirect` の最終 target 解決 trait。
///
/// 関数型が一致すれば `WasmDone<State>`、不一致なら
/// `WasmTrap<TrapCallIndirectTypeMismatch>` を返します。
pub trait CheckedCallIndirectTarget<TypeIdx, FuncIdx, Store, Stack, Locals, Frames, Branches, Rest>
{
    type Output;
}

impl<Funcs, Types, Exports, TypeIdx, FuncIdx, Store, Stack, Locals, Frames, Branches, Rest>
    CheckedCallIndirectTarget<TypeIdx, FuncIdx, Store, Stack, Locals, Frames, Branches, Rest>
    for WasmResolvedModule<Funcs, Types, Exports>
where
    WasmResolvedModule<Funcs, Types, Exports>: ModuleFuncLookup<FuncIdx>,
    Types: Get<TypeIdx>,
    <WasmResolvedModule<Funcs, Types, Exports> as ModuleFuncLookup<FuncIdx>>::Output:
        FuncSignature + InvokeCall<
            WasmResolvedModule<Funcs, Types, Exports>,
            Store,
            Stack,
            Locals,
            Frames,
            Branches,
            Rest,
        >,
    <<WasmResolvedModule<Funcs, Types, Exports> as ModuleFuncLookup<FuncIdx>>::Output as FuncSignature>::Output:
        FuncTypeEq<<Types as Get<TypeIdx>>::Output>,
    <<<WasmResolvedModule<Funcs, Types, Exports> as ModuleFuncLookup<FuncIdx>>::Output as FuncSignature>::Output as FuncTypeEq<
        <Types as Get<TypeIdx>>::Output,
    >>::Output: CheckedCallIndirectTypeMatch<
        <WasmResolvedModule<Funcs, Types, Exports> as ModuleFuncLookup<FuncIdx>>::Output,
        WasmResolvedModule<Funcs, Types, Exports>,
        Store,
        Stack,
        Locals,
        Frames,
        Branches,
        Rest,
    >,
{
    type Output = <<<<WasmResolvedModule<Funcs, Types, Exports> as ModuleFuncLookup<FuncIdx>>::Output as FuncSignature>::Output as FuncTypeEq<
        <Types as Get<TypeIdx>>::Output,
    >>::Output as CheckedCallIndirectTypeMatch<
        <WasmResolvedModule<Funcs, Types, Exports> as ModuleFuncLookup<FuncIdx>>::Output,
        WasmResolvedModule<Funcs, Types, Exports>,
        Store,
        Stack,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
}

impl<
    Module,
    Memory,
    Tables,
    Globals,
    TypeIdx,
    TableIdx,
    SlotIdx,
    Stack,
    Locals,
    Frames,
    Branches,
    Rest,
> Eval
    for CheckedStep<
        WasmState<
            Module,
            WasmStore<Memory, Tables, Globals>,
            TArr<WasmI32<SlotIdx>, Stack>,
            Locals,
            Frames,
            Branches,
            TArr<OpCallIndirect<TypeIdx, TableIdx>, Rest>,
        >,
    >
where
    Tables: Get<TableIdx>,
    <Tables as Get<TableIdx>>::Output: TableReadRefChecked<SlotIdx>,
    <<Tables as Get<TableIdx>>::Output as TableReadRefChecked<SlotIdx>>::Output:
        CheckedCallIndirectResolved<
                TypeIdx,
                Module,
                WasmStore<Memory, Tables, Globals>,
                Stack,
                Locals,
                Frames,
                Branches,
                Rest,
            >,
{
    type Output = <<<Tables as Get<TableIdx>>::Output as TableReadRefChecked<SlotIdx>>::Output as CheckedCallIndirectResolved<
        TypeIdx,
        Module,
        WasmStore<Memory, Tables, Globals>,
        Stack,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
}
