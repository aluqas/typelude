use core::marker::PhantomData;

use typelude_std::{
    core::{ELit, Eval, Evaluate, TyFn},
    std::col::array::{Array, Concat, IsList, Nil},
};

use crate::{
    core::traits::{Bind, Monad, MonadError, MonadState, MonadSuspend, MonadWriter},
    opcode::{
        control::{OpCall, OpIf, OpReturn, OpWhile},
        host::OpHostCall,
        local::{OpDropLocal, OpGetLocal, OpLet, OpSetLocal},
        memory::{OpLoad, OpStore},
        numeric::{OpAdd, OpAnd, OpEq, OpGt, OpLt, OpNeq, OpNot, OpOr, OpSub},
        stack::{OpDrop, OpDup, OpPop, OpPush, OpSwap},
    },
    shared::{
        frame::ReturnFrame,
        request::HostRequest,
        trap::{
            BadLocalIndex, BadMemoryIndex, InvalidCondition, LocalUnderflow, ReturnUnderflow,
            StackUnderflow,
        },
    },
    vm::algebra::{
        effect::{
            io::{VmRequest, YieldVm},
            stack::VmFx,
            state_ops::{GetVm, PutVm, Then},
            trace::{PushTrace, VmTrace},
            trap::VmTrap,
        },
        interpret::{
            control::LowerWhile,
            helpers::{
                condition::{BranchFalse, BranchInvalid, BranchTrue, DecideBranch},
                local_index::{FoundLocal, GetAt, MissingLocal, SetAt, SetLocalOk},
                memory_index::{FoundMemory, MemoryGet, MemorySet, MissingMemory, SetMemoryOk},
                step_result::{LRunFallible, StepErr, StepOk},
            },
        },
        state::VmState,
    },
};

pub trait InterpInstr<F> {
    type Output;
}

pub trait AsValueExpr {
    type Output;
}

impl<T> AsValueExpr for ELit<T> {
    type Output = ELit<T>;
}

impl AsValueExpr for typenum::UTerm {
    type Output = ELit<typenum::UTerm>;
}

impl<N, B> AsValueExpr for typenum::UInt<N, B> {
    type Output = ELit<typenum::UInt<N, B>>;
}

impl<U> AsValueExpr for typenum::PInt<U>
where
    U: typenum::Unsigned + typenum::NonZero,
{
    type Output = ELit<typenum::PInt<U>>;
}

impl<U> AsValueExpr for typenum::NInt<U>
where
    U: typenum::Unsigned + typenum::NonZero,
{
    type Output = ELit<typenum::NInt<U>>;
}

impl AsValueExpr for typenum::Z0 {
    type Output = ELit<typenum::Z0>;
}

impl AsValueExpr for typenum::B0 {
    type Output = ELit<typenum::B0>;
}

impl AsValueExpr for typenum::B1 {
    type Output = ELit<typenum::B1>;
}

impl AsValueExpr for typelude_std::std::prim::bool::True {
    type Output = ELit<typelude_std::std::prim::bool::True>;
}

impl AsValueExpr for typelude_std::std::prim::bool::False {
    type Output = ELit<typelude_std::std::prim::bool::False>;
}

pub trait BinaryResult<Lhs, Rhs> {
    type Output;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpAdd
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::EAdd<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::EAdd<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpSub
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::ESub<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::ESub<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

#[cfg(feature = "nightly")]
impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpEq
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::EEq<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::EEq<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

#[cfg(feature = "nightly")]
impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpNeq
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::ENeq<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::ENeq<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpLt
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::ELt<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::ELt<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpGt
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::EGt<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::EGt<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpAnd
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::EAnd<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::EAnd<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpOr
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::EOr<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::EOr<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

pub struct LPushValue<V>(pub PhantomData<V>);
pub struct LDropTop<Inst>(pub PhantomData<Inst>);
pub struct LDupTop;
pub struct LSwapTop;
pub struct LUnaryNot;
pub struct LBinaryStep<Inst>(pub PhantomData<Inst>);
pub struct LLet;
pub struct LDropLocal;
pub struct LGetLocal<Idx>(pub PhantomData<Idx>);
pub struct LSetLocal<Idx>(pub PhantomData<Idx>);
pub struct LLoad;
pub struct LStore;
pub struct LIf<ThenProg, ElseProg>(pub PhantomData<(ThenProg, ElseProg)>);
pub struct LWhile<CondProg, BodyProg>(pub PhantomData<(CondProg, BodyProg)>);
pub struct LCall<TargetProg>(pub PhantomData<TargetProg>);
pub struct LReturn;

impl<V, Stack, Locals, Memory, Frames, Rest>
    TyFn<VmState<Stack, Locals, Memory, Frames, Array<OpPush<V>, Rest>>> for LPushValue<V>
where
    Rest: IsList,
    Stack: IsList,
{
    type Output = StepOk<VmState<Array<V, Stack>, Locals, Memory, Frames, Rest>>;
}

impl<Inst, Locals, Memory, Frames, Rest>
    TyFn<VmState<Nil, Locals, Memory, Frames, Array<Inst, Rest>>> for LDropTop<Inst>
where
    Rest: IsList,
{
    type Output = StepErr<StackUnderflow>;
}

impl<Inst, Head, Tail, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Head, Tail>, Locals, Memory, Frames, Array<Inst, Rest>>> for LDropTop<Inst>
where
    Tail: IsList,
    Rest: IsList,
{
    type Output = StepOk<VmState<Tail, Locals, Memory, Frames, Rest>>;
}

impl<Locals, Memory, Frames, Rest>
    TyFn<VmState<Nil, Locals, Memory, Frames, Array<OpDup, Rest>>> for LDupTop
where
    Rest: IsList,
{
    type Output = StepErr<StackUnderflow>;
}

impl<Head, Tail, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Head, Tail>, Locals, Memory, Frames, Array<OpDup, Rest>>> for LDupTop
where
    Tail: IsList,
    Rest: IsList,
{
    type Output = StepOk<VmState<Array<Head, Array<Head, Tail>>, Locals, Memory, Frames, Rest>>;
}

impl<Locals, Memory, Frames, Rest>
    TyFn<VmState<Nil, Locals, Memory, Frames, Array<OpSwap, Rest>>> for LSwapTop
where
    Rest: IsList,
{
    type Output = StepErr<StackUnderflow>;
}

impl<Head, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Head, Nil>, Locals, Memory, Frames, Array<OpSwap, Rest>>> for LSwapTop
where
    Rest: IsList,
{
    type Output = StepErr<StackUnderflow>;
}

impl<A, B, Tail, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<A, Array<B, Tail>>, Locals, Memory, Frames, Array<OpSwap, Rest>>> for LSwapTop
where
    Tail: IsList,
    Rest: IsList,
{
    type Output = StepOk<VmState<Array<B, Array<A, Tail>>, Locals, Memory, Frames, Rest>>;
}

impl<Locals, Memory, Frames, Rest>
    TyFn<VmState<Nil, Locals, Memory, Frames, Array<OpNot, Rest>>> for LUnaryNot
where
    Rest: IsList,
{
    type Output = StepErr<StackUnderflow>;
}

impl<Val, Tail, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Val, Tail>, Locals, Memory, Frames, Array<OpNot, Rest>>> for LUnaryNot
where
    Tail: IsList,
    Rest: IsList,
    Val: AsValueExpr,
    typelude_std::std::ops::ENot<<Val as AsValueExpr>::Output>: Eval,
{
    type Output = StepOk<
        VmState<
            Array<ELit<Evaluate<typelude_std::std::ops::ENot<<Val as AsValueExpr>::Output>>>, Tail>,
            Locals,
            Memory,
            Frames,
            Rest,
        >,
    >;
}

impl<Inst, Locals, Memory, Frames, Rest>
    TyFn<VmState<Nil, Locals, Memory, Frames, Array<Inst, Rest>>> for LBinaryStep<Inst>
where
    Rest: IsList,
{
    type Output = StepErr<StackUnderflow>;
}

impl<Inst, Head, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Head, Nil>, Locals, Memory, Frames, Array<Inst, Rest>>> for LBinaryStep<Inst>
where
    Rest: IsList,
{
    type Output = StepErr<StackUnderflow>;
}

impl<Inst, Lhs, Rhs, Tail, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Lhs, Array<Rhs, Tail>>, Locals, Memory, Frames, Array<Inst, Rest>>>
    for LBinaryStep<Inst>
where
    Inst: BinaryResult<Lhs, Rhs>,
    Tail: IsList,
    Rest: IsList,
{
    type Output = StepOk<
        VmState<Array<<Inst as BinaryResult<Lhs, Rhs>>::Output, Tail>, Locals, Memory, Frames, Rest>,
    >;
}

impl<Locals, Memory, Frames, Program> TyFn<VmState<Nil, Locals, Memory, Frames, Program>> for LLet {
    type Output = StepErr<StackUnderflow>;
}

impl<Value, Tail, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Value, Tail>, Locals, Memory, Frames, Array<OpLet, Rest>>> for LLet
where
    Tail: IsList,
    Locals: IsList,
    Rest: IsList,
{
    type Output = StepOk<VmState<Tail, Array<Value, Locals>, Memory, Frames, Rest>>;
}

impl<Stack, Memory, Frames, Rest>
    TyFn<VmState<Stack, Nil, Memory, Frames, Array<OpDropLocal, Rest>>> for LDropLocal
where
    Rest: IsList,
{
    type Output = StepErr<LocalUnderflow>;
}

impl<Stack, Head, Tail, Memory, Frames, Rest>
    TyFn<VmState<Stack, Array<Head, Tail>, Memory, Frames, Array<OpDropLocal, Rest>>> for LDropLocal
where
    Tail: IsList,
    Rest: IsList,
{
    type Output = StepOk<VmState<Stack, Tail, Memory, Frames, Rest>>;
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
    type Output = StepOk<VmState<Array<Value, Stack>, Locals, Memory, Frames, Rest>>;
}

impl<Idx, Stack, Locals, Memory, Frames, Rest> Eval
    for EGetAtResult<MissingLocal<Idx>, Stack, Locals, Memory, Frames, Rest>
{
    type Output = StepErr<BadLocalIndex<Idx>>;
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
    type Output = StepOk<VmState<RestStack, Locals, Memory, Frames, Program>>;
}

impl<Idx, RestStack, Memory, Frames, Program> Eval
    for ESetAtResult<MissingLocal<Idx>, RestStack, Memory, Frames, Program, Idx>
where
    RestStack: IsList,
{
    type Output = StepErr<BadLocalIndex<Idx>>;
}

impl<Idx, Locals, Memory, Frames, Rest>
    TyFn<VmState<Nil, Locals, Memory, Frames, Array<OpSetLocal<Idx>, Rest>>> for LSetLocal<Idx>
where
    Rest: IsList,
{
    type Output = StepErr<StackUnderflow>;
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

pub struct ELoadResult<Value, RestStack, Locals, Memory, Frames, Program, Idx>(
    pub PhantomData<(Value, RestStack, Locals, Memory, Frames, Program, Idx)>,
);

impl<Value, RestStack, Locals, Memory, Frames, Program, Idx> Eval
    for ELoadResult<FoundMemory<Value>, RestStack, Locals, Memory, Frames, Program, Idx>
where
    RestStack: IsList,
    Program: IsList,
{
    type Output = StepOk<VmState<Array<Value, RestStack>, Locals, Memory, Frames, Program>>;
}

impl<Idx, RestStack, Locals, Memory, Frames, Program> Eval
    for ELoadResult<MissingMemory<Idx>, RestStack, Locals, Memory, Frames, Program, Idx>
{
    type Output = StepErr<BadMemoryIndex<Idx>>;
}

impl<Locals, Memory, Frames, Rest>
    TyFn<VmState<Nil, Locals, Memory, Frames, Array<OpLoad, Rest>>> for LLoad
where
    Rest: IsList,
{
    type Output = StepErr<StackUnderflow>;
}

impl<Addr, RestStack, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Addr, RestStack>, Locals, Memory, Frames, Array<OpLoad, Rest>>> for LLoad
where
    Addr: Eval,
    Evaluate<Addr>: typenum::Unsigned,
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
    type Output = StepOk<VmState<RestStack, Locals, Memory, Frames, Program>>;
}

impl<Idx, RestStack, Locals, Frames, Program> Eval
    for EStoreResult<MissingMemory<Idx>, RestStack, Locals, Frames, Program, Idx>
{
    type Output = StepErr<BadMemoryIndex<Idx>>;
}

impl<Locals, Memory, Frames, Rest>
    TyFn<VmState<Nil, Locals, Memory, Frames, Array<OpStore, Rest>>> for LStore
where
    Rest: IsList,
{
    type Output = StepErr<StackUnderflow>;
}

impl<Value, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Value, Nil>, Locals, Memory, Frames, Array<OpStore, Rest>>> for LStore
where
    Rest: IsList,
{
    type Output = StepErr<StackUnderflow>;
}

impl<Value, Addr, RestStack, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Value, Array<Addr, RestStack>>, Locals, Memory, Frames, Array<OpStore, Rest>>>
    for LStore
where
    Addr: Eval,
    Evaluate<Addr>: typenum::Unsigned,
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

pub trait SelectBranchResult<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest> {
    type Output;
}

impl<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest>
    SelectBranchResult<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest> for BranchTrue
where
    Stack: IsList,
    Rest: IsList,
    ThenProg: Concat<Rest>,
{
    type Output = StepOk<VmState<Stack, Locals, Memory, Frames, <ThenProg as Concat<Rest>>::Output>>;
}

impl<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest>
    SelectBranchResult<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest> for BranchFalse
where
    Stack: IsList,
    Rest: IsList,
    ElseProg: Concat<Rest>,
{
    type Output = StepOk<VmState<Stack, Locals, Memory, Frames, <ElseProg as Concat<Rest>>::Output>>;
}

impl<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest>
    SelectBranchResult<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest> for BranchInvalid
{
    type Output = StepErr<InvalidCondition>;
}

impl<ThenProg, ElseProg, Locals, Memory, Frames, Rest>
    TyFn<VmState<Nil, Locals, Memory, Frames, Array<OpIf<ThenProg, ElseProg>, Rest>>>
    for LIf<ThenProg, ElseProg>
where
    Rest: IsList,
{
    type Output = StepErr<StackUnderflow>;
}

impl<ThenProg, ElseProg, Cond, Stack, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Cond, Stack>, Locals, Memory, Frames, Array<OpIf<ThenProg, ElseProg>, Rest>>>
    for LIf<ThenProg, ElseProg>
where
    Cond: DecideBranch,
    Stack: IsList,
    Rest: IsList,
    <Cond as DecideBranch>::Output:
        SelectBranchResult<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Rest>,
{
    type Output = <<Cond as DecideBranch>::Output as SelectBranchResult<
        ThenProg,
        ElseProg,
        Stack,
        Locals,
        Memory,
        Frames,
        Rest,
    >>::Output;
}

impl<CondProg, BodyProg, Stack, Locals, Memory, Frames, Rest>
    TyFn<VmState<Stack, Locals, Memory, Frames, Array<OpWhile<CondProg, BodyProg>, Rest>>>
    for LWhile<CondProg, BodyProg>
where
    Stack: IsList,
    Rest: IsList,
    OpWhile<CondProg, BodyProg>: LowerWhile<Rest>,
{
    type Output = StepOk<
        VmState<
            Stack,
            Locals,
            Memory,
            Frames,
            <OpWhile<CondProg, BodyProg> as LowerWhile<Rest>>::Output,
        >,
    >;
}

impl<TargetProg, Stack, Locals, Memory, Frames, Rest>
    TyFn<VmState<Stack, Locals, Memory, Frames, Array<OpCall<TargetProg>, Rest>>> for LCall<TargetProg>
where
    Stack: IsList,
    Locals: IsList,
    Frames: IsList,
    Rest: IsList,
{
    type Output = StepOk<
        VmState<Stack, Nil, Memory, Array<ReturnFrame<Locals, Rest>, Frames>, TargetProg>,
    >;
}

impl<Stack, Locals, Memory, Rest>
    TyFn<VmState<Stack, Locals, Memory, Nil, Array<OpReturn, Rest>>> for LReturn
where
    Rest: IsList,
{
    type Output = StepErr<ReturnUnderflow>;
}

impl<Stack, Locals, Memory, CallerLocals, Continuation, RestFrames, Rest>
    TyFn<VmState<Stack, Locals, Memory, Array<ReturnFrame<CallerLocals, Continuation>, RestFrames>, Array<OpReturn, Rest>>>
    for LReturn
where
    Stack: IsList,
    RestFrames: IsList,
    Rest: IsList,
{
    type Output = StepOk<VmState<Stack, CallerLocals, Memory, RestFrames, Continuation>>;
}

pub struct LHostCall<Sig, F, RootState>(pub PhantomData<(Sig, F, RootState)>);

impl<Sig, F, RootState, Stack, Locals, Memory, Frames, Rest>
    TyFn<VmState<Stack, Locals, Memory, Frames, Array<OpHostCall<Sig>, Rest>>>
    for LHostCall<Sig, F, RootState>
where
    F: MonadState<RootState> + MonadSuspend<VmRequest>,
    Rest: IsList,
    PutVm<F, RootState, VmState<Stack, Locals, Memory, Frames, Rest>>: Sized,
    Then<
        F,
        PutVm<F, RootState, VmState<Stack, Locals, Memory, Frames, Rest>>,
        YieldVm<F, HostRequest<Sig, Stack>>,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            F,
            PutVm<F, RootState, VmState<Stack, Locals, Memory, Frames, Rest>>,
            YieldVm<F, HostRequest<Sig, Stack>>,
        >,
    >;
}

macro_rules! impl_simple_fallible_instr {
    ($inst:ty, $func:ty) => {
        impl<RootState, Trace, Trap, Req> InterpInstr<VmFx<RootState, Trace, Trap, Req>> for $inst
        where
            VmFx<RootState, Trace, Trap, Req>: Monad
                + MonadWriter<VmTrace>
                + MonadState<RootState>
                + MonadError<VmTrap>,
            Then<
                VmFx<RootState, Trace, Trap, Req>,
                PushTrace<VmFx<RootState, Trace, Trap, Req>, $inst>,
                Bind<
                    VmFx<RootState, Trace, Trap, Req>,
                    GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
                    LRunFallible<$func, VmFx<RootState, Trace, Trap, Req>, RootState>,
                >,
            >: Eval,
        {
            type Output = Evaluate<
                Then<
                    VmFx<RootState, Trace, Trap, Req>,
                    PushTrace<VmFx<RootState, Trace, Trap, Req>, $inst>,
                    Bind<
                        VmFx<RootState, Trace, Trap, Req>,
                        GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
                        LRunFallible<$func, VmFx<RootState, Trace, Trap, Req>, RootState>,
                    >,
                >,
            >;
        }
    };
}

impl<V, RootState, Trace, Trap, Req> InterpInstr<VmFx<RootState, Trace, Trap, Req>> for OpPush<V>
where
    VmFx<RootState, Trace, Trap, Req>: Monad
        + MonadWriter<VmTrace>
        + MonadState<RootState>
        + MonadError<VmTrap>,
    Then<
        VmFx<RootState, Trace, Trap, Req>,
        PushTrace<VmFx<RootState, Trace, Trap, Req>, OpPush<V>>,
        Bind<
            VmFx<RootState, Trace, Trap, Req>,
            GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
            LRunFallible<LPushValue<V>, VmFx<RootState, Trace, Trap, Req>, RootState>,
        >,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<RootState, Trace, Trap, Req>,
            PushTrace<VmFx<RootState, Trace, Trap, Req>, OpPush<V>>,
            Bind<
                VmFx<RootState, Trace, Trap, Req>,
                GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
                LRunFallible<LPushValue<V>, VmFx<RootState, Trace, Trap, Req>, RootState>,
            >,
        >,
    >;
}

impl_simple_fallible_instr!(OpDrop, LDropTop<OpDrop>);
impl_simple_fallible_instr!(OpPop, LDropTop<OpPop>);
impl_simple_fallible_instr!(OpDup, LDupTop);
impl_simple_fallible_instr!(OpSwap, LSwapTop);
impl_simple_fallible_instr!(OpAdd, LBinaryStep<OpAdd>);
impl_simple_fallible_instr!(OpSub, LBinaryStep<OpSub>);
#[cfg(feature = "nightly")]
impl_simple_fallible_instr!(OpEq, LBinaryStep<OpEq>);
#[cfg(feature = "nightly")]
impl_simple_fallible_instr!(OpNeq, LBinaryStep<OpNeq>);
impl_simple_fallible_instr!(OpLt, LBinaryStep<OpLt>);
impl_simple_fallible_instr!(OpGt, LBinaryStep<OpGt>);
impl_simple_fallible_instr!(OpAnd, LBinaryStep<OpAnd>);
impl_simple_fallible_instr!(OpOr, LBinaryStep<OpOr>);
impl_simple_fallible_instr!(OpNot, LUnaryNot);
impl_simple_fallible_instr!(OpLet, LLet);
impl_simple_fallible_instr!(OpDropLocal, LDropLocal);
impl_simple_fallible_instr!(OpLoad, LLoad);
impl_simple_fallible_instr!(OpStore, LStore);
impl_simple_fallible_instr!(OpReturn, LReturn);

impl<Idx, RootState, Trace, Trap, Req> InterpInstr<VmFx<RootState, Trace, Trap, Req>>
    for OpGetLocal<Idx>
where
    VmFx<RootState, Trace, Trap, Req>: Monad
        + MonadWriter<VmTrace>
        + MonadState<RootState>
        + MonadError<VmTrap>,
    Then<
        VmFx<RootState, Trace, Trap, Req>,
        PushTrace<VmFx<RootState, Trace, Trap, Req>, OpGetLocal<Idx>>,
        Bind<
            VmFx<RootState, Trace, Trap, Req>,
            GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
            LRunFallible<LGetLocal<Idx>, VmFx<RootState, Trace, Trap, Req>, RootState>,
        >,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<RootState, Trace, Trap, Req>,
            PushTrace<VmFx<RootState, Trace, Trap, Req>, OpGetLocal<Idx>>,
            Bind<
                VmFx<RootState, Trace, Trap, Req>,
                GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
                LRunFallible<LGetLocal<Idx>, VmFx<RootState, Trace, Trap, Req>, RootState>,
            >,
        >,
    >;
}

impl<Idx, RootState, Trace, Trap, Req> InterpInstr<VmFx<RootState, Trace, Trap, Req>>
    for OpSetLocal<Idx>
where
    VmFx<RootState, Trace, Trap, Req>: Monad
        + MonadWriter<VmTrace>
        + MonadState<RootState>
        + MonadError<VmTrap>,
    Then<
        VmFx<RootState, Trace, Trap, Req>,
        PushTrace<VmFx<RootState, Trace, Trap, Req>, OpSetLocal<Idx>>,
        Bind<
            VmFx<RootState, Trace, Trap, Req>,
            GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
            LRunFallible<LSetLocal<Idx>, VmFx<RootState, Trace, Trap, Req>, RootState>,
        >,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<RootState, Trace, Trap, Req>,
            PushTrace<VmFx<RootState, Trace, Trap, Req>, OpSetLocal<Idx>>,
            Bind<
                VmFx<RootState, Trace, Trap, Req>,
                GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
                LRunFallible<LSetLocal<Idx>, VmFx<RootState, Trace, Trap, Req>, RootState>,
            >,
        >,
    >;
}

impl<ThenProg, ElseProg, RootState, Trace, Trap, Req> InterpInstr<VmFx<RootState, Trace, Trap, Req>>
    for OpIf<ThenProg, ElseProg>
where
    VmFx<RootState, Trace, Trap, Req>: Monad
        + MonadWriter<VmTrace>
        + MonadState<RootState>
        + MonadError<VmTrap>,
    Then<
        VmFx<RootState, Trace, Trap, Req>,
        PushTrace<VmFx<RootState, Trace, Trap, Req>, OpIf<ThenProg, ElseProg>>,
        Bind<
            VmFx<RootState, Trace, Trap, Req>,
            GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
            LRunFallible<LIf<ThenProg, ElseProg>, VmFx<RootState, Trace, Trap, Req>, RootState>,
        >,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<RootState, Trace, Trap, Req>,
            PushTrace<VmFx<RootState, Trace, Trap, Req>, OpIf<ThenProg, ElseProg>>,
            Bind<
                VmFx<RootState, Trace, Trap, Req>,
                GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
                LRunFallible<
                    LIf<ThenProg, ElseProg>,
                    VmFx<RootState, Trace, Trap, Req>,
                    RootState,
                >,
            >,
        >,
    >;
}

impl<CondProg, BodyProg, RootState, Trace, Trap, Req>
    InterpInstr<VmFx<RootState, Trace, Trap, Req>> for OpWhile<CondProg, BodyProg>
where
    VmFx<RootState, Trace, Trap, Req>: Monad
        + MonadWriter<VmTrace>
        + MonadState<RootState>
        + MonadError<VmTrap>,
    Then<
        VmFx<RootState, Trace, Trap, Req>,
        PushTrace<VmFx<RootState, Trace, Trap, Req>, OpWhile<CondProg, BodyProg>>,
        Bind<
            VmFx<RootState, Trace, Trap, Req>,
            GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
            LRunFallible<LWhile<CondProg, BodyProg>, VmFx<RootState, Trace, Trap, Req>, RootState>,
        >,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<RootState, Trace, Trap, Req>,
            PushTrace<VmFx<RootState, Trace, Trap, Req>, OpWhile<CondProg, BodyProg>>,
            Bind<
                VmFx<RootState, Trace, Trap, Req>,
                GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
                LRunFallible<
                    LWhile<CondProg, BodyProg>,
                    VmFx<RootState, Trace, Trap, Req>,
                    RootState,
                >,
            >,
        >,
    >;
}

impl<TargetProg, RootState, Trace, Trap, Req> InterpInstr<VmFx<RootState, Trace, Trap, Req>>
    for OpCall<TargetProg>
where
    VmFx<RootState, Trace, Trap, Req>: Monad
        + MonadWriter<VmTrace>
        + MonadState<RootState>
        + MonadError<VmTrap>,
    Then<
        VmFx<RootState, Trace, Trap, Req>,
        PushTrace<VmFx<RootState, Trace, Trap, Req>, OpCall<TargetProg>>,
        Bind<
            VmFx<RootState, Trace, Trap, Req>,
            GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
            LRunFallible<LCall<TargetProg>, VmFx<RootState, Trace, Trap, Req>, RootState>,
        >,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<RootState, Trace, Trap, Req>,
            PushTrace<VmFx<RootState, Trace, Trap, Req>, OpCall<TargetProg>>,
            Bind<
                VmFx<RootState, Trace, Trap, Req>,
                GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
                LRunFallible<
                    LCall<TargetProg>,
                    VmFx<RootState, Trace, Trap, Req>,
                    RootState,
                >,
            >,
        >,
    >;
}

impl<Sig, RootState, Trace, Trap, Req> InterpInstr<VmFx<RootState, Trace, Trap, Req>>
    for OpHostCall<Sig>
where
    VmFx<RootState, Trace, Trap, Req>: Monad
        + MonadWriter<VmTrace>
        + MonadState<RootState>
        + MonadSuspend<VmRequest>,
    Then<
        VmFx<RootState, Trace, Trap, Req>,
        PushTrace<VmFx<RootState, Trace, Trap, Req>, OpHostCall<Sig>>,
        Bind<
            VmFx<RootState, Trace, Trap, Req>,
            GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
            LHostCall<Sig, VmFx<RootState, Trace, Trap, Req>, RootState>,
        >,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<RootState, Trace, Trap, Req>,
            PushTrace<VmFx<RootState, Trace, Trap, Req>, OpHostCall<Sig>>,
            Bind<
                VmFx<RootState, Trace, Trap, Req>,
                GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
                LHostCall<Sig, VmFx<RootState, Trace, Trap, Req>, RootState>,
            >,
        >,
    >;
}
