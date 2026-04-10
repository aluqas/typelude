use typelude_col::{Concat, TArr, TTerm};
use typelude_std::core::{Eq, Eval, Get};
use typenum::{B0, B1, U0};

use crate::{
    frame::{BranchBlock, BranchLoop, ReturnFrame},
    func::WasmFunc,
    helpers::{
        branch_stack::{BranchJump, ContinueIfZero, ResolveBranch},
        call::{BindLocals, FuncSignature, HostCall, HostCallResult, ModuleFuncLookup, ParamCount, PopArgs},
        table::TableReadRef,
    },
    module::{WasmFuncType, WasmHostFunc, WasmResolvedModule},
    opcode::{
        OpBlock, OpBr, OpBrIf, OpCall, OpCallIndirect, OpEndBlock, OpEndFunc, OpEndLoop, OpIf,
        OpLoop, OpReturn, OpSelect,
    },
    run::Step,
    state::{WasmState, WasmStore},
    value::WasmI32,
};

#[doc(hidden)]
pub trait IfProgram<Then, Else, Rest> {
    type Output;
}

impl<Then, Else, Rest> IfProgram<Then, Else, Rest> for B0
where
    Then: Concat<Rest>,
{
    type Output = <Then as Concat<Rest>>::Output;
}

impl<Then, Else, Rest> IfProgram<Then, Else, Rest> for B1
where
    Else: Concat<Rest>,
{
    type Output = <Else as Concat<Rest>>::Output;
}

#[doc(hidden)]
pub trait SelectResult<TrueValue, FalseValue> {
    type Output;
}

impl<TrueValue, FalseValue> SelectResult<TrueValue, FalseValue> for B0 {
    type Output = TrueValue;
}

impl<TrueValue, FalseValue> SelectResult<TrueValue, FalseValue> for B1 {
    type Output = FalseValue;
}

#[doc(hidden)]
pub trait InvokeCall<Module, Store, Stack, Locals, Frames, Branches, Rest> {
    type Output;
}

impl<Module, Store, FuncType, LocalInits, FuncProgram, Stack, Locals, Frames, Branches, Rest>
    InvokeCall<Module, Store, Stack, Locals, Frames, Branches, Rest>
    for WasmFunc<FuncType, LocalInits, FuncProgram>
where
    FuncType: ParamCount,
    Stack: PopArgs<<FuncType as ParamCount>::Output> + BindLocals<FuncType, LocalInits>,
    FuncProgram: Concat<TArr<OpEndFunc, TTerm>>,
{
    type Output = WasmState<
        Module,
        Store,
        <Stack as PopArgs<<FuncType as ParamCount>::Output>>::RemainingStack,
        <Stack as BindLocals<FuncType, LocalInits>>::Output,
        TArr<ReturnFrame<Locals, Branches, Rest>, Frames>,
        TTerm,
        <FuncProgram as Concat<TArr<OpEndFunc, TTerm>>>::Output,
    >;
}

impl<Module, Store, FuncType, Host, Stack, Locals, Frames, Branches, Rest>
    InvokeCall<Module, Store, Stack, Locals, Frames, Branches, Rest> for WasmHostFunc<FuncType, Host>
where
    FuncType: ParamCount,
    Stack: PopArgs<<FuncType as ParamCount>::Output>,
    Host: HostCall<FuncType, Store, <Stack as PopArgs<<FuncType as ParamCount>::Output>>::Params>,
    <Host as HostCall<FuncType, Store, <Stack as PopArgs<<FuncType as ParamCount>::Output>>::Params>>::Output:
        HostCallOutput<Module, <Stack as PopArgs<<FuncType as ParamCount>::Output>>::RemainingStack, Locals, Frames, Branches, Rest>,
{
    type Output = <<Host as HostCall<
        FuncType,
        Store,
        <Stack as PopArgs<<FuncType as ParamCount>::Output>>::Params,
    >>::Output as HostCallOutput<
        Module,
        <Stack as PopArgs<<FuncType as ParamCount>::Output>>::RemainingStack,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
}

pub trait HostCallOutput<Module, Stack, Locals, Frames, Branches, Rest> {
    type Output;
}

impl<Module, Store, Results, Stack, Locals, Frames, Branches, Rest>
    HostCallOutput<Module, Stack, Locals, Frames, Branches, Rest> for HostCallResult<Store, Results>
where
    Results: Concat<Stack>,
{
    type Output = WasmState<
        Module,
        Store,
        <Results as Concat<Stack>>::Output,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, FuncIdx, Store, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<OpCall<FuncIdx>, Rest>>,
    >
where
    Module: ModuleFuncLookup<FuncIdx>,
    <Module as ModuleFuncLookup<FuncIdx>>::Output:
        InvokeCall<Module, Store, Stack, Locals, Frames, Branches, Rest>,
{
    type Output = <<Module as ModuleFuncLookup<FuncIdx>>::Output as InvokeCall<
        Module,
        Store,
        Stack,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
}

pub trait SameFuncType<Expected> {}

impl<Params, Results> SameFuncType<WasmFuncType<Params, Results>> for WasmFuncType<Params, Results> {}

pub trait CallIndirectTarget<TypeIdx, FuncIdx, Store, Stack, Locals, Frames, Branches, Rest> {
    type Output;
}

impl<Funcs, Types, Exports, TypeIdx, FuncIdx, Store, Stack, Locals, Frames, Branches, Rest>
    CallIndirectTarget<TypeIdx, FuncIdx, Store, Stack, Locals, Frames, Branches, Rest>
    for WasmResolvedModule<Funcs, Types, Exports>
where
    WasmResolvedModule<Funcs, Types, Exports>: ModuleFuncLookup<FuncIdx>,
    Types: Get<TypeIdx>,
    <WasmResolvedModule<Funcs, Types, Exports> as ModuleFuncLookup<FuncIdx>>::Output: FuncSignature + InvokeCall<
            WasmResolvedModule<Funcs, Types, Exports>,
            Store,
            Stack,
            Locals,
            Frames,
            Branches,
            Rest,
        >,
    <<WasmResolvedModule<Funcs, Types, Exports> as ModuleFuncLookup<FuncIdx>>::Output as FuncSignature>::Output:
        SameFuncType<<Types as Get<TypeIdx>>::Output>,
{
    type Output = <<WasmResolvedModule<Funcs, Types, Exports> as ModuleFuncLookup<FuncIdx>>::Output as InvokeCall<
        WasmResolvedModule<Funcs, Types, Exports>,
        Store,
        Stack,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
}

impl<Module, Memory, Tables, Globals, TypeIdx, TableIdx, SlotIdx, Stack, Locals, Frames, Branches, Rest>
    Eval
    for Step<
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
    <Tables as Get<TableIdx>>::Output: TableReadRef<SlotIdx>,
    Module: CallIndirectTarget<
        TypeIdx,
        <<Tables as Get<TableIdx>>::Output as TableReadRef<SlotIdx>>::Output,
        WasmStore<Memory, Tables, Globals>,
        Stack,
        Locals,
        Frames,
        Branches,
        Rest,
    >,
{
    type Output = <Module as CallIndirectTarget<
        TypeIdx,
        <<Tables as Get<TableIdx>>::Output as TableReadRef<SlotIdx>>::Output,
        WasmStore<Memory, Tables, Globals>,
        Stack,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
}

impl<
    Module,
    Store,
    Stack,
    Locals,
    Branches,
    CallerLocals,
    CallerBranches,
    Continuation,
    RestFrames,
    Rest,
> Eval
    for Step<
        WasmState<
            Module,
            Store,
            Stack,
            Locals,
            TArr<ReturnFrame<CallerLocals, CallerBranches, Continuation>, RestFrames>,
            Branches,
            TArr<OpReturn, Rest>,
        >,
    >
{
    type Output =
        WasmState<Module, Store, Stack, CallerLocals, RestFrames, CallerBranches, Continuation>;
}

impl<
    Module,
    Store,
    Stack,
    Locals,
    Branches,
    CallerLocals,
    CallerBranches,
    Continuation,
    RestFrames,
    Rest,
> Eval
    for Step<
        WasmState<
            Module,
            Store,
            Stack,
            Locals,
            TArr<ReturnFrame<CallerLocals, CallerBranches, Continuation>, RestFrames>,
            Branches,
            TArr<OpEndFunc, Rest>,
        >,
    >
{
    type Output =
        WasmState<Module, Store, Stack, CallerLocals, RestFrames, CallerBranches, Continuation>;
}

impl<Module, Store, Body, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<OpBlock<Body>, Rest>>>
where
    Body: Concat<TArr<OpEndBlock, Rest>>,
{
    type Output = WasmState<
        Module,
        Store,
        Stack,
        Locals,
        Frames,
        TArr<BranchBlock<Rest>, Branches>,
        <Body as Concat<TArr<OpEndBlock, Rest>>>::Output,
    >;
}

impl<Module, Store, Body, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<OpLoop<Body>, Rest>>>
where
    Body: Concat<TArr<OpEndLoop, Rest>>,
{
    type Output = WasmState<
        Module,
        Store,
        Stack,
        Locals,
        Frames,
        TArr<BranchLoop<<Body as Concat<TArr<OpEndLoop, Rest>>>::Output>, Branches>,
        <Body as Concat<TArr<OpEndLoop, Rest>>>::Output,
    >;
}

impl<Module, Store, Stack, Locals, Frames, RestProgram, RestBranches> Eval
    for Step<
        WasmState<
            Module,
            Store,
            Stack,
            Locals,
            Frames,
            TArr<BranchBlock<RestProgram>, RestBranches>,
            TArr<OpEndBlock, RestProgram>,
        >,
    >
{
    type Output = WasmState<Module, Store, Stack, Locals, Frames, RestBranches, RestProgram>;
}

impl<Module, Store, Stack, Locals, Frames, LoopProgram, RestProgram, RestBranches> Eval
    for Step<
        WasmState<
            Module,
            Store,
            Stack,
            Locals,
            Frames,
            TArr<BranchLoop<LoopProgram>, RestBranches>,
            TArr<OpEndLoop, RestProgram>,
        >,
    >
{
    type Output = WasmState<Module, Store, Stack, Locals, Frames, RestBranches, RestProgram>;
}

impl<Module, Store, Depth, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<OpBr<Depth>, Rest>>>
where
    Branches: ResolveBranch<Depth>,
    <Branches as ResolveBranch<Depth>>::Output: BranchJump<Module, Store, Stack, Locals, Frames>,
{
    type Output = <<Branches as ResolveBranch<Depth>>::Output as BranchJump<
        Module,
        Store,
        Stack,
        Locals,
        Frames,
    >>::Output;
}

impl<Module, Store, Depth, Cond, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI32<Cond>, Stack>,
            Locals,
            Frames,
            Branches,
            TArr<OpBrIf<Depth>, Rest>,
        >,
    >
where
    Cond: Eq<U0>,
    Branches: ResolveBranch<Depth>,
    <Branches as ResolveBranch<Depth>>::Output: BranchJump<Module, Store, Stack, Locals, Frames>,
    <Cond as Eq<U0>>::Output: ContinueIfZero<
            <Branches as ResolveBranch<Depth>>::Output,
            Module,
            Store,
            Stack,
            Locals,
            Frames,
            Branches,
            Rest,
        >,
{
    type Output = <<Cond as Eq<U0>>::Output as ContinueIfZero<
        <Branches as ResolveBranch<Depth>>::Output,
        Module,
        Store,
        Stack,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
}

impl<Module, Store, Then, Else, Cond, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI32<Cond>, Stack>,
            Locals,
            Frames,
            Branches,
            TArr<OpIf<Then, Else>, Rest>,
        >,
    >
where
    Cond: Eq<U0>,
    <Cond as Eq<U0>>::Output: IfProgram<Then, Else, Rest>,
{
    type Output = WasmState<
        Module,
        Store,
        Stack,
        Locals,
        Frames,
        Branches,
        <<Cond as Eq<U0>>::Output as IfProgram<Then, Else, Rest>>::Output,
    >;
}

impl<Module, Store, Cond, TrueValue, FalseValue, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI32<Cond>, TArr<FalseValue, TArr<TrueValue, Stack>>>,
            Locals,
            Frames,
            Branches,
            TArr<OpSelect, Rest>,
        >,
    >
where
    Cond: Eq<U0>,
    <Cond as Eq<U0>>::Output: SelectResult<TrueValue, FalseValue>,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<<<Cond as Eq<U0>>::Output as SelectResult<TrueValue, FalseValue>>::Output, Stack>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}
