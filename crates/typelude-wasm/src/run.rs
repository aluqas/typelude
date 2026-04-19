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

/// モジュールを host environment に対してインスタンス化する型関数。
pub struct InstantiateModule<Module, Env>(PhantomData<(Module, Env)>);
/// success-only runtime の実行器。
///
/// trap 相当の失敗は型エラーや trait 未解決として現れることがあります。
/// `Program` が空になるまで `Step<State>` を再帰評価し、最後は生の `WasmState`
/// を返します。既存テストや成功経路の parity 確認ではこの surface を使います。
pub struct RunWasm<State>(PhantomData<State>);
/// trap-aware runtime の実行器。
///
/// `WasmDone<State>` または `WasmTrap<Reason>` を再帰的に評価します。
/// `WasmTrap` が出た時点でそれ以上 `Program` を評価せず、trap outcome
/// を伝播します。 success-only runtime と異なり、一部の runtime failure を
/// trait 未解決ではなく型値として観測できます。
pub struct RunCheckedWasm<Outcome>(PhantomData<Outcome>);

/// checked runtime における正常終了結果。
///
/// `State` は final `WasmState` です。`StateStack` などの accessor trait は
/// `WasmDone<State>` にも委譲実装されているため、成功結果は legacy runtime
/// と近い形で読めます。
pub struct WasmDone<State>(PhantomData<State>);
/// checked runtime における trap 結果。
///
/// `Reason` は trap の種類を表すマーカー型です。`WasmTrap` には `StateStack`
/// などの accessor は実装されないため、成功状態と trap
/// 状態を型レベルで分離できます。
pub struct WasmTrap<Reason>(PhantomData<Reason>);

/// `unreachable` 実行時の trap reason。
pub struct TrapUnreachable;
/// `call_indirect` が null funcref に当たったときの trap reason。
pub struct TrapCallIndirectNull;
/// `call_indirect` が table 範囲外を参照したときの trap reason。
pub struct TrapCallIndirectTableOob;
/// `call_indirect` の期待関数型と実体の関数型が一致しないときの trap reason。
pub struct TrapCallIndirectTypeMismatch;
/// memory load/store が境界外アクセスしたときの trap reason。
pub struct TrapMemoryOob;

/// checked runtime で trap しない opcode を表すマーカー。
///
/// `CheckedStep` の generic fallback は、このマーカーを持つ opcode だけを既存
/// `Step` で評価して `WasmDone` に包みます。trap-aware に扱う opcode は個別の
/// checked 実装を持ちます。
pub trait InfallibleOpcode {}

impl<State> Value for WasmDone<State> {}
impl<Reason> Value for WasmTrap<Reason> {}
impl Value for TrapUnreachable {}
impl Value for TrapCallIndirectNull {}
impl Value for TrapCallIndirectTableOob {}
impl Value for TrapCallIndirectTypeMismatch {}
impl Value for TrapMemoryOob {}

/// import を持たない空の host environment。
pub type EmptyHostEnv = WasmHostEnv<TTerm, TTerm, TTerm, TTerm>;
/// import / export / 関数を持たない最小モジュール。
pub type EmptyModule = WasmModule<
    TTerm,
    WasmFuncSpace<TTerm, TTerm>,
    WasmModuleMemory<NoMemoryDecl, TTerm>,
    WasmModuleTables<TTerm, TTerm>,
    TTerm,
    TTerm,
    NoStart,
>;

/// 何も積まれていない初期状態。
pub type EmptyState = WasmState<
    WasmResolvedModule<TTerm, TTerm, TTerm>,
    WasmStore<WasmMemory<U0, U4294967295, TTerm>, TTerm, TTerm>,
    TTerm,
    TTerm,
    TTerm,
    TTerm,
    TTerm,
>;
/// success-only runtime で `State` を最後まで実行した結果。
///
/// `State` はすでに module / store / stack / locals / frames / branches /
/// program を持つ `WasmState` である必要があります。
pub type Run<State> = Evaluate<RunWasm<State>>;
/// checked runtime で `State` を最後まで実行した結果。
///
/// 成功すれば `WasmDone<FinalState>`、trap すれば `WasmTrap<Reason>`
/// になります。
pub type RunChecked<State> = Evaluate<RunCheckedWasm<WasmDone<State>>>;
/// 空の host environment で `Program` を実行する補助 alias。
///
/// `Module` をインスタンス化し、operand stack / locals / frames / branches
/// を空にした状態から 任意の `Program` を success-only runtime で走らせます。
pub type ModuleProgramRun<Module, Program> =
    Run<Evaluate<BuildProgramState<Evaluate<InstantiateModule<Module, EmptyHostEnv>>, Program>>>;
/// 空の host environment で `Program` を checked runtime で実行する補助 alias。
///
/// start 関数がある module では、指定 `Program` の前に start
/// 呼び出しが挿入されます。
pub type ModuleProgramRunChecked<Module, Program> = RunChecked<
    Evaluate<BuildProgramState<Evaluate<InstantiateModule<Module, EmptyHostEnv>>, Program>>,
>;
/// 空の host environment で関数 index を呼び出す補助 alias。
///
/// `Args` は呼び出し引数の型リストです。実行時 stack
/// に積むため内部で反転されます。
pub type InvokeFunc<Module, FuncIdx, Args> =
    InvokeFuncWithEnv<Module, EmptyHostEnv, FuncIdx, Args>;
/// 指定 host environment で関数 index を呼び出す補助 alias。
///
/// import を持つ module では `Env` に `HostFuncBinding` などの binding
/// 列を渡します。
pub type InvokeFuncWithEnv<Module, Env, FuncIdx, Args> =
    Run<Evaluate<BuildInvokeState<Evaluate<InstantiateModule<Module, Env>>, FuncIdx, Args>>>;
/// 空の host environment で関数 index を checked runtime で呼び出す補助 alias。
///
/// checked trap を観測したい場合はこちらを使います。
pub type InvokeFuncChecked<Module, FuncIdx, Args> =
    InvokeFuncCheckedWithEnv<Module, EmptyHostEnv, FuncIdx, Args>;
/// 指定 host environment で関数 index を checked runtime で呼び出す補助 alias。
///
/// host import 解決は success-only と同じですが、実行中の selected runtime
/// failure は `WasmTrap<Reason>` として返ります。
pub type InvokeFuncCheckedWithEnv<Module, Env, FuncIdx, Args> = RunChecked<
    Evaluate<BuildInvokeState<Evaluate<InstantiateModule<Module, Env>>, FuncIdx, Args>>,
>;
/// export 名で関数を呼び出す success-only 補助 alias。
///
/// `Name` は `typelude_str::tstr!` などで作る型レベル文字列です。
pub type InvokeExport<Module, Name, Args> = InvokeExportWithEnv<Module, EmptyHostEnv, Name, Args>;
/// host environment 付きで export 名から関数を呼び出す success-only 補助
/// alias。
///
/// export 名はまず関数 index に解決され、その後 `InvokeFuncWithEnv`
/// と同じ経路で実行されます。
pub type InvokeExportWithEnv<Module, Env, Name, Args> =
    Run<Evaluate<BuildInvokeExportState<Evaluate<InstantiateModule<Module, Env>>, Name, Args>>>;
/// export 名で関数を呼び出す checked 補助 alias。
///
/// WAT 由来の module をテストするときは、この alias が trap-aware
/// な主入口になります。
pub type InvokeExportChecked<Module, Name, Args> =
    InvokeExportCheckedWithEnv<Module, EmptyHostEnv, Name, Args>;
/// host environment 付きで export 名から関数を呼び出す checked 補助 alias。
///
/// import を含む module に対して、export 名指定と checked runtime
/// を同時に使う入口です。
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

/// 実行結果から store を取り出す trait。
pub trait StateStore {
    /// 取り出された store 型。
    type Output;
}

/// 実行結果から operand stack を取り出す trait。
pub trait StateStack {
    /// 取り出された operand stack 型。
    type Output;
}

/// 実行結果から current locals を取り出す trait。
pub trait StateLocals {
    /// 取り出された locals 型。
    type Output;
}

/// 実行結果から残り program を取り出す trait。
pub trait StateProgram {
    /// 取り出された残り program 型。
    type Output;
}

/// 実行結果から memory を取り出す trait。
pub trait StateMemory {
    /// 取り出された memory 型。
    type Output;
}

/// 実行結果から tables を取り出す trait。
pub trait StateTables {
    /// 取り出された tables 型。
    type Output;
}

/// 実行結果から globals を取り出す trait。
pub trait StateGlobals {
    /// 取り出された globals 型。
    type Output;
}

/// 実行結果から branch stack を取り出す trait。
pub trait StateBranches {
    /// 取り出された branch stack 型。
    type Output;
}

/// 実行結果の exported global を名前で解決する trait。
pub trait StateExportGlobal<Name> {
    /// 解決された exported global 型。
    type Output;
}

/// 実行結果の exported table を名前で解決する trait。
pub trait StateExportTable<Name> {
    /// 解決された exported table 型。
    type Output;
}

/// 実行結果の exported memory を名前で解決する trait。
pub trait StateExportMemory<Name> {
    /// 解決された exported memory 型。
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
