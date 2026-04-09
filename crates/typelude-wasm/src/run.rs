use core::marker::PhantomData;

use typelude_col::{TArr, TTerm};
use typelude_std::core::{Eval, Evaluate};
use typenum::U0;

use crate::{
    frame::ReturnFrame,
    module::WasmModule,
    opcode::OpCall,
    state::{WasmMemory, WasmState},
};

#[doc(hidden)]
pub struct Step<State>(PhantomData<State>);

#[doc(hidden)]
pub struct BuildProgramState<Module, Program>(PhantomData<(Module, Program)>);

#[doc(hidden)]
pub struct BuildInvokeState<Module, FuncIdx, Args>(PhantomData<(Module, FuncIdx, Args)>);

pub struct RunWasm<State>(PhantomData<State>);

#[doc(hidden)]
pub type EmptyModule = WasmModule<TTerm, WasmMemory<U0, TTerm>>;

pub type EmptyState =
    WasmState<EmptyModule, TTerm, TTerm, WasmMemory<U0, TTerm>, TTerm, TTerm, TTerm>;
pub type Run<State> = Evaluate<RunWasm<State>>;
pub type ModuleProgramRun<Module, Program> = Run<Evaluate<BuildProgramState<Module, Program>>>;
pub type InvokeFunc<Module, FuncIdx, Args> =
    Run<Evaluate<BuildInvokeState<Module, FuncIdx, Args>>>;

impl<Funcs, InitialMemory, Program> Eval
    for BuildProgramState<WasmModule<Funcs, InitialMemory>, Program>
{
    type Output = WasmState<
        WasmModule<Funcs, InitialMemory>,
        TTerm,
        TTerm,
        InitialMemory,
        TTerm,
        TTerm,
        Program,
    >;
}

impl<Funcs, InitialMemory, FuncIdx, Args> Eval
    for BuildInvokeState<WasmModule<Funcs, InitialMemory>, FuncIdx, Args>
{
    type Output = WasmState<
        WasmModule<Funcs, InitialMemory>,
        Args,
        TTerm,
        InitialMemory,
        TArr<ReturnFrame<TTerm, TTerm, TTerm>, TTerm>,
        TTerm,
        TArr<OpCall<FuncIdx>, TTerm>,
    >;
}

impl<Module, Stack, Locals, Memory, Frames, Branches> Eval
    for RunWasm<WasmState<Module, Stack, Locals, Memory, Frames, Branches, TTerm>>
{
    type Output = WasmState<Module, Stack, Locals, Memory, Frames, Branches, TTerm>;
}

impl<Module, Stack, Locals, Memory, Frames, Branches, Instr, Rest> Eval
    for RunWasm<WasmState<Module, Stack, Locals, Memory, Frames, Branches, TArr<Instr, Rest>>>
where
    Step<WasmState<Module, Stack, Locals, Memory, Frames, Branches, TArr<Instr, Rest>>>: Eval,
    RunWasm<
        Evaluate<
            Step<WasmState<Module, Stack, Locals, Memory, Frames, Branches, TArr<Instr, Rest>>>,
        >,
    >: Eval,
{
    type Output = Evaluate<
        RunWasm<
            Evaluate<
                Step<
                    WasmState<Module, Stack, Locals, Memory, Frames, Branches, TArr<Instr, Rest>>,
                >,
            >,
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

impl<Module, Stack, Locals, Memory, Frames, Branches, Program> StateStack
    for WasmState<Module, Stack, Locals, Memory, Frames, Branches, Program>
{
    type Output = Stack;
}

impl<Module, Stack, Locals, Memory, Frames, Branches, Program> StateLocals
    for WasmState<Module, Stack, Locals, Memory, Frames, Branches, Program>
{
    type Output = Locals;
}

impl<Module, Stack, Locals, Memory, Frames, Branches, Program> StateProgram
    for WasmState<Module, Stack, Locals, Memory, Frames, Branches, Program>
{
    type Output = Program;
}

impl<Module, Stack, Locals, Memory, Frames, Branches, Program> StateMemory
    for WasmState<Module, Stack, Locals, Memory, Frames, Branches, Program>
{
    type Output = Memory;
}

impl<Module, Stack, Locals, Memory, Frames, Branches, Program> StateBranches
    for WasmState<Module, Stack, Locals, Memory, Frames, Branches, Program>
{
    type Output = Branches;
}
