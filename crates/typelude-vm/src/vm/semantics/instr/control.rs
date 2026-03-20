use core::marker::PhantomData;

use typelude_std::{
    core::TyFn,
    std::col::array::{Array, Concat, IsList, Nil},
};

use crate::{
    opcode::control::{OpCall, OpIf, OpReturn, OpWhile},
    vm::{
        protocol::trap::{InvalidCondition, ReturnUnderflow, StackUnderflow},
        semantics::{
            frame::ReturnFrame,
            helpers::condition::{BranchFalse, BranchInvalid, BranchTrue, DecideBranch},
            lowering::LowerWhile,
            state::VmState,
            step::{StepContinue, StepInstr, StepTrap},
        },
    },
};

pub struct LIf<ThenProg, ElseProg>(pub PhantomData<(ThenProg, ElseProg)>);
pub struct LWhile<CondProg, BodyProg>(pub PhantomData<(CondProg, BodyProg)>);
pub struct LCall<TargetProg>(pub PhantomData<TargetProg>);
pub struct LReturn;

pub trait SelectBranchResult<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest> {
    type Output;
}

impl<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest>
    SelectBranchResult<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest> for BranchTrue
where
    Stack: IsList,
    Rest: IsList,
    ThenProg: Concat<Rest>,
{
    type Output =
        StepContinue<VmState<Stack, Locals, Memory, Frames, <ThenProg as Concat<Rest>>::Output>>;
}

impl<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest>
    SelectBranchResult<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest> for BranchFalse
where
    Stack: IsList,
    Rest: IsList,
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
    TyFn<VmState<Nil, Locals, Memory, Frames, Array<OpIf<ThenProg, ElseProg>, Rest>>>
    for LIf<ThenProg, ElseProg>
where
    Rest: IsList,
{
    type Output = StepTrap<StackUnderflow>;
}

impl<ThenProg, ElseProg, Cond, Stack, Locals, Memory, Frames, Rest>
    TyFn<
        VmState<Array<Cond, Stack>, Locals, Memory, Frames, Array<OpIf<ThenProg, ElseProg>, Rest>>,
    > for LIf<ThenProg, ElseProg>
where
    Cond: DecideBranch,
    Stack: IsList,
    Rest: IsList,
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
    TyFn<VmState<Stack, Locals, Memory, Frames, Array<OpWhile<CondProg, BodyProg>, Rest>>>
    for LWhile<CondProg, BodyProg>
where
    Stack: IsList,
    Rest: IsList,
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
    TyFn<VmState<Stack, Locals, Memory, Frames, Array<OpCall<TargetProg>, Rest>>>
    for LCall<TargetProg>
where
    Stack: IsList,
    Locals: IsList,
    Frames: IsList,
    Rest: IsList,
{
    type Output = StepContinue<
        VmState<Stack, Nil, Memory, Array<ReturnFrame<Locals, Rest>, Frames>, TargetProg>,
    >;
}

impl<Stack, Locals, Memory, Rest> TyFn<VmState<Stack, Locals, Memory, Nil, Array<OpReturn, Rest>>>
    for LReturn
where
    Rest: IsList,
{
    type Output = StepTrap<ReturnUnderflow>;
}

impl<Stack, Locals, Memory, CallerLocals, Continuation, RestFrames, Rest>
    TyFn<
        VmState<
            Stack,
            Locals,
            Memory,
            Array<ReturnFrame<CallerLocals, Continuation>, RestFrames>,
            Array<OpReturn, Rest>,
        >,
    > for LReturn
where
    Stack: IsList,
    RestFrames: IsList,
    Rest: IsList,
{
    type Output = StepContinue<VmState<Stack, CallerLocals, Memory, RestFrames, Continuation>>;
}

impl<ThenProg, ElseProg, State> StepInstr<State> for OpIf<ThenProg, ElseProg>
where
    LIf<ThenProg, ElseProg>: TyFn<State>,
{
    type Output = <LIf<ThenProg, ElseProg> as TyFn<State>>::Output;
}

impl<CondProg, BodyProg, State> StepInstr<State> for OpWhile<CondProg, BodyProg>
where
    LWhile<CondProg, BodyProg>: TyFn<State>,
{
    type Output = <LWhile<CondProg, BodyProg> as TyFn<State>>::Output;
}

impl<TargetProg, State> StepInstr<State> for OpCall<TargetProg>
where
    LCall<TargetProg>: TyFn<State>,
{
    type Output = <LCall<TargetProg> as TyFn<State>>::Output;
}

impl<State> StepInstr<State> for OpReturn
where
    LReturn: TyFn<State>,
{
    type Output = <LReturn as TyFn<State>>::Output;
}
