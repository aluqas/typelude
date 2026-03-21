//! Local instruction semantics.
//!
//! Stack top is the array head. `Let` moves the top stack value into local slot
//! `0`; local index operations resolve from the local head and trap with
//! `BadLocalIndex` when missing.

use core::marker::PhantomData;

use typelude_std::{
    core::{Eval, Evaluate, TyFn},
    std::col::array::{Array, IsList, Nil},
};

use crate::{
    opcode::local::{OpDropLocal, OpGetLocal, OpLet, OpSetLocal},
    vm::{
        protocol::trap::{BadLocalIndex, LocalUnderflow, StackUnderflow},
        semantics::{
            helpers::local_index::{FoundLocal, GetAt, MissingLocal, SetAt, SetLocalOk},
            state::VmState,
            step::{StepContinue, StepInstr, StepTrap},
        },
    },
};

pub struct LLet;
pub struct LDropLocal;
pub struct LGetLocal<Idx>(pub PhantomData<Idx>);
pub struct LSetLocal<Idx>(pub PhantomData<Idx>);

impl<Locals, Memory, Frames, Program> TyFn<VmState<Nil, Locals, Memory, Frames, Program>>
    for LLet
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Value, Tail, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Value, Tail>, Locals, Memory, Frames, Array<OpLet, Rest>>> for LLet
where
    Tail: IsList,
    Locals: IsList,
    Rest: IsList,
{
    type Output = StepContinue<VmState<Tail, Array<Value, Locals>, Memory, Frames, Rest>>;
}

impl<Stack, Memory, Frames, Rest>
    TyFn<VmState<Stack, Nil, Memory, Frames, Array<OpDropLocal, Rest>>> for LDropLocal
where
    Rest: IsList,
{
    type Output = StepTrap<LocalUnderflow>;
}

impl<Stack, Head, Tail, Memory, Frames, Rest>
    TyFn<VmState<Stack, Array<Head, Tail>, Memory, Frames, Array<OpDropLocal, Rest>>>
    for LDropLocal
where
    Tail: IsList,
    Rest: IsList,
{
    type Output = StepContinue<VmState<Stack, Tail, Memory, Frames, Rest>>;
}

pub type GetAtResult<Locals, Idx, Stack, Memory, Frames, Rest> =
    EGetAtResult<<Locals as GetAt<Idx>>::Output, Stack, Locals, Memory, Frames, Rest>;

pub struct EGetAtResult<Result, Stack, Locals, Memory, Frames, Rest>(
    pub PhantomData<(Result, Stack, Locals, Memory, Frames, Rest)>,
);

impl<Value, Stack, Locals, Memory, Frames, Rest> Eval
    for EGetAtResult<FoundLocal<Value>, Stack, Locals, Memory, Frames, Rest>
where
    Stack: IsList,
    Rest: IsList,
{
    type Output = StepContinue<VmState<Array<Value, Stack>, Locals, Memory, Frames, Rest>>;
}

impl<Idx, Stack, Locals, Memory, Frames, Rest> Eval
    for EGetAtResult<MissingLocal<Idx>, Stack, Locals, Memory, Frames, Rest>
{
    type Output = StepTrap<BadLocalIndex<Idx>>;
}

impl<Idx, Stack, Locals, Memory, Frames, Rest>
    TyFn<VmState<Stack, Locals, Memory, Frames, Array<OpGetLocal<Idx>, Rest>>> for LGetLocal<Idx>
where
    Idx: Eval,
    Stack: IsList,
    Rest: IsList,
    Locals: GetAt<Evaluate<Idx>>,
    GetAtResult<Locals, Evaluate<Idx>, Stack, Memory, Frames, Rest>: Eval,
{
    type Output = Evaluate<GetAtResult<Locals, Evaluate<Idx>, Stack, Memory, Frames, Rest>>;
}

pub struct ESetAtResult<Locals, RestStack, Memory, Frames, Program, Idx>(
    pub PhantomData<(Locals, RestStack, Memory, Frames, Program, Idx)>,
);

impl<Locals, RestStack, Memory, Frames, Program, Idx> Eval
    for ESetAtResult<SetLocalOk<Locals>, RestStack, Memory, Frames, Program, Idx>
where
    Locals: IsList,
    RestStack: IsList,
    Program: IsList,
{
    type Output = StepContinue<VmState<RestStack, Locals, Memory, Frames, Program>>;
}

impl<Idx, RestStack, Memory, Frames, Program> Eval
    for ESetAtResult<MissingLocal<Idx>, RestStack, Memory, Frames, Program, Idx>
where
    RestStack: IsList,
{
    type Output = StepTrap<BadLocalIndex<Idx>>;
}

impl<Idx, Locals, Memory, Frames, Rest>
    TyFn<VmState<Nil, Locals, Memory, Frames, Array<OpSetLocal<Idx>, Rest>>> for LSetLocal<Idx>
where
    Rest: IsList,
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Idx, Value, RestStack, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Value, RestStack>, Locals, Memory, Frames, Array<OpSetLocal<Idx>, Rest>>>
    for LSetLocal<Idx>
where
    Idx: Eval,
    RestStack: IsList,
    Rest: IsList,
    Locals: SetAt<Evaluate<Idx>, Value> + IsList,
    ESetAtResult<
        <Locals as SetAt<Evaluate<Idx>, Value>>::Output,
        RestStack,
        Memory,
        Frames,
        Rest,
        Evaluate<Idx>,
    >: Eval,
{
    type Output = Evaluate<
        ESetAtResult<
            <Locals as SetAt<Evaluate<Idx>, Value>>::Output,
            RestStack,
            Memory,
            Frames,
            Rest,
            Evaluate<Idx>,
        >,
    >;
}

impl<State> StepInstr<State> for OpLet
where
    LLet: TyFn<State>,
{
    type Output = <LLet as TyFn<State>>::Output;
}

impl<State> StepInstr<State> for OpDropLocal
where
    LDropLocal: TyFn<State>,
{
    type Output = <LDropLocal as TyFn<State>>::Output;
}

impl<Idx, State> StepInstr<State> for OpGetLocal<Idx>
where
    LGetLocal<Idx>: TyFn<State>,
{
    type Output = <LGetLocal<Idx> as TyFn<State>>::Output;
}

impl<Idx, State> StepInstr<State> for OpSetLocal<Idx>
where
    LSetLocal<Idx>: TyFn<State>,
{
    type Output = <LSetLocal<Idx> as TyFn<State>>::Output;
}
