//! Control instruction semantics.
//!
//! `OpIf` consumes the top condition value and dispatches to a branch program.
//! `OpWhile` records a surface-level loop step and lowers to `cond ++ if(body
//! ++ while, nil)` for core execution.

use typelude_col::{TArr, TTerm};
use typelude_std::core::{Concat, Op};

use crate::{
    opcode::control::{OpCall, OpIf, OpReturn, OpWhile},
    vm::{
        protocol::trap::{InvalidCondition, ReturnUnderflow, StackUnderflow},
        semantics::{
            frame::ReturnFrame,
            helpers::condition::{BranchFalse, BranchInvalid, BranchTrue, DecideBranch},
            lowering::LowerWhile,
            state::VmState,
            step::{StepContinue, StepTrap},
        },
    },
};

pub trait SelectBranchResult<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest> {
    type Output;
}

impl<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest>
    SelectBranchResult<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest> for BranchTrue
where
    ThenProg: Concat<Rest>,
{
    type Output =
        StepContinue<VmState<Stack, Locals, Memory, Frames, <ThenProg as Concat<Rest>>::Output>>;
}

impl<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest>
    SelectBranchResult<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest> for BranchFalse
where
    ElseProg: Concat<Rest>,
{
    type Output =
        StepContinue<VmState<Stack, Locals, Memory, Frames, <ElseProg as Concat<Rest>>::Output>>;
}

impl<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest>
    SelectBranchResult<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest> for BranchInvalid
{
    type Output = StepTrap<InvalidCondition>;
}

impl<ThenProg, ElseProg, Locals, Memory, Frames, Rest>
    Op<VmState<TTerm, Locals, Memory, Frames, TArr<OpIf<ThenProg, ElseProg>, Rest>>>
    for OpIf<ThenProg, ElseProg>
{
    type Output = StepTrap<StackUnderflow>;
}

impl<ThenProg, ElseProg, Cond, Stack, Locals, Memory, Frames, Rest>
    Op<VmState<TArr<Cond, Stack>, Locals, Memory, Frames, TArr<OpIf<ThenProg, ElseProg>, Rest>>>
    for OpIf<ThenProg, ElseProg>
where
    Cond: DecideBranch,
    <Cond as DecideBranch>::Output:
        SelectBranchResult<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest>,
{
    type Output = <<Cond as DecideBranch>::Output as SelectBranchResult<
        ThenProg,
        ElseProg,
        Stack,
        Locals,
        Memory,
        Frames,
        Rest,
    >>::Output;
}

impl<CondProg, BodyProg, Stack, Locals, Memory, Frames, Rest>
    Op<VmState<Stack, Locals, Memory, Frames, TArr<OpWhile<CondProg, BodyProg>, Rest>>>
    for OpWhile<CondProg, BodyProg>
where
    OpWhile<CondProg, BodyProg>: LowerWhile<Rest>,
{
    type Output = StepContinue<
        VmState<
            Stack,
            Locals,
            Memory,
            Frames,
            <OpWhile<CondProg, BodyProg> as LowerWhile<Rest>>::Output,
        >,
    >;
}

impl<TargetProg, Stack, Locals, Memory, Frames, Rest>
    Op<VmState<Stack, Locals, Memory, Frames, TArr<OpCall<TargetProg>, Rest>>>
    for OpCall<TargetProg>
{
    type Output = StepContinue<
        VmState<Stack, TTerm, Memory, TArr<ReturnFrame<Locals, Rest>, Frames>, TargetProg>,
    >;
}

impl<Stack, Locals, Memory, Rest> Op<VmState<Stack, Locals, Memory, TTerm, TArr<OpReturn, Rest>>>
    for OpReturn
{
    type Output = StepTrap<ReturnUnderflow>;
}

impl<Stack, Locals, Memory, CallerLocals, Continuation, RestFrames, Rest>
    Op<
        VmState<
            Stack,
            Locals,
            Memory,
            TArr<ReturnFrame<CallerLocals, Continuation>, RestFrames>,
            TArr<OpReturn, Rest>,
        >,
    > for OpReturn
{
    type Output = StepContinue<VmState<Stack, CallerLocals, Memory, RestFrames, Continuation>>;
}
