//! Memory instruction semantics.
//!
//! `Load` consumes the top address. `Store` consumes the top value first and
//! the next stack slot as the target address. Invalid addresses trap without
//! committing the step.

use core::marker::PhantomData;

use typelude_col::{TArr, TTerm};
use typelude_std::core::{Eval, Op};

use crate::{
    opcode::memory::{OpLoad, OpStore},
    vm::{
        protocol::trap::{BadMemoryIndex, StackUnderflow},
        semantics::{
            helpers::memory_index::{
                FoundMemory, MemoryGet, MemorySet, MissingMemory, SetMemoryOk,
            },
            state::VmState,
            step::{StepContinue, StepTrap},
        },
        value::Unlit,
    },
};

pub struct ResolveMemoryLoad<Value, RestStack, Locals, Memory, Frames, Program, Idx>(
    pub PhantomData<(Value, RestStack, Locals, Memory, Frames, Program, Idx)>,
);

impl<Value, RestStack, Locals, Memory, Frames, Program, Idx> Eval
    for ResolveMemoryLoad<FoundMemory<Value>, RestStack, Locals, Memory, Frames, Program, Idx>
{
    type Output = StepContinue<VmState<TArr<Value, RestStack>, Locals, Memory, Frames, Program>>;
}

impl<Idx, RestStack, Locals, Memory, Frames, Program> Eval
    for ResolveMemoryLoad<MissingMemory<Idx>, RestStack, Locals, Memory, Frames, Program, Idx>
{
    type Output = StepTrap<BadMemoryIndex<Idx>>;
}

pub struct ResolveMemoryStore<Memory, RestStack, Locals, Frames, Program, Idx>(
    pub PhantomData<(Memory, RestStack, Locals, Frames, Program, Idx)>,
);

impl<Memory, RestStack, Locals, Frames, Program, Idx> Eval
    for ResolveMemoryStore<SetMemoryOk<Memory>, RestStack, Locals, Frames, Program, Idx>
{
    type Output = StepContinue<VmState<RestStack, Locals, Memory, Frames, Program>>;
}

impl<Idx, RestStack, Locals, Frames, Program> Eval
    for ResolveMemoryStore<MissingMemory<Idx>, RestStack, Locals, Frames, Program, Idx>
{
    type Output = StepTrap<BadMemoryIndex<Idx>>;
}

impl<Locals, Memory, Frames, Rest> Op<VmState<TTerm, Locals, Memory, Frames, TArr<OpLoad, Rest>>>
    for OpLoad
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Addr, RestStack, Locals, Memory, Frames, Rest>
    Op<VmState<TArr<Addr, RestStack>, Locals, Memory, Frames, TArr<OpLoad, Rest>>> for OpLoad
where
    Addr: Unlit,
    Memory: MemoryGet<<Addr as Unlit>::Output>,
    ResolveMemoryLoad<
        <Memory as MemoryGet<<Addr as Unlit>::Output>>::Output,
        RestStack,
        Locals,
        Memory,
        Frames,
        Rest,
        <Addr as Unlit>::Output,
    >: Eval,
{
    type Output = <ResolveMemoryLoad<
        <Memory as MemoryGet<<Addr as Unlit>::Output>>::Output,
        RestStack,
        Locals,
        Memory,
        Frames,
        Rest,
        <Addr as Unlit>::Output,
    > as Eval>::Output;
}

impl<Locals, Memory, Frames, Rest> Op<VmState<TTerm, Locals, Memory, Frames, TArr<OpStore, Rest>>>
    for OpStore
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Value, Locals, Memory, Frames, Rest>
    Op<VmState<TArr<Value, TTerm>, Locals, Memory, Frames, TArr<OpStore, Rest>>> for OpStore
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Value, Addr, RestStack, Locals, Memory, Frames, Rest>
    Op<VmState<TArr<Value, TArr<Addr, RestStack>>, Locals, Memory, Frames, TArr<OpStore, Rest>>>
    for OpStore
where
    Addr: Unlit,
    Memory: MemorySet<<Addr as Unlit>::Output, Value>,
    ResolveMemoryStore<
        <Memory as MemorySet<<Addr as Unlit>::Output, Value>>::Output,
        RestStack,
        Locals,
        Frames,
        Rest,
        <Addr as Unlit>::Output,
    >: Eval,
{
    type Output = <ResolveMemoryStore<
        <Memory as MemorySet<<Addr as Unlit>::Output, Value>>::Output,
        RestStack,
        Locals,
        Frames,
        Rest,
        <Addr as Unlit>::Output,
    > as Eval>::Output;
}
