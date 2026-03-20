//! Memory instruction semantics.
//!
//! `Load` consumes the top address. `Store` consumes the top value first and the next stack slot as
//! the target address. Invalid addresses trap without committing the step.

use core::marker::PhantomData;

use typelude_std::{
    core::{Eval, Evaluate, TyFn},
    std::col::array::{Array, IsList, Nil},
};

use crate::{
    opcode::memory::{OpLoad, OpStore},
    vm::{
        protocol::trap::{BadMemoryIndex, StackUnderflow},
        semantics::{
            helpers::memory_index::{
                FoundMemory, MemoryGet, MemorySet, MissingMemory, SetMemoryOk,
            },
            state::VmState,
            step::{StepContinue, StepInstr, StepTrap},
        },
    },
};

pub struct LLoad;
pub struct LStore;

pub struct ELoadResult<Value, RestStack, Locals, Memory, Frames, Program, Idx>(
    pub PhantomData<(Value, RestStack, Locals, Memory, Frames, Program, Idx)>,
);

impl<Value, RestStack, Locals, Memory, Frames, Program, Idx> Eval
    for ELoadResult<FoundMemory<Value>, RestStack, Locals, Memory, Frames, Program, Idx>
where
    RestStack: IsList,
    Program: IsList,
{
    type Output = StepContinue<VmState<Array<Value, RestStack>, Locals, Memory, Frames, Program>>;
}

impl<Idx, RestStack, Locals, Memory, Frames, Program> Eval
    for ELoadResult<MissingMemory<Idx>, RestStack, Locals, Memory, Frames, Program, Idx>
{
    type Output = StepTrap<BadMemoryIndex<Idx>>;
}

impl<Locals, Memory, Frames, Rest> TyFn<VmState<Nil, Locals, Memory, Frames, Array<OpLoad, Rest>>>
    for LLoad
where
    Rest: IsList,
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Addr, RestStack, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Addr, RestStack>, Locals, Memory, Frames, Array<OpLoad, Rest>>> for LLoad
where
    Addr: Eval,
    RestStack: IsList,
    Rest: IsList,
    Memory: MemoryGet<Evaluate<Addr>>,
    ELoadResult<
        <Memory as MemoryGet<Evaluate<Addr>>>::Output,
        RestStack,
        Locals,
        Memory,
        Frames,
        Rest,
        Evaluate<Addr>,
    >: Eval,
{
    type Output = Evaluate<
        ELoadResult<
            <Memory as MemoryGet<Evaluate<Addr>>>::Output,
            RestStack,
            Locals,
            Memory,
            Frames,
            Rest,
            Evaluate<Addr>,
        >,
    >;
}

pub struct EStoreResult<Memory, RestStack, Locals, Frames, Program, Idx>(
    pub PhantomData<(Memory, RestStack, Locals, Frames, Program, Idx)>,
);

impl<Memory, RestStack, Locals, Frames, Program, Idx> Eval
    for EStoreResult<SetMemoryOk<Memory>, RestStack, Locals, Frames, Program, Idx>
where
    Memory: IsList,
    RestStack: IsList,
    Program: IsList,
{
    type Output = StepContinue<VmState<RestStack, Locals, Memory, Frames, Program>>;
}

impl<Idx, RestStack, Locals, Frames, Program> Eval
    for EStoreResult<MissingMemory<Idx>, RestStack, Locals, Frames, Program, Idx>
{
    type Output = StepTrap<BadMemoryIndex<Idx>>;
}

impl<Locals, Memory, Frames, Rest> TyFn<VmState<Nil, Locals, Memory, Frames, Array<OpStore, Rest>>>
    for LStore
where
    Rest: IsList,
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Value, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Value, Nil>, Locals, Memory, Frames, Array<OpStore, Rest>>> for LStore
where
    Rest: IsList,
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Value, Addr, RestStack, Locals, Memory, Frames, Rest>
    TyFn<
        VmState<
            Array<Value, Array<Addr, RestStack>>,
            Locals,
            Memory,
            Frames,
            Array<OpStore, Rest>,
        >,
    > for LStore
where
    Addr: Eval,
    RestStack: IsList,
    Rest: IsList,
    Memory: MemorySet<Evaluate<Addr>, Value> + IsList,
    EStoreResult<
        <Memory as MemorySet<Evaluate<Addr>, Value>>::Output,
        RestStack,
        Locals,
        Frames,
        Rest,
        Evaluate<Addr>,
    >: Eval,
{
    type Output = Evaluate<
        EStoreResult<
            <Memory as MemorySet<Evaluate<Addr>, Value>>::Output,
            RestStack,
            Locals,
            Frames,
            Rest,
            Evaluate<Addr>,
        >,
    >;
}

impl<State> StepInstr<State> for OpLoad
where
    LLoad: TyFn<State>,
{
    type Output = <LLoad as TyFn<State>>::Output;
}

impl<State> StepInstr<State> for OpStore
where
    LStore: TyFn<State>,
{
    type Output = <LStore as TyFn<State>>::Output;
}
