use core::marker::PhantomData;

use typelude_std::{
    core::TyFn,
    std::col::array::{Array, IsList, Nil},
};

use crate::{
    opcode::stack::{OpDrop, OpDup, OpPop, OpPush, OpSwap},
    vm::{
        protocol::trap::StackUnderflow,
        semantics::{
            state::VmState,
            step::{StepContinue, StepInstr, StepTrap},
        },
    },
};

pub struct LPushValue<V>(pub PhantomData<V>);
pub struct LDropTop<Inst>(pub PhantomData<Inst>);
pub struct LDupTop;
pub struct LSwapTop;

impl<V, Stack, Locals, Memory, Frames, Rest>
    TyFn<VmState<Stack, Locals, Memory, Frames, Array<OpPush<V>, Rest>>> for LPushValue<V>
where
    Rest: IsList,
    Stack: IsList,
{
    type Output = StepContinue<VmState<Array<V, Stack>, Locals, Memory, Frames, Rest>>;
}

impl<Inst, Locals, Memory, Frames, Rest>
    TyFn<VmState<Nil, Locals, Memory, Frames, Array<Inst, Rest>>> for LDropTop<Inst>
where
    Rest: IsList,
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Inst, Head, Tail, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Head, Tail>, Locals, Memory, Frames, Array<Inst, Rest>>> for LDropTop<Inst>
where
    Tail: IsList,
    Rest: IsList,
{
    type Output = StepContinue<VmState<Tail, Locals, Memory, Frames, Rest>>;
}

impl<Locals, Memory, Frames, Rest>
    TyFn<VmState<Nil, Locals, Memory, Frames, Array<OpDup, Rest>>> for LDupTop
where
    Rest: IsList,
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Head, Tail, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Head, Tail>, Locals, Memory, Frames, Array<OpDup, Rest>>> for LDupTop
where
    Tail: IsList,
    Rest: IsList,
{
    type Output =
        StepContinue<VmState<Array<Head, Array<Head, Tail>>, Locals, Memory, Frames, Rest>>;
}

impl<Locals, Memory, Frames, Rest>
    TyFn<VmState<Nil, Locals, Memory, Frames, Array<OpSwap, Rest>>> for LSwapTop
where
    Rest: IsList,
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Head, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Head, Nil>, Locals, Memory, Frames, Array<OpSwap, Rest>>> for LSwapTop
where
    Rest: IsList,
{
    type Output = StepTrap<StackUnderflow>;
}

impl<A, B, Tail, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<A, Array<B, Tail>>, Locals, Memory, Frames, Array<OpSwap, Rest>>>
    for LSwapTop
where
    Tail: IsList,
    Rest: IsList,
{
    type Output = StepContinue<VmState<Array<B, Array<A, Tail>>, Locals, Memory, Frames, Rest>>;
}

impl<V, State> StepInstr<State> for OpPush<V>
where
    LPushValue<V>: TyFn<State>,
{
    type Output = <LPushValue<V> as TyFn<State>>::Output;
}

impl<State> StepInstr<State> for OpDrop
where
    LDropTop<OpDrop>: TyFn<State>,
{
    type Output = <LDropTop<OpDrop> as TyFn<State>>::Output;
}

impl<State> StepInstr<State> for OpPop
where
    LDropTop<OpPop>: TyFn<State>,
{
    type Output = <LDropTop<OpPop> as TyFn<State>>::Output;
}

impl<State> StepInstr<State> for OpDup
where
    LDupTop: TyFn<State>,
{
    type Output = <LDupTop as TyFn<State>>::Output;
}

impl<State> StepInstr<State> for OpSwap
where
    LSwapTop: TyFn<State>,
{
    type Output = <LSwapTop as TyFn<State>>::Output;
}
