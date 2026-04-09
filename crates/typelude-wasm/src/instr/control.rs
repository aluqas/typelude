use typelude_col::{Concat, TArr, TTerm};
use typelude_std::core::{Eq, Eval};
use typenum::{B0, B1, U0};

use crate::{
    frame::{BranchBlock, BranchLoop, ReturnFrame},
    func::WasmFunc,
    helpers::{
        branch_stack::{BranchJump, ContinueIfZero, ResolveBranch},
        call::{BindLocals, ModuleFuncLookup, PopArgs},
    },
    opcode::{
        OpBlock, OpBr, OpBrIf, OpCall, OpEndBlock, OpEndFunc, OpEndLoop, OpIf, OpLoop, OpReturn,
        OpSelect,
    },
    run::Step,
    state::WasmState,
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
    type Output = WasmI32<TrueValue>;
}

impl<TrueValue, FalseValue> SelectResult<TrueValue, FalseValue> for B1 {
    type Output = WasmI32<FalseValue>;
}

#[doc(hidden)]
pub trait InvokeCall<Module, Stack, Locals, Memory, Frames, Branches, Rest> {
    type Output;
}

impl<Module, ParamCount, LocalInits, FuncProgram, Stack, Locals, Memory, Frames, Branches, Rest>
    InvokeCall<Module, Stack, Locals, Memory, Frames, Branches, Rest>
    for WasmFunc<ParamCount, LocalInits, FuncProgram>
where
    Stack: PopArgs<ParamCount> + BindLocals<ParamCount, LocalInits>,
    FuncProgram: Concat<TArr<OpEndFunc, TTerm>>,
{
    type Output = WasmState<
        Module,
        <Stack as PopArgs<ParamCount>>::RemainingStack,
        <Stack as BindLocals<ParamCount, LocalInits>>::Output,
        Memory,
        TArr<ReturnFrame<Locals, Branches, Rest>, Frames>,
        TTerm,
        <FuncProgram as Concat<TArr<OpEndFunc, TTerm>>>::Output,
    >;
}

impl<Module, FuncIdx, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<
        WasmState<Module, Stack, Locals, Memory, Frames, Branches, TArr<OpCall<FuncIdx>, Rest>>,
    >
where
    Module: ModuleFuncLookup<FuncIdx>,
    <Module as ModuleFuncLookup<FuncIdx>>::Output:
        InvokeCall<Module, Stack, Locals, Memory, Frames, Branches, Rest>,
{
    type Output = <<Module as ModuleFuncLookup<FuncIdx>>::Output as InvokeCall<
        Module,
        Stack,
        Locals,
        Memory,
        Frames,
        Branches,
        Rest,
    >>::Output;
}

impl<
    Module,
    Stack,
    Locals,
    Memory,
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
            Stack,
            Locals,
            Memory,
            TArr<ReturnFrame<CallerLocals, CallerBranches, Continuation>, RestFrames>,
            Branches,
            TArr<OpReturn, Rest>,
        >,
    >
{
    type Output =
        WasmState<Module, Stack, CallerLocals, Memory, RestFrames, CallerBranches, Continuation>;
}

impl<
    Module,
    Stack,
    Locals,
    Memory,
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
            Stack,
            Locals,
            Memory,
            TArr<ReturnFrame<CallerLocals, CallerBranches, Continuation>, RestFrames>,
            Branches,
            TArr<OpEndFunc, Rest>,
        >,
    >
{
    type Output =
        WasmState<Module, Stack, CallerLocals, Memory, RestFrames, CallerBranches, Continuation>;
}

impl<Module, Body, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<WasmState<Module, Stack, Locals, Memory, Frames, Branches, TArr<OpBlock<Body>, Rest>>>
where
    Body: Concat<TArr<OpEndBlock, Rest>>,
{
    type Output = WasmState<
        Module,
        Stack,
        Locals,
        Memory,
        Frames,
        TArr<BranchBlock<Rest>, Branches>,
        <Body as Concat<TArr<OpEndBlock, Rest>>>::Output,
    >;
}

impl<Module, Body, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<WasmState<Module, Stack, Locals, Memory, Frames, Branches, TArr<OpLoop<Body>, Rest>>>
where
    Body: Concat<TArr<OpEndLoop, Rest>>,
{
    type Output = WasmState<
        Module,
        Stack,
        Locals,
        Memory,
        Frames,
        TArr<BranchLoop<<Body as Concat<TArr<OpEndLoop, Rest>>>::Output>, Branches>,
        <Body as Concat<TArr<OpEndLoop, Rest>>>::Output,
    >;
}

impl<Module, Stack, Locals, Memory, Frames, RestProgram, RestBranches> Eval
    for Step<
        WasmState<
            Module,
            Stack,
            Locals,
            Memory,
            Frames,
            TArr<BranchBlock<RestProgram>, RestBranches>,
            TArr<OpEndBlock, RestProgram>,
        >,
    >
{
    type Output = WasmState<Module, Stack, Locals, Memory, Frames, RestBranches, RestProgram>;
}

impl<Module, Stack, Locals, Memory, Frames, LoopProgram, RestProgram, RestBranches> Eval
    for Step<
        WasmState<
            Module,
            Stack,
            Locals,
            Memory,
            Frames,
            TArr<BranchLoop<LoopProgram>, RestBranches>,
            TArr<OpEndLoop, RestProgram>,
        >,
    >
{
    type Output = WasmState<Module, Stack, Locals, Memory, Frames, RestBranches, RestProgram>;
}

impl<Module, Depth, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<WasmState<Module, Stack, Locals, Memory, Frames, Branches, TArr<OpBr<Depth>, Rest>>>
where
    Branches: ResolveBranch<Depth>,
    <Branches as ResolveBranch<Depth>>::Output: BranchJump<Module, Stack, Locals, Memory, Frames>,
{
    type Output = <<Branches as ResolveBranch<Depth>>::Output as BranchJump<
        Module,
        Stack,
        Locals,
        Memory,
        Frames,
    >>::Output;
}

impl<Module, Depth, Cond, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            TArr<WasmI32<Cond>, Stack>,
            Locals,
            Memory,
            Frames,
            Branches,
            TArr<OpBrIf<Depth>, Rest>,
        >,
    >
where
    Cond: Eq<U0>,
    Branches: ResolveBranch<Depth>,
    <Branches as ResolveBranch<Depth>>::Output: BranchJump<Module, Stack, Locals, Memory, Frames>,
    <Cond as Eq<U0>>::Output: ContinueIfZero<
            <Branches as ResolveBranch<Depth>>::Output,
            Module,
            Stack,
            Locals,
            Memory,
            Frames,
            Branches,
            Rest,
        >,
{
    type Output = <<Cond as Eq<U0>>::Output as ContinueIfZero<
        <Branches as ResolveBranch<Depth>>::Output,
        Module,
        Stack,
        Locals,
        Memory,
        Frames,
        Branches,
        Rest,
    >>::Output;
}

impl<Module, Then, Else, Cond, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            TArr<WasmI32<Cond>, Stack>,
            Locals,
            Memory,
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
        Stack,
        Locals,
        Memory,
        Frames,
        Branches,
        <<Cond as Eq<U0>>::Output as IfProgram<Then, Else, Rest>>::Output,
    >;
}

impl<Module, Cond, TrueValue, FalseValue, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            TArr<WasmI32<Cond>, TArr<WasmI32<FalseValue>, TArr<WasmI32<TrueValue>, Stack>>>,
            Locals,
            Memory,
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
        TArr<<<Cond as Eq<U0>>::Output as SelectResult<TrueValue, FalseValue>>::Output, Stack>,
        Locals,
        Memory,
        Frames,
        Branches,
        Rest,
    >;
}
