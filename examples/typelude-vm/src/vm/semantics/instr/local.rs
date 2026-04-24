//! Local instruction semantics.
//!
//! Stack top is the array head. `Let` moves the top stack value into local slot
//! `0`; local index operations resolve from the local head and trap with
//! `BadLocalIndex` when missing.

use core::marker::PhantomData;

use typelude_col::{TArr, TTerm};
use typelude_std::core::{Eval, Op};

use crate::{
    opcode::local::{OpDropLocal, OpGetLocal, OpLet, OpSetLocal},
    vm::{
        protocol::trap::{BadLocalIndex, LocalUnderflow, StackUnderflow},
        semantics::{
            helpers::local_index::{FoundLocal, GetAt, MissingLocal, SetAt, SetLocalOk},
            state::VmState,
            step::{StepContinue, StepTrap},
        },
        value::Unlit,
    },
};

pub type GetAtResult<Locals, Idx, Stack, Memory, Frames, Rest> =
    ResolveLocalGet<<Locals as GetAt<Idx>>::Output, Stack, Locals, Memory, Frames, Rest>;

pub struct ResolveLocalGet<Result, Stack, Locals, Memory, Frames, Rest>(
    pub PhantomData<(Result, Stack, Locals, Memory, Frames, Rest)>,
);

impl<Value, Stack, Locals, Memory, Frames, Rest> Eval
    for ResolveLocalGet<FoundLocal<Value>, Stack, Locals, Memory, Frames, Rest>
{
    type Output = StepContinue<VmState<TArr<Value, Stack>, Locals, Memory, Frames, Rest>>;
}

impl<Idx, Stack, Locals, Memory, Frames, Rest> Eval
    for ResolveLocalGet<MissingLocal<Idx>, Stack, Locals, Memory, Frames, Rest>
{
    type Output = StepTrap<BadLocalIndex<Idx>>;
}

pub struct ResolveLocalSet<Locals, RestStack, Memory, Frames, Program, Idx>(
    pub PhantomData<(Locals, RestStack, Memory, Frames, Program, Idx)>,
);

impl<Locals, RestStack, Memory, Frames, Program, Idx> Eval
    for ResolveLocalSet<SetLocalOk<Locals>, RestStack, Memory, Frames, Program, Idx>
{
    type Output = StepContinue<VmState<RestStack, Locals, Memory, Frames, Program>>;
}

impl<Idx, RestStack, Memory, Frames, Program> Eval
    for ResolveLocalSet<MissingLocal<Idx>, RestStack, Memory, Frames, Program, Idx>
{
    type Output = StepTrap<BadLocalIndex<Idx>>;
}

impl<Locals, Memory, Frames, Program> Op<VmState<TTerm, Locals, Memory, Frames, Program>>
    for OpLet
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Value, Tail, Locals, Memory, Frames, Rest>
    Op<VmState<TArr<Value, Tail>, Locals, Memory, Frames, TArr<OpLet, Rest>>> for OpLet
{
    type Output = StepContinue<VmState<Tail, TArr<Value, Locals>, Memory, Frames, Rest>>;
}

impl<Stack, Memory, Frames, Rest>
    Op<VmState<Stack, TTerm, Memory, Frames, TArr<OpDropLocal, Rest>>> for OpDropLocal
{
    type Output = StepTrap<LocalUnderflow>;
}

impl<Stack, Head, Tail, Memory, Frames, Rest>
    Op<VmState<Stack, TArr<Head, Tail>, Memory, Frames, TArr<OpDropLocal, Rest>>> for OpDropLocal
{
    type Output = StepContinue<VmState<Stack, Tail, Memory, Frames, Rest>>;
}

impl<Idx, Stack, Locals, Memory, Frames, Rest>
    Op<VmState<Stack, Locals, Memory, Frames, TArr<OpGetLocal<Idx>, Rest>>> for OpGetLocal<Idx>
where
    Idx: Unlit,
    Locals: GetAt<<Idx as Unlit>::Output>,
    GetAtResult<Locals, <Idx as Unlit>::Output, Stack, Memory, Frames, Rest>: Eval,
{
    type Output =
        <GetAtResult<Locals, <Idx as Unlit>::Output, Stack, Memory, Frames, Rest> as Eval>::Output;
}

impl<Idx, Locals, Memory, Frames, Rest>
    Op<VmState<TTerm, Locals, Memory, Frames, TArr<OpSetLocal<Idx>, Rest>>> for OpSetLocal<Idx>
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Idx, Value, RestStack, Locals, Memory, Frames, Rest>
    Op<VmState<TArr<Value, RestStack>, Locals, Memory, Frames, TArr<OpSetLocal<Idx>, Rest>>>
    for OpSetLocal<Idx>
where
    Idx: Unlit,
    Locals: SetAt<<Idx as Unlit>::Output, Value>,
    ResolveLocalSet<
        <Locals as SetAt<<Idx as Unlit>::Output, Value>>::Output,
        RestStack,
        Memory,
        Frames,
        Rest,
        <Idx as Unlit>::Output,
    >: Eval,
{
    type Output = <ResolveLocalSet<
        <Locals as SetAt<<Idx as Unlit>::Output, Value>>::Output,
        RestStack,
        Memory,
        Frames,
        Rest,
        <Idx as Unlit>::Output,
    > as Eval>::Output;
}
