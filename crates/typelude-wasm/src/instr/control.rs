use typelude_col::{Concat, TArr, TTerm};
use typelude_std::core::{Eq, Eval};
use typenum::{B0, B1, U0};

use crate::{
    frame::{BranchBlock, BranchLoop, ReturnFrame},
    func::WasmFunc,
    helpers::{
        branch_stack::{BranchJump, ContinueIfZero, ResolveBranch},
        call::{BindLocals, PopArgs},
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

impl<ParamCount, LocalInits, FuncProgram, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Stack,
            Locals,
            Memory,
            Frames,
            Branches,
            TArr<OpCall<WasmFunc<ParamCount, LocalInits, FuncProgram>>, Rest>,
        >,
    >
where
    Stack: PopArgs<ParamCount> + BindLocals<ParamCount, LocalInits>,
    FuncProgram: Concat<TArr<OpEndFunc, TTerm>>,
{
    type Output = WasmState<
        <Stack as PopArgs<ParamCount>>::RemainingStack,
        <Stack as BindLocals<ParamCount, LocalInits>>::Output,
        Memory,
        TArr<ReturnFrame<Locals, Branches, Rest>, Frames>,
        TTerm,
        <FuncProgram as Concat<TArr<OpEndFunc, TTerm>>>::Output,
    >;
}

impl<Stack, Locals, Memory, Branches, CallerLocals, CallerBranches, Continuation, RestFrames, Rest>
    Eval
    for Step<
        WasmState<
            Stack,
            Locals,
            Memory,
            TArr<ReturnFrame<CallerLocals, CallerBranches, Continuation>, RestFrames>,
            Branches,
            TArr<OpReturn, Rest>,
        >,
    >
{
    type Output = WasmState<Stack, CallerLocals, Memory, RestFrames, CallerBranches, Continuation>;
}

impl<Stack, Locals, Memory, Branches, CallerLocals, CallerBranches, Continuation, RestFrames, Rest>
    Eval
    for Step<
        WasmState<
            Stack,
            Locals,
            Memory,
            TArr<ReturnFrame<CallerLocals, CallerBranches, Continuation>, RestFrames>,
            Branches,
            TArr<OpEndFunc, Rest>,
        >,
    >
{
    type Output = WasmState<Stack, CallerLocals, Memory, RestFrames, CallerBranches, Continuation>;
}

impl<Body, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<WasmState<Stack, Locals, Memory, Frames, Branches, TArr<OpBlock<Body>, Rest>>>
where
    Body: Concat<TArr<OpEndBlock, Rest>>,
{
    type Output = WasmState<
        Stack,
        Locals,
        Memory,
        Frames,
        TArr<BranchBlock<Rest>, Branches>,
        <Body as Concat<TArr<OpEndBlock, Rest>>>::Output,
    >;
}

impl<Body, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<WasmState<Stack, Locals, Memory, Frames, Branches, TArr<OpLoop<Body>, Rest>>>
where
    Body: Concat<TArr<OpEndLoop, Rest>>,
{
    type Output = WasmState<
        Stack,
        Locals,
        Memory,
        Frames,
        TArr<BranchLoop<<Body as Concat<TArr<OpEndLoop, Rest>>>::Output>, Branches>,
        <Body as Concat<TArr<OpEndLoop, Rest>>>::Output,
    >;
}

impl<Stack, Locals, Memory, Frames, RestProgram, RestBranches> Eval
    for Step<
        WasmState<
            Stack,
            Locals,
            Memory,
            Frames,
            TArr<BranchBlock<RestProgram>, RestBranches>,
            TArr<OpEndBlock, RestProgram>,
        >,
    >
{
    type Output = WasmState<Stack, Locals, Memory, Frames, RestBranches, RestProgram>;
}

impl<Stack, Locals, Memory, Frames, LoopProgram, RestProgram, RestBranches> Eval
    for Step<
        WasmState<
            Stack,
            Locals,
            Memory,
            Frames,
            TArr<BranchLoop<LoopProgram>, RestBranches>,
            TArr<OpEndLoop, RestProgram>,
        >,
    >
{
    type Output = WasmState<Stack, Locals, Memory, Frames, RestBranches, RestProgram>;
}

impl<Depth, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<WasmState<Stack, Locals, Memory, Frames, Branches, TArr<OpBr<Depth>, Rest>>>
where
    Branches: ResolveBranch<Depth>,
    <Branches as ResolveBranch<Depth>>::Output: BranchJump<Stack, Locals, Memory, Frames>,
{
    type Output = <<Branches as ResolveBranch<Depth>>::Output as BranchJump<
        Stack,
        Locals,
        Memory,
        Frames,
    >>::Output;
}

impl<Depth, Cond, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
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
    <Branches as ResolveBranch<Depth>>::Output: BranchJump<Stack, Locals, Memory, Frames>,
    <Cond as Eq<U0>>::Output: ContinueIfZero<
            <Branches as ResolveBranch<Depth>>::Output,
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
        Stack,
        Locals,
        Memory,
        Frames,
        Branches,
        Rest,
    >>::Output;
}

impl<Then, Else, Cond, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
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
        Stack,
        Locals,
        Memory,
        Frames,
        Branches,
        <<Cond as Eq<U0>>::Output as IfProgram<Then, Else, Rest>>::Output,
    >;
}

impl<Cond, TrueValue, FalseValue, Stack, Locals, Memory, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
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
        TArr<<<Cond as Eq<U0>>::Output as SelectResult<TrueValue, FalseValue>>::Output, Stack>,
        Locals,
        Memory,
        Frames,
        Branches,
        Rest,
    >;
}
