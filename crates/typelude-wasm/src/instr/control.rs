use typelude_col::{Concat, TArr, TTerm};
use typelude_std::core::{Eq, Eval, Get};
use typenum::{B0, B1, U0};

use crate::{
    frame::{BranchBlock, BranchLoop, ReturnFrame},
    func::WasmFunc,
    helpers::{
        branch_stack::{BranchJump, ContinueIfZero, ResolveBranch, SelectBrTableTarget},
        call::{
            BindLocals, FuncSignature, HostCall, HostCallResult, ModuleFuncLookup, ParamTypes,
            PopArgs, ReverseList,
        },
        table::TableReadRef,
    },
    module::{WasmFuncType, WasmHostFunc, WasmResolvedModule},
    opcode::{
        OpBlock, OpBr, OpBrIf, OpBrTable, OpCall, OpCallIndirect, OpEndBlock, OpEndFunc,
        OpEndLoop, OpIf, OpLoop, OpNop, OpReturn, OpSelect,
    },
    run::Step,
    state::{WasmState, WasmStore},
    value::WasmI32,
};

impl<Module, Store, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<OpNop, Rest>>>
{
    type Output = WasmState<Module, Store, Stack, Locals, Frames, Branches, Rest>;
}

#[doc(hidden)]
pub trait IfProgram<Then, Else, Rest> {
    type Output;
}

impl<Then, Else, Rest> IfProgram<Then, Else, Rest> for B0
where
    Then: Concat<Rest>,
{
    type Output = <Then as Concat<Rest>>::Output;
}

impl<Then, Else, Rest> IfProgram<Then, Else, Rest> for B1
where
    Else: Concat<Rest>,
{
    type Output = <Else as Concat<Rest>>::Output;
}

#[doc(hidden)]
pub trait SelectResult<TrueValue, FalseValue> {
    type Output;
}

impl<TrueValue, FalseValue> SelectResult<TrueValue, FalseValue> for B0 {
    type Output = TrueValue;
}

impl<TrueValue, FalseValue> SelectResult<TrueValue, FalseValue> for B1 {
    type Output = FalseValue;
}

#[doc(hidden)]
pub trait InvokeCall<Module, Store, Stack, Locals, Frames, Branches, Rest> {
    type Output;
}

impl<Module, Store, FuncType, LocalInits, FuncProgram, Stack, Locals, Frames, Branches, Rest>
    InvokeCall<Module, Store, Stack, Locals, Frames, Branches, Rest>
    for WasmFunc<FuncType, LocalInits, FuncProgram>
where
    FuncType: ParamTypes,
    <FuncType as ParamTypes>::Output: ReverseList,
    Stack: PopArgs<<<FuncType as ParamTypes>::Output as ReverseList>::Output>
        + BindLocals<FuncType, LocalInits>,
    FuncProgram: Concat<TArr<OpEndFunc, TTerm>>,
{
    type Output = WasmState<
        Module,
        Store,
        <Stack as PopArgs<<<FuncType as ParamTypes>::Output as ReverseList>::Output>>::RemainingStack,
        <Stack as BindLocals<FuncType, LocalInits>>::Output,
        TArr<ReturnFrame<Locals, Branches, Rest>, Frames>,
        TTerm,
        <FuncProgram as Concat<TArr<OpEndFunc, TTerm>>>::Output,
    >;
}

impl<Module, Store, FuncType, Host, Stack, Locals, Frames, Branches, Rest>
    InvokeCall<Module, Store, Stack, Locals, Frames, Branches, Rest> for WasmHostFunc<FuncType, Host>
where
    FuncType: ParamTypes,
    <FuncType as ParamTypes>::Output: ReverseList,
    Stack: PopArgs<<<FuncType as ParamTypes>::Output as ReverseList>::Output>,
    Host: HostCall<
        FuncType,
        Store,
        <Stack as PopArgs<<<FuncType as ParamTypes>::Output as ReverseList>::Output>>::Params,
    >,
    <Host as HostCall<
        FuncType,
        Store,
        <Stack as PopArgs<<<FuncType as ParamTypes>::Output as ReverseList>::Output>>::Params,
    >>::Output: HostCallOutput<
        Module,
        <Stack as PopArgs<<<FuncType as ParamTypes>::Output as ReverseList>::Output>>::RemainingStack,
        Locals,
        Frames,
        Branches,
        Rest,
    >,
{
    type Output = <<Host as HostCall<
        FuncType,
        Store,
        <Stack as PopArgs<<<FuncType as ParamTypes>::Output as ReverseList>::Output>>::Params,
    >>::Output as HostCallOutput<
        Module,
        <Stack as PopArgs<<<FuncType as ParamTypes>::Output as ReverseList>::Output>>::RemainingStack,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
}

pub trait HostCallOutput<Module, Stack, Locals, Frames, Branches, Rest> {
    type Output;
}

impl<Module, Store, Results, Stack, Locals, Frames, Branches, Rest>
    HostCallOutput<Module, Stack, Locals, Frames, Branches, Rest>
    for HostCallResult<Store, Results>
where
    Results: Concat<Stack>,
{
    type Output = WasmState<
        Module,
        Store,
        <Results as Concat<Stack>>::Output,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

impl<Module, FuncIdx, Store, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<OpCall<FuncIdx>, Rest>>,
    >
where
    Module: ModuleFuncLookup<FuncIdx>,
    <Module as ModuleFuncLookup<FuncIdx>>::Output:
        InvokeCall<Module, Store, Stack, Locals, Frames, Branches, Rest>,
{
    type Output = <<Module as ModuleFuncLookup<FuncIdx>>::Output as InvokeCall<
        Module,
        Store,
        Stack,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
}

pub trait SameFuncType<Expected> {}

impl<Params, Results> SameFuncType<WasmFuncType<Params, Results>>
    for WasmFuncType<Params, Results>
{
}

pub trait CallIndirectTarget<TypeIdx, FuncIdx, Store, Stack, Locals, Frames, Branches, Rest> {
    type Output;
}

impl<Funcs, Types, Exports, TypeIdx, FuncIdx, Store, Stack, Locals, Frames, Branches, Rest>
    CallIndirectTarget<TypeIdx, FuncIdx, Store, Stack, Locals, Frames, Branches, Rest>
    for WasmResolvedModule<Funcs, Types, Exports>
where
    WasmResolvedModule<Funcs, Types, Exports>: ModuleFuncLookup<FuncIdx>,
    Types: Get<TypeIdx>,
    <WasmResolvedModule<Funcs, Types, Exports> as ModuleFuncLookup<FuncIdx>>::Output: FuncSignature + InvokeCall<
            WasmResolvedModule<Funcs, Types, Exports>,
            Store,
            Stack,
            Locals,
            Frames,
            Branches,
            Rest,
        >,
    <<WasmResolvedModule<Funcs, Types, Exports> as ModuleFuncLookup<FuncIdx>>::Output as FuncSignature>::Output:
        SameFuncType<<Types as Get<TypeIdx>>::Output>,
{
    type Output = <<WasmResolvedModule<Funcs, Types, Exports> as ModuleFuncLookup<FuncIdx>>::Output as InvokeCall<
        WasmResolvedModule<Funcs, Types, Exports>,
        Store,
        Stack,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
}

impl<
    Module,
    Memory,
    Tables,
    Globals,
    TypeIdx,
    TableIdx,
    SlotIdx,
    Stack,
    Locals,
    Frames,
    Branches,
    Rest,
> Eval
    for Step<
        WasmState<
            Module,
            WasmStore<Memory, Tables, Globals>,
            TArr<WasmI32<SlotIdx>, Stack>,
            Locals,
            Frames,
            Branches,
            TArr<OpCallIndirect<TypeIdx, TableIdx>, Rest>,
        >,
    >
where
    Tables: Get<TableIdx>,
    <Tables as Get<TableIdx>>::Output: TableReadRef<SlotIdx>,
    Module: CallIndirectTarget<
            TypeIdx,
            <<Tables as Get<TableIdx>>::Output as TableReadRef<SlotIdx>>::Output,
            WasmStore<Memory, Tables, Globals>,
            Stack,
            Locals,
            Frames,
            Branches,
            Rest,
        >,
{
    type Output = <Module as CallIndirectTarget<
        TypeIdx,
        <<Tables as Get<TableIdx>>::Output as TableReadRef<SlotIdx>>::Output,
        WasmStore<Memory, Tables, Globals>,
        Stack,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
}

impl<
    Module,
    Store,
    Stack,
    Locals,
    Branches,
    CallerLocals,
    CallerBranches,
    Continuation,
    RestFrames,
    Rest,
> Eval
    for Step<
        WasmState<
            Module,
            Store,
            Stack,
            Locals,
            TArr<ReturnFrame<CallerLocals, CallerBranches, Continuation>, RestFrames>,
            Branches,
            TArr<OpReturn, Rest>,
        >,
    >
{
    type Output =
        WasmState<Module, Store, Stack, CallerLocals, RestFrames, CallerBranches, Continuation>;
}

impl<
    Module,
    Store,
    Stack,
    Locals,
    Branches,
    CallerLocals,
    CallerBranches,
    Continuation,
    RestFrames,
    Rest,
> Eval
    for Step<
        WasmState<
            Module,
            Store,
            Stack,
            Locals,
            TArr<ReturnFrame<CallerLocals, CallerBranches, Continuation>, RestFrames>,
            Branches,
            TArr<OpEndFunc, Rest>,
        >,
    >
{
    type Output =
        WasmState<Module, Store, Stack, CallerLocals, RestFrames, CallerBranches, Continuation>;
}

impl<Module, Store, Body, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<OpBlock<Body>, Rest>>>
where
    Body: Concat<TArr<OpEndBlock, Rest>>,
{
    type Output = WasmState<
        Module,
        Store,
        Stack,
        Locals,
        Frames,
        TArr<BranchBlock<Rest>, Branches>,
        <Body as Concat<TArr<OpEndBlock, Rest>>>::Output,
    >;
}

impl<Module, Store, Body, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<OpLoop<Body>, Rest>>>
where
    Body: Concat<TArr<OpEndLoop, Rest>>,
{
    type Output = WasmState<
        Module,
        Store,
        Stack,
        Locals,
        Frames,
        TArr<BranchLoop<<Body as Concat<TArr<OpEndLoop, Rest>>>::Output>, Branches>,
        <Body as Concat<TArr<OpEndLoop, Rest>>>::Output,
    >;
}

impl<Module, Store, Stack, Locals, Frames, RestProgram, RestBranches> Eval
    for Step<
        WasmState<
            Module,
            Store,
            Stack,
            Locals,
            Frames,
            TArr<BranchBlock<RestProgram>, RestBranches>,
            TArr<OpEndBlock, RestProgram>,
        >,
    >
{
    type Output = WasmState<Module, Store, Stack, Locals, Frames, RestBranches, RestProgram>;
}

impl<Module, Store, Stack, Locals, Frames, LoopProgram, RestProgram, RestBranches> Eval
    for Step<
        WasmState<
            Module,
            Store,
            Stack,
            Locals,
            Frames,
            TArr<BranchLoop<LoopProgram>, RestBranches>,
            TArr<OpEndLoop, RestProgram>,
        >,
    >
{
    type Output = WasmState<Module, Store, Stack, Locals, Frames, RestBranches, RestProgram>;
}

impl<Module, Store, Depth, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<WasmState<Module, Store, Stack, Locals, Frames, Branches, TArr<OpBr<Depth>, Rest>>>
where
    Branches: ResolveBranch<Depth>,
    <Branches as ResolveBranch<Depth>>::Output: BranchJump<Module, Store, Stack, Locals, Frames>,
{
    type Output = <<Branches as ResolveBranch<Depth>>::Output as BranchJump<
        Module,
        Store,
        Stack,
        Locals,
        Frames,
    >>::Output;
}

impl<Module, Store, Depth, Cond, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI32<Cond>, Stack>,
            Locals,
            Frames,
            Branches,
            TArr<OpBrIf<Depth>, Rest>,
        >,
    >
where
    Cond: Eq<U0>,
    Branches: ResolveBranch<Depth>,
    <Branches as ResolveBranch<Depth>>::Output: BranchJump<Module, Store, Stack, Locals, Frames>,
    <Cond as Eq<U0>>::Output: ContinueIfZero<
            <Branches as ResolveBranch<Depth>>::Output,
            Module,
            Store,
            Stack,
            Locals,
            Frames,
            Branches,
            Rest,
        >,
{
    type Output = <<Cond as Eq<U0>>::Output as ContinueIfZero<
        <Branches as ResolveBranch<Depth>>::Output,
        Module,
        Store,
        Stack,
        Locals,
        Frames,
        Branches,
        Rest,
    >>::Output;
}

impl<Module, Store, Targets, Default, Index, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI32<Index>, Stack>,
            Locals,
            Frames,
            Branches,
            TArr<OpBrTable<Targets, Default>, Rest>,
        >,
    >
where
    Targets: SelectBrTableTarget<Default, Index>,
    Branches: ResolveBranch<<Targets as SelectBrTableTarget<Default, Index>>::Output>,
    <Branches as ResolveBranch<<Targets as SelectBrTableTarget<Default, Index>>::Output>>::Output:
        BranchJump<Module, Store, Stack, Locals, Frames>,
{
    type Output = <<Branches as ResolveBranch<
        <Targets as SelectBrTableTarget<Default, Index>>::Output,
    >>::Output as BranchJump<Module, Store, Stack, Locals, Frames>>::Output;
}

impl<Module, Store, Then, Else, Cond, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI32<Cond>, Stack>,
            Locals,
            Frames,
            Branches,
            TArr<OpIf<Then, Else>, Rest>,
        >,
    >
where
    Cond: Eq<U0>,
    <Cond as Eq<U0>>::Output: IfProgram<Then, Else, Rest>,
{
    type Output = WasmState<
        Module,
        Store,
        Stack,
        Locals,
        Frames,
        Branches,
        <<Cond as Eq<U0>>::Output as IfProgram<Then, Else, Rest>>::Output,
    >;
}

impl<Module, Store, Cond, TrueValue, FalseValue, Stack, Locals, Frames, Branches, Rest> Eval
    for Step<
        WasmState<
            Module,
            Store,
            TArr<WasmI32<Cond>, TArr<FalseValue, TArr<TrueValue, Stack>>>,
            Locals,
            Frames,
            Branches,
            TArr<OpSelect, Rest>,
        >,
    >
where
    Cond: Eq<U0>,
    <Cond as Eq<U0>>::Output: SelectResult<TrueValue, FalseValue>,
{
    type Output = WasmState<
        Module,
        Store,
        TArr<<<Cond as Eq<U0>>::Output as SelectResult<TrueValue, FalseValue>>::Output, Stack>,
        Locals,
        Frames,
        Branches,
        Rest,
    >;
}

#[cfg(test)]
mod tests {
    use crate::tests::support::*;

    type AddTwoAndReturn = Fn0<TTerm, tarr![OpI32Const<U2>, OpI32Add, OpReturn]>;
    type SimpleCallModule = Module<tarr![AddTwoAndReturn], U0>;

    type ReturnsOne =
        WasmFunc<WasmFuncType<TTerm, tarr![WasmI32Type]>, TTerm, tarr![OpI32Const<U1>, OpReturn]>;
    type ReturnsTwo =
        WasmFunc<WasmFuncType<TTerm, tarr![WasmI32Type]>, TTerm, tarr![OpI32Const<U2>, OpReturn]>;
    type IndirectTypes = tarr![WasmFuncType<TTerm, tarr![WasmI32Type]>];
    type DefaultTableDecls = tarr![WasmTableDecl<U2, U2>];
    type DefaultElemSegments =
        tarr![WasmElemSegment<U0, WasmConstExpr<tarr![OpI32Const<U0>]>, tarr![U0, U1]>];
    type DefaultTableModule = ModuleWithTables<
        tarr![ReturnsOne, ReturnsTwo],
        IndirectTypes,
        DefaultTableDecls,
        DefaultElemSegments,
    >;

    #[test]
    fn simple_call_and_return_resume_caller_continuation() {
        type Program = tarr![OpI32Const<U3>, OpCall<U0>, OpI32Add];
        type Final = ModuleProgramRun<SimpleCallModule, Program>;

        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U5>]);
    }

    #[test]
    fn if_false_branch_executes_else_program() {
        type Program = tarr![OpI32Const<U0>, OpIf<tarr![OpI32Const<U1>], tarr![OpI32Const<U2>]>];
        type Final = ModuleProgramRun<EmptyModule, Program>;

        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U2>]);
    }

    #[test]
    fn loop_and_br_if_drive_countdown_to_zero() {
        type Program = tarr![
            OpI32Const<U3>,
            OpLoop<
                tarr![
                    OpLocalTee<U0>,
                    OpI32Eqz,
                    OpBrIf<U1>,
                    OpLocalGet<U0>,
                    OpI32Const<U1>,
                    OpI32Sub,
                    OpBr<U0>
                ],
            >
        ];
        type Final = Run<InitialState<TTerm, ZeroPages, tarr![WasmI32<U0>], Program>>;

        assert_type_eq_all!(<Final as StateLocals>::Output, tarr![WasmI32<U0>]);
    }

    #[test]
    fn call_indirect_dispatches_through_default_table() {
        type Program = tarr![OpI32Const<U1>, OpCallIndirect<U0>];
        type Final = ModuleProgramRun<DefaultTableModule, Program>;

        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U2>]);
    }

    #[test]
    fn nop_leaves_stack_unchanged() {
        type Program = tarr![OpI32Const<U1>, OpNop];
        type Final = ModuleProgramRun<EmptyModule, Program>;

        assert_type_eq_all!(<Final as StateStack>::Output, tarr![WasmI32<U1>]);
    }

    #[test]
    fn br_table_selects_indexed_and_default_targets() {
        type IndexZeroProgram = tarr![
            OpBlock<tarr![
                OpBlock<tarr![OpI32Const<U0>, OpBrTable<tarr![U0, U1], U1>, OpI32Const<U9>]>,
                OpI32Const<U1>
            ]>
        ];
        type IndexZeroFinal = ModuleProgramRun<EmptyModule, IndexZeroProgram>;

        type IndexOneProgram = tarr![
            OpBlock<tarr![
                OpBlock<tarr![OpI32Const<U1>, OpBrTable<tarr![U0, U1], U1>, OpI32Const<U9>]>,
                OpI32Const<U1>
            ]>
        ];
        type IndexOneFinal = ModuleProgramRun<EmptyModule, IndexOneProgram>;

        type DefaultProgram = tarr![
            OpBlock<tarr![
                OpBlock<tarr![OpI32Const<U3>, OpBrTable<tarr![U0], U1>, OpI32Const<U9>]>,
                OpI32Const<U1>
            ]>
        ];
        type DefaultFinal = ModuleProgramRun<EmptyModule, DefaultProgram>;

        assert_type_eq_all!(<IndexZeroFinal as StateStack>::Output, tarr![WasmI32<U1>]);
        assert_type_eq_all!(<IndexOneFinal as StateStack>::Output, TTerm);
        assert_type_eq_all!(<DefaultFinal as StateStack>::Output, TTerm);
    }

    #[test]
    fn br_table_can_target_loop_back_edge() {
        type Program = tarr![
            OpI32Const<U2>,
            OpLoop<
                tarr![
                    OpLocalTee<U0>,
                    OpI32Eqz,
                    OpIf<tarr![OpI32Const<U1>], tarr![OpI32Const<U0>]>,
                    OpBrTable<tarr![U0], U1>,
                    OpLocalGet<U0>,
                    OpI32Const<U1>,
                    OpI32Sub
                ]
            >
        ];
        type Final = Run<InitialState<TTerm, ZeroPages, tarr![WasmI32<U0>], Program>>;

        assert_type_eq_all!(<Final as StateLocals>::Output, tarr![WasmI32<U0>]);
    }
}
