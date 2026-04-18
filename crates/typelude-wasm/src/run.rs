use core::marker::PhantomData;

use typelude_col::{TArr, TTerm};
use typelude_std::core::{Eval, Evaluate, Value};
use typenum::U0;

use crate::{
    frame::ReturnFrame,
    helpers::{
        call::ReverseList,
        export::{
            ResolveExportFunc, ResolveExportGlobal, ResolveExportMemory, ResolveExportTable,
        },
        i32::U4294967295,
        instance::Instantiate,
    },
    module::{
        NoMemoryDecl, NoStart, StartFunc, WasmFuncSpace, WasmHostEnv, WasmInstance, WasmModule,
        WasmModuleMemory, WasmModuleTables, WasmResolvedModule,
    },
    opcode::OpCall,
    state::{WasmMemory, WasmState, WasmStore},
};

#[doc(hidden)]
pub struct Step<State>(PhantomData<State>);

#[doc(hidden)]
pub struct CheckedStep<State>(PhantomData<State>);

#[doc(hidden)]
pub struct BuildProgramState<Instance, Program>(PhantomData<(Instance, Program)>);

#[doc(hidden)]
pub struct BuildInvokeState<Instance, FuncIdx, Args>(PhantomData<(Instance, FuncIdx, Args)>);

#[doc(hidden)]
pub struct BuildInvokeExportState<Instance, Name, Args>(PhantomData<(Instance, Name, Args)>);

pub struct InstantiateModule<Module, Env>(PhantomData<(Module, Env)>);
pub struct RunWasm<State>(PhantomData<State>);
pub struct RunCheckedWasm<Outcome>(PhantomData<Outcome>);

pub struct WasmDone<State>(PhantomData<State>);
pub struct WasmTrap<Reason>(PhantomData<Reason>);

pub struct TrapUnreachable;
pub struct TrapCallIndirectNull;
pub struct TrapCallIndirectTableOob;
pub struct TrapCallIndirectTypeMismatch;
pub struct TrapMemoryOob;

pub trait InfallibleOpcode {}

impl<State> Value for WasmDone<State> {}
impl<Reason> Value for WasmTrap<Reason> {}
impl Value for TrapUnreachable {}
impl Value for TrapCallIndirectNull {}
impl Value for TrapCallIndirectTableOob {}
impl Value for TrapCallIndirectTypeMismatch {}
impl Value for TrapMemoryOob {}

pub type EmptyHostEnv = WasmHostEnv<TTerm, TTerm, TTerm, TTerm>;
pub type EmptyModule = WasmModule<
    TTerm,
    WasmFuncSpace<TTerm, TTerm>,
    WasmModuleMemory<NoMemoryDecl, TTerm>,
    WasmModuleTables<TTerm, TTerm>,
    TTerm,
    TTerm,
    NoStart,
>;

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
pub type RunChecked<State> = Evaluate<RunCheckedWasm<WasmDone<State>>>;
pub type ModuleProgramRun<Module, Program> =
    Run<Evaluate<BuildProgramState<Evaluate<InstantiateModule<Module, EmptyHostEnv>>, Program>>>;
pub type ModuleProgramRunChecked<Module, Program> = RunChecked<
    Evaluate<BuildProgramState<Evaluate<InstantiateModule<Module, EmptyHostEnv>>, Program>>,
>;
pub type InvokeFunc<Module, FuncIdx, Args> =
    InvokeFuncWithEnv<Module, EmptyHostEnv, FuncIdx, Args>;
pub type InvokeFuncWithEnv<Module, Env, FuncIdx, Args> =
    Run<Evaluate<BuildInvokeState<Evaluate<InstantiateModule<Module, Env>>, FuncIdx, Args>>>;
pub type InvokeFuncChecked<Module, FuncIdx, Args> =
    InvokeFuncCheckedWithEnv<Module, EmptyHostEnv, FuncIdx, Args>;
pub type InvokeFuncCheckedWithEnv<Module, Env, FuncIdx, Args> = RunChecked<
    Evaluate<BuildInvokeState<Evaluate<InstantiateModule<Module, Env>>, FuncIdx, Args>>,
>;
pub type InvokeExport<Module, Name, Args> = InvokeExportWithEnv<Module, EmptyHostEnv, Name, Args>;
pub type InvokeExportWithEnv<Module, Env, Name, Args> =
    Run<Evaluate<BuildInvokeExportState<Evaluate<InstantiateModule<Module, Env>>, Name, Args>>>;
pub type InvokeExportChecked<Module, Name, Args> =
    InvokeExportCheckedWithEnv<Module, EmptyHostEnv, Name, Args>;
pub type InvokeExportCheckedWithEnv<Module, Env, Name, Args> = RunChecked<
    Evaluate<BuildInvokeExportState<Evaluate<InstantiateModule<Module, Env>>, Name, Args>>,
>;

impl<Module, Env> Eval for InstantiateModule<Module, Env>
where
    Module: Instantiate<Env>,
{
    type Output = <Module as Instantiate<Env>>::Output;
}

impl<Module, Store, Program> Eval
    for BuildProgramState<WasmInstance<Module, Store, NoStart>, Program>
{
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
where
    Args: ReverseList,
{
    type Output = WasmState<
        Module,
        Store,
        <Args as ReverseList>::Output,
        TTerm,
        TArr<ReturnFrame<TTerm, TTerm, TTerm>, TTerm>,
        TTerm,
        TArr<OpCall<FuncIdx>, TTerm>,
    >;
}

impl<Module, Store, StartIdx, FuncIdx, Args> Eval
    for BuildInvokeState<WasmInstance<Module, Store, StartFunc<StartIdx>>, FuncIdx, Args>
where
    Args: ReverseList,
{
    type Output = WasmState<
        Module,
        Store,
        <Args as ReverseList>::Output,
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
    BuildInvokeState<
        WasmInstance<Module, Store, Start>,
        <Module as ResolveExportFunc<Name>>::Output,
        Args,
    >: Eval,
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
                Step<WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<Instr, Rest>>>,
            >,
        >,
    >;
}

impl<Reason> Eval for RunCheckedWasm<WasmTrap<Reason>> {
    type Output = WasmTrap<Reason>;
}

impl<Module, Store, Stack, Locals, Frames, Branches> Eval
    for RunCheckedWasm<WasmDone<WasmState<Module, Store, Stack, Locals, Frames, Branches, TTerm>>>
{
    type Output = WasmDone<WasmState<Module, Store, Stack, Locals, Frames, Branches, TTerm>>;
}

impl<Module, Store, Stack, Locals, Frames, Branches, Instr, Rest> Eval
    for RunCheckedWasm<
        WasmDone<WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<Instr, Rest>>>,
    >
where
    CheckedStep<WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<Instr, Rest>>>:
        Eval,
    RunCheckedWasm<
        Evaluate<
            CheckedStep<
                WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<Instr, Rest>>,
            >,
        >,
    >: Eval,
{
    type Output = Evaluate<
        RunCheckedWasm<
            Evaluate<
                CheckedStep<
                    WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<Instr, Rest>>,
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

pub trait StateExportGlobal<Name> {
    type Output;
}

pub trait StateExportTable<Name> {
    type Output;
}

pub trait StateExportMemory<Name> {
    type Output;
}

impl<Module, Store, Stack, Locals, Frames, Branches, Program> StateStore
    for WasmState<Module, Store, Stack, Locals, Frames, Branches, Program>
{
    type Output = Store;
}

impl<State> StateStore for WasmDone<State>
where
    State: StateStore,
{
    type Output = <State as StateStore>::Output;
}

impl<Module, Store, Stack, Locals, Frames, Branches, Program> StateStack
    for WasmState<Module, Store, Stack, Locals, Frames, Branches, Program>
{
    type Output = Stack;
}

impl<State> StateStack for WasmDone<State>
where
    State: StateStack,
{
    type Output = <State as StateStack>::Output;
}

impl<Module, Store, Stack, Locals, Frames, Branches, Program> StateLocals
    for WasmState<Module, Store, Stack, Locals, Frames, Branches, Program>
{
    type Output = Locals;
}

impl<State> StateLocals for WasmDone<State>
where
    State: StateLocals,
{
    type Output = <State as StateLocals>::Output;
}

impl<Module, Store, Stack, Locals, Frames, Branches, Program> StateProgram
    for WasmState<Module, Store, Stack, Locals, Frames, Branches, Program>
{
    type Output = Program;
}

impl<State> StateProgram for WasmDone<State>
where
    State: StateProgram,
{
    type Output = <State as StateProgram>::Output;
}

impl<Module, Memory, Tables, Globals, Stack, Locals, Frames, Branches, Program> StateMemory
    for WasmState<
        Module,
        WasmStore<Memory, Tables, Globals>,
        Stack,
        Locals,
        Frames,
        Branches,
        Program,
    >
{
    type Output = Memory;
}

impl<State> StateMemory for WasmDone<State>
where
    State: StateMemory,
{
    type Output = <State as StateMemory>::Output;
}

impl<Module, Memory, Tables, Globals, Stack, Locals, Frames, Branches, Program> StateTables
    for WasmState<
        Module,
        WasmStore<Memory, Tables, Globals>,
        Stack,
        Locals,
        Frames,
        Branches,
        Program,
    >
{
    type Output = Tables;
}

impl<State> StateTables for WasmDone<State>
where
    State: StateTables,
{
    type Output = <State as StateTables>::Output;
}

impl<Module, Memory, Tables, Globals, Stack, Locals, Frames, Branches, Program> StateGlobals
    for WasmState<
        Module,
        WasmStore<Memory, Tables, Globals>,
        Stack,
        Locals,
        Frames,
        Branches,
        Program,
    >
{
    type Output = Globals;
}

impl<State> StateGlobals for WasmDone<State>
where
    State: StateGlobals,
{
    type Output = <State as StateGlobals>::Output;
}

impl<Module, Store, Stack, Locals, Frames, Branches, Program> StateBranches
    for WasmState<Module, Store, Stack, Locals, Frames, Branches, Program>
{
    type Output = Branches;
}

impl<State> StateBranches for WasmDone<State>
where
    State: StateBranches,
{
    type Output = <State as StateBranches>::Output;
}

impl<Module, Memory, Tables, Globals, Stack, Locals, Frames, Branches, Program, Name>
    StateExportGlobal<Name>
    for WasmState<
        Module,
        WasmStore<Memory, Tables, Globals>,
        Stack,
        Locals,
        Frames,
        Branches,
        Program,
    >
where
    Module: ResolveExportGlobal<Name>,
    Globals: typelude_std::core::Get<<Module as ResolveExportGlobal<Name>>::Output>,
{
    type Output = <Globals as typelude_std::core::Get<
        <Module as ResolveExportGlobal<Name>>::Output,
    >>::Output;
}

impl<State, Name> StateExportGlobal<Name> for WasmDone<State>
where
    State: StateExportGlobal<Name>,
{
    type Output = <State as StateExportGlobal<Name>>::Output;
}

impl<Module, Memory, Tables, Globals, Stack, Locals, Frames, Branches, Program, Name>
    StateExportTable<Name>
    for WasmState<
        Module,
        WasmStore<Memory, Tables, Globals>,
        Stack,
        Locals,
        Frames,
        Branches,
        Program,
    >
where
    Module: ResolveExportTable<Name>,
    Tables: typelude_std::core::Get<<Module as ResolveExportTable<Name>>::Output>,
{
    type Output =
        <Tables as typelude_std::core::Get<<Module as ResolveExportTable<Name>>::Output>>::Output;
}

impl<State, Name> StateExportTable<Name> for WasmDone<State>
where
    State: StateExportTable<Name>,
{
    type Output = <State as StateExportTable<Name>>::Output;
}

impl<Module, Memory, Tables, Globals, Stack, Locals, Frames, Branches, Program, Name>
    StateExportMemory<Name>
    for WasmState<
        Module,
        WasmStore<Memory, Tables, Globals>,
        Stack,
        Locals,
        Frames,
        Branches,
        Program,
    >
where
    Module: ResolveExportMemory<Name>,
{
    type Output = Memory;
}

impl<State, Name> StateExportMemory<Name> for WasmDone<State>
where
    State: StateExportMemory<Name>,
{
    type Output = <State as StateExportMemory<Name>>::Output;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::support::*;

    #[test]
    fn invoke_export_resolves_function_exports_by_name() {
        type Main = Fn0<TTerm, tarr![OpI32Const<U1>, OpReturn]>;
        type Module = WasmModule<
            TTerm,
            WasmFuncSpace<tarr![WasmFuncType<TTerm, tarr![WasmI32Type]>], tarr![Main]>,
            WasmModuleMemory<NoMemoryDecl, TTerm>,
            WasmModuleTables<TTerm, TTerm>,
            TTerm,
            tarr![WasmExport<typelude_str::tstr!("main"), ExportFunc<U0>>],
            NoStart,
        >;
        type Final = InvokeExport<Module, typelude_str::tstr!("main"), TTerm>;

        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U1>]);
    }
}
