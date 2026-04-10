use core::marker::PhantomData;

use typelude_col::{TArr, TTerm};
use typelude_std::core::{Eval, Evaluate};
use typenum::U0;

use crate::{
    frame::ReturnFrame,
    helpers::{
        export::ResolveExportFunc,
        i32::U4294967295,
        instance::Instantiate,
    },
    module::{
        NoLimit, NoStart, StartFunc, WasmFuncSpace, WasmHostEnv, WasmInstance, WasmMemoryDecl,
        WasmModule, WasmResolvedModule,
    },
    opcode::OpCall,
    state::{WasmMemory, WasmState, WasmStore},
};

#[doc(hidden)]
pub struct Step<State>(PhantomData<State>);

#[doc(hidden)]
pub struct BuildProgramState<Instance, Program>(PhantomData<(Instance, Program)>);

#[doc(hidden)]
pub struct BuildInvokeState<Instance, FuncIdx, Args>(PhantomData<(Instance, FuncIdx, Args)>);

#[doc(hidden)]
pub struct BuildInvokeExportState<Instance, Name, Args>(PhantomData<(Instance, Name, Args)>);

pub struct InstantiateModule<Module, Env>(PhantomData<(Module, Env)>);
pub struct RunWasm<State>(PhantomData<State>);

pub type EmptyHostEnv = WasmHostEnv<TTerm, TTerm, TTerm, TTerm>;
pub type EmptyModule =
    WasmModule<TTerm, WasmFuncSpace<TTerm, TTerm>, WasmMemoryDecl<U0, NoLimit, TTerm>, TTerm, TTerm, TTerm, NoStart>;

pub type EmptyState = WasmState<
    WasmResolvedModule<TTerm, TTerm, TTerm>,
    WasmStore<WasmMemory<U0, U4294967295, TTerm>, TTerm, TTerm>,
    TTerm,
    TTerm,
    TTerm,
    TTerm,
    TTerm,
>;
pub type Run<State> = Evaluate<RunWasm<State>>;
pub type ModuleProgramRun<Module, Program> =
    Run<Evaluate<BuildProgramState<Evaluate<InstantiateModule<Module, EmptyHostEnv>>, Program>>>;
pub type InvokeFunc<Module, FuncIdx, Args> = InvokeFuncWithEnv<Module, EmptyHostEnv, FuncIdx, Args>;
pub type InvokeFuncWithEnv<Module, Env, FuncIdx, Args> =
    Run<Evaluate<BuildInvokeState<Evaluate<InstantiateModule<Module, Env>>, FuncIdx, Args>>>;
pub type InvokeExport<Module, Name, Args> = InvokeExportWithEnv<Module, EmptyHostEnv, Name, Args>;
pub type InvokeExportWithEnv<Module, Env, Name, Args> =
    Run<Evaluate<BuildInvokeExportState<Evaluate<InstantiateModule<Module, Env>>, Name, Args>>>;

impl<Module, Env> Eval for InstantiateModule<Module, Env>
where
    Module: Instantiate<Env>,
{
    type Output = <Module as Instantiate<Env>>::Output;
}

impl<Module, Store, Program> Eval for BuildProgramState<WasmInstance<Module, Store, NoStart>, Program> {
    type Output = WasmState<Module, Store, TTerm, TTerm, TTerm, TTerm, Program>;
}

impl<Module, Store, StartIdx, Program> Eval
    for BuildProgramState<WasmInstance<Module, Store, StartFunc<StartIdx>>, Program>
{
    type Output =
        WasmState<Module, Store, TTerm, TTerm, TTerm, TTerm, TArr<OpCall<StartIdx>, Program>>;
}

impl<Module, Store, FuncIdx, Args> Eval
    for BuildInvokeState<WasmInstance<Module, Store, NoStart>, FuncIdx, Args>
{
    type Output = WasmState<
        Module,
        Store,
        Args,
        TTerm,
        TArr<ReturnFrame<TTerm, TTerm, TTerm>, TTerm>,
        TTerm,
        TArr<OpCall<FuncIdx>, TTerm>,
    >;
}

impl<Module, Store, StartIdx, FuncIdx, Args> Eval
    for BuildInvokeState<WasmInstance<Module, Store, StartFunc<StartIdx>>, FuncIdx, Args>
{
    type Output = WasmState<
        Module,
        Store,
        Args,
        TTerm,
        TArr<ReturnFrame<TTerm, TTerm, TTerm>, TTerm>,
        TTerm,
        TArr<OpCall<StartIdx>, TArr<OpCall<FuncIdx>, TTerm>>,
    >;
}

impl<Module, Store, Name, Args, Start> Eval
    for BuildInvokeExportState<WasmInstance<Module, Store, Start>, Name, Args>
where
    Module: ResolveExportFunc<Name>,
    BuildInvokeState<WasmInstance<Module, Store, Start>, <Module as ResolveExportFunc<Name>>::Output, Args>:
        Eval,
{
    type Output = Evaluate<
        BuildInvokeState<
            WasmInstance<Module, Store, Start>,
            <Module as ResolveExportFunc<Name>>::Output,
            Args,
        >,
    >;
}

impl<Module, Store, Stack, Locals, Frames, Branches> Eval
    for RunWasm<WasmState<Module, Store, Stack, Locals, Frames, Branches, TTerm>>
{
    type Output = WasmState<Module, Store, Stack, Locals, Frames, Branches, TTerm>;
}

impl<Module, Store, Stack, Locals, Frames, Branches, Instr, Rest> Eval
    for RunWasm<WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<Instr, Rest>>>
where
    Step<WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<Instr, Rest>>>: Eval,
    RunWasm<
        Evaluate<
            Step<WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<Instr, Rest>>>,
        >,
    >: Eval,
{
    type Output = Evaluate<
        RunWasm<
            Evaluate<
                Step<
                    WasmState<
                        Module,
                        Store,
                        Stack,
                        Locals,
                        Frames,
                        Branches,
                        TArr<Instr, Rest>,
                    >,
                >,
            >,
        >,
    >;
}

pub trait StateStore {
    type Output;
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

pub trait StateTables {
    type Output;
}

pub trait StateGlobals {
    type Output;
}

pub trait StateBranches {
    type Output;
}

impl<Module, Store, Stack, Locals, Frames, Branches, Program> StateStore
    for WasmState<Module, Store, Stack, Locals, Frames, Branches, Program>
{
    type Output = Store;
}

impl<Module, Store, Stack, Locals, Frames, Branches, Program> StateStack
    for WasmState<Module, Store, Stack, Locals, Frames, Branches, Program>
{
    type Output = Stack;
}

impl<Module, Store, Stack, Locals, Frames, Branches, Program> StateLocals
    for WasmState<Module, Store, Stack, Locals, Frames, Branches, Program>
{
    type Output = Locals;
}

impl<Module, Store, Stack, Locals, Frames, Branches, Program> StateProgram
    for WasmState<Module, Store, Stack, Locals, Frames, Branches, Program>
{
    type Output = Program;
}

impl<Module, Memory, Tables, Globals, Stack, Locals, Frames, Branches, Program> StateMemory
    for WasmState<Module, WasmStore<Memory, Tables, Globals>, Stack, Locals, Frames, Branches, Program>
{
    type Output = Memory;
}

impl<Module, Memory, Tables, Globals, Stack, Locals, Frames, Branches, Program> StateTables
    for WasmState<Module, WasmStore<Memory, Tables, Globals>, Stack, Locals, Frames, Branches, Program>
{
    type Output = Tables;
}

impl<Module, Memory, Tables, Globals, Stack, Locals, Frames, Branches, Program> StateGlobals
    for WasmState<Module, WasmStore<Memory, Tables, Globals>, Stack, Locals, Frames, Branches, Program>
{
    type Output = Globals;
}

impl<Module, Store, Stack, Locals, Frames, Branches, Program> StateBranches
    for WasmState<Module, Store, Stack, Locals, Frames, Branches, Program>
{
    type Output = Branches;
}
