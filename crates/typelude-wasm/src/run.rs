use core::marker::PhantomData;

use typelude_col::{TArr, TTerm};
use typelude_std::core::{Eval, Evaluate};
use typenum::U0;

use crate::state::{WasmMemory, WasmState};

#[doc(hidden)]
pub struct Step<State>(PhantomData<State>);

pub struct RunWasm<State>(PhantomData<State>);

pub type EmptyState = WasmState<TTerm, TTerm, WasmMemory<U0, TTerm>, TTerm, TTerm, TTerm>;
pub type Run<State> = Evaluate<RunWasm<State>>;
pub type ProgramRun<Program> =
    Run<WasmState<TTerm, TTerm, WasmMemory<U0, TTerm>, TTerm, TTerm, Program>>;

impl<Stack, Locals, Memory, Frames, Branches> Eval
    for RunWasm<WasmState<Stack, Locals, Memory, Frames, Branches, TTerm>>
{
    type Output = WasmState<Stack, Locals, Memory, Frames, Branches, TTerm>;
}

impl<Stack, Locals, Memory, Frames, Branches, Instr, Rest> Eval
    for RunWasm<WasmState<Stack, Locals, Memory, Frames, Branches, TArr<Instr, Rest>>>
where
    Step<WasmState<Stack, Locals, Memory, Frames, Branches, TArr<Instr, Rest>>>: Eval,
    RunWasm<Evaluate<Step<WasmState<Stack, Locals, Memory, Frames, Branches, TArr<Instr, Rest>>>>>:
        Eval,
{
    type Output = Evaluate<
        RunWasm<
            Evaluate<Step<WasmState<Stack, Locals, Memory, Frames, Branches, TArr<Instr, Rest>>>>,
        >,
    >;
}

pub trait StateStack {
    type Output;
}

pub trait StateLocals {
    type Output;
}

pub trait StateProgram {
    type Output;
}

pub trait StateMemory {
    type Output;
}

pub trait StateBranches {
    type Output;
}

impl<Stack, Locals, Memory, Frames, Branches, Program> StateStack
    for WasmState<Stack, Locals, Memory, Frames, Branches, Program>
{
    type Output = Stack;
}

impl<Stack, Locals, Memory, Frames, Branches, Program> StateLocals
    for WasmState<Stack, Locals, Memory, Frames, Branches, Program>
{
    type Output = Locals;
}

impl<Stack, Locals, Memory, Frames, Branches, Program> StateProgram
    for WasmState<Stack, Locals, Memory, Frames, Branches, Program>
{
    type Output = Program;
}

impl<Stack, Locals, Memory, Frames, Branches, Program> StateMemory
    for WasmState<Stack, Locals, Memory, Frames, Branches, Program>
{
    type Output = Memory;
}

impl<Stack, Locals, Memory, Frames, Branches, Program> StateBranches
    for WasmState<Stack, Locals, Memory, Frames, Branches, Program>
{
    type Output = Branches;
}
