use core::marker::PhantomData;

use typelude_std::{
    core::{Eval, Evaluate, TyFn},
    std::col::array::{Array, IsList, Nil},
};

use crate::{
    composed::{
        core::{Bind, Monad},
        vm::{
            effects::{
                BadLocalIndex, BadMemoryIndex, ModifyVm, PushTrace, StackUnderflow, Then, ThrowVm, VmFx, VmRequest,
                VmTrace, VmTrap, YieldVm, ReturnUnderflow,
            },
            state::{CallFrame, VmState},
        },
    },
    machine::instr::core::*,
};

pub trait InterpInstr<F> {
    type Output;
}

pub struct StepOk<State>(pub PhantomData<State>);
pub struct StepErr<Reason>(pub PhantomData<Reason>);

pub struct LPushValue<V>(pub PhantomData<V>);

impl<V, Stack, Locals, Memory, Frames> TyFn<VmState<Stack, Locals, Memory, Frames>> for LPushValue<V>
where
    Stack: IsList,
{
    type Output = VmState<Array<V, Stack>, Locals, Memory, Frames>;
}

pub struct LDropTop;

impl<Locals, Memory, Frames> TyFn<VmState<Nil, Locals, Memory, Frames>> for LDropTop {
    type Output = ThrowVm<VmFx<VmState<Nil, Locals, Memory, Frames>, VmTrace, VmTrap>, StackUnderflow>;
}

impl<Head, Tail, Locals, Memory, Frames> TyFn<VmState<Array<Head, Tail>, Locals, Memory, Frames>> for LDropTop
where
    Tail: IsList,
{
    type Output = VmState<Tail, Locals, Memory, Frames>;
}

pub struct LDupTop;

impl<Locals, Memory, Frames> TyFn<VmState<Nil, Locals, Memory, Frames>> for LDupTop {
    type Output = ThrowVm<VmFx<VmState<Nil, Locals, Memory, Frames>, VmTrace>, StackUnderflow>;
}

impl<Head, Tail, Locals, Memory, Frames> TyFn<VmState<Array<Head, Tail>, Locals, Memory, Frames>> for LDupTop
where
    Tail: IsList,
{
    type Output = VmState<Array<Head, Array<Head, Tail>>, Locals, Memory, Frames>;
}

pub struct LSwapTop;

impl<Locals, Memory, Frames> TyFn<VmState<Nil, Locals, Memory, Frames>> for LSwapTop {
    type Output = ThrowVm<VmFx<VmState<Nil, Locals, Memory, Frames>, VmTrace>, StackUnderflow>;
}

impl<Head, Locals, Memory, Frames> TyFn<VmState<Array<Head, Nil>, Locals, Memory, Frames>> for LSwapTop {
    type Output = ThrowVm<VmFx<VmState<Array<Head, Nil>, Locals, Memory, Frames>, VmTrace>, StackUnderflow>;
}

impl<A, B, Tail, Locals, Memory, Frames> TyFn<VmState<Array<A, Array<B, Tail>>, Locals, Memory, Frames>> for LSwapTop
where
    Tail: IsList,
{
    type Output = VmState<Array<B, Array<A, Tail>>, Locals, Memory, Frames>;
}

pub struct LUnaryNot;

impl<Locals, Memory, Frames> TyFn<VmState<Nil, Locals, Memory, Frames>> for LUnaryNot {
    type Output = ThrowVm<VmFx<VmState<Nil, Locals, Memory, Frames>, VmTrace, VmTrap>, StackUnderflow>;
}

impl<Val, Tail, Locals, Memory, Frames> TyFn<VmState<Array<Val, Tail>, Locals, Memory, Frames>> for LUnaryNot
where
    Tail: IsList,
    typelude_std::std::ops::ENot<Val>: Eval,
{
    type Output = VmState<Array<Evaluate<typelude_std::std::ops::ENot<Val>>, Tail>, Locals, Memory, Frames>;
}

pub trait BinaryResult<Lhs, Rhs> {
    type Output;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpAdd
where
    typelude_std::std::ops::EAdd<Lhs, Rhs>: Eval,
{
    type Output = Evaluate<typelude_std::std::ops::EAdd<Lhs, Rhs>>;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpSub
where
    typelude_std::std::ops::ESub<Lhs, Rhs>: Eval,
{
    type Output = Evaluate<typelude_std::std::ops::ESub<Lhs, Rhs>>;
}

#[cfg(feature = "nightly")]
impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpEq
where
    typelude_std::std::ops::EEq<Lhs, Rhs>: Eval,
{
    type Output = Evaluate<typelude_std::std::ops::EEq<Lhs, Rhs>>;
}

#[cfg(feature = "nightly")]
impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpNeq
where
    typelude_std::std::ops::ENeq<Lhs, Rhs>: Eval,
{
    type Output = Evaluate<typelude_std::std::ops::ENeq<Lhs, Rhs>>;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpLt
where
    typelude_std::std::ops::ELt<Lhs, Rhs>: Eval,
{
    type Output = Evaluate<typelude_std::std::ops::ELt<Lhs, Rhs>>;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpGt
where
    typelude_std::std::ops::EGt<Lhs, Rhs>: Eval,
{
    type Output = Evaluate<typelude_std::std::ops::EGt<Lhs, Rhs>>;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpAnd
where
    typelude_std::std::ops::EAnd<Lhs, Rhs>: Eval,
{
    type Output = Evaluate<typelude_std::std::ops::EAnd<Lhs, Rhs>>;
}

impl<Lhs, Rhs> BinaryResult<Lhs, Rhs> for OpOr
where
    typelude_std::std::ops::EOr<Lhs, Rhs>: Eval,
{
    type Output = Evaluate<typelude_std::std::ops::EOr<Lhs, Rhs>>;
}

pub struct LBinaryStep<Inst>(pub PhantomData<Inst>);

impl<Inst, Locals, Memory, Frames> TyFn<VmState<Nil, Locals, Memory, Frames>> for LBinaryStep<Inst> {
    type Output = StepErr<StackUnderflow>;
}

impl<Inst, Head, Locals, Memory, Frames> TyFn<VmState<Array<Head, Nil>, Locals, Memory, Frames>>
    for LBinaryStep<Inst>
{
    type Output = StepErr<StackUnderflow>;
}

impl<Inst, Lhs, Rhs, Tail, Locals, Memory, Frames> TyFn<VmState<Array<Lhs, Array<Rhs, Tail>>, Locals, Memory, Frames>>
    for LBinaryStep<Inst>
where
    Inst: BinaryResult<Lhs, Rhs>,
    Tail: IsList,
{
    type Output = StepOk<VmState<Array<<Inst as BinaryResult<Lhs, Rhs>>::Output, Tail>, Locals, Memory, Frames>>;
}

pub struct LApplyStep<F, InitState>(pub PhantomData<(F, InitState)>);

impl<F, InitState, NextState> TyFn<StepOk<NextState>> for LApplyStep<F, InitState>
where
    F: crate::composed::core::MonadState<InitState>,
    crate::composed::vm::effects::PutVm<F, InitState, NextState>: Sized,
{
    type Output = crate::composed::vm::effects::PutVm<F, InitState, NextState>;
}

impl<F, InitState, Reason> TyFn<StepErr<Reason>> for LApplyStep<F, InitState>
where
    F: crate::composed::core::MonadError<VmTrap>,
{
    type Output = ThrowVm<F, Reason>;
}

pub struct LRunFallible<Func, F, InitState>(pub PhantomData<(Func, F, InitState)>);

impl<Func, F, InitState, CurrentState> TyFn<CurrentState> for LRunFallible<Func, F, InitState>
where
    Func: TyFn<CurrentState>,
    LApplyStep<F, InitState>: TyFn<<Func as TyFn<CurrentState>>::Output>,
{
    type Output = <LApplyStep<F, InitState> as TyFn<<Func as TyFn<CurrentState>>::Output>>::Output;
}

pub trait GetAt<Idx> {
    type Output;
}

pub struct FoundLocal<Value>(pub PhantomData<Value>);
pub struct MissingLocal<Idx>(pub PhantomData<Idx>);

impl<Idx> GetAt<Idx> for Nil {
    type Output = MissingLocal<Idx>;
}

impl<Head, Tail> GetAt<typenum::U0> for Array<Head, Tail>
where
    Tail: IsList,
{
    type Output = FoundLocal<Head>;
}

impl<Head, Tail, N, B> GetAt<typenum::UInt<N, B>> for Array<Head, Tail>
where
    Tail: GetAt<typenum::Sub1<typenum::UInt<N, B>>> + IsList,
    typenum::UInt<N, B>: core::ops::Sub<typenum::B1>,
    typenum::Sub1<typenum::UInt<N, B>>: typenum::Unsigned,
{
    type Output = <Tail as GetAt<typenum::Sub1<typenum::UInt<N, B>>>>::Output;
}

pub trait SetAt<Idx, Value> {
    type Output;
}

pub struct SetLocalOk<Locals>(pub PhantomData<Locals>);

impl<Idx, Value> SetAt<Idx, Value> for Nil {
    type Output = MissingLocal<Idx>;
}

impl<Head, Tail, Value> SetAt<typenum::U0, Value> for Array<Head, Tail>
where
    Tail: IsList,
{
    type Output = SetLocalOk<Array<Value, Tail>>;
}

impl<Head, Tail, N, B, Value> SetAt<typenum::UInt<N, B>, Value> for Array<Head, Tail>
where
    Tail: SetAt<typenum::Sub1<typenum::UInt<N, B>>, Value> + IsList,
    typenum::UInt<N, B>: core::ops::Sub<typenum::B1>,
    typenum::Sub1<typenum::UInt<N, B>>: typenum::Unsigned,
    <Tail as SetAt<typenum::Sub1<typenum::UInt<N, B>>, Value>>::Output: SetAtTailResult,
{
    type Output =
        <<Tail as SetAt<typenum::Sub1<typenum::UInt<N, B>>, Value>>::Output as SetAtTailResult>::WithHead<Head>;
}

pub trait SetAtTailResult {
    type WithHead<Head>;
}

impl<Locals> SetAtTailResult for SetLocalOk<Locals>
where
    Locals: IsList,
{
    type WithHead<Head> = SetLocalOk<Array<Head, Locals>>;
}

impl<Idx> SetAtTailResult for MissingLocal<Idx> {
    type WithHead<Head> = MissingLocal<Idx>;
}

pub struct LLet;

impl<Locals, Memory, Frames> TyFn<VmState<Nil, Locals, Memory, Frames>> for LLet {
    type Output = ThrowVm<VmFx<VmState<Nil, Locals, Memory, Frames>, VmTrace, VmTrap>, StackUnderflow>;
}

impl<Value, Tail, Locals, Memory, Frames> TyFn<VmState<Array<Value, Tail>, Locals, Memory, Frames>> for LLet
where
    Tail: IsList,
    Locals: IsList,
{
    type Output = VmState<Tail, Array<Value, Locals>, Memory, Frames>;
}

pub struct LDropLocal;

impl<Stack, Memory, Frames> TyFn<VmState<Stack, Nil, Memory, Frames>> for LDropLocal {
    type Output = ThrowVm<VmFx<VmState<Stack, Nil, Memory, Frames>, VmTrace, VmTrap>, StackUnderflow>;
}

impl<Stack, Head, Tail, Memory, Frames> TyFn<VmState<Stack, Array<Head, Tail>, Memory, Frames>> for LDropLocal
where
    Tail: IsList,
{
    type Output = VmState<Stack, Tail, Memory, Frames>;
}

pub struct LGetLocal<Idx>(pub PhantomData<Idx>);

impl<Idx, Stack, Locals, Memory, Frames> TyFn<VmState<Stack, Locals, Memory, Frames>> for LGetLocal<Idx>
where
    Stack: IsList,
    Locals: GetAt<Idx>,
    GetAtResult<Locals, Idx, Stack, Memory, Frames>: Eval,
{
    type Output = Evaluate<GetAtResult<Locals, Idx, Stack, Memory, Frames>>;
}

pub type GetAtResult<Locals, Idx, Stack, Memory, Frames> =
    EGetAtResult<<Locals as GetAt<Idx>>::Output, Stack, Locals, Memory, Frames>;

pub struct EGetAtResult<Result, Stack, Locals, Memory, Frames>(pub PhantomData<(Result, Stack, Locals, Memory, Frames)>);

impl<Value, Stack, Locals, Memory, Frames> Eval for EGetAtResult<FoundLocal<Value>, Stack, Locals, Memory, Frames>
where
    Stack: IsList,
{
    type Output = VmState<Array<Value, Stack>, Locals, Memory, Frames>;
}

impl<Idx, Stack, Locals, Memory, Frames> Eval
    for EGetAtResult<MissingLocal<Idx>, Stack, Locals, Memory, Frames>
{
    type Output = ThrowVm<VmFx<VmState<Stack, Locals, Memory, Frames>, VmTrace, VmTrap>, BadLocalIndex<Idx>>;
}

pub struct LSetLocal<Idx>(pub PhantomData<Idx>);

impl<Idx, Value, Rest, Locals, Memory, Frames> TyFn<VmState<Array<Value, Rest>, Locals, Memory, Frames>>
    for LSetLocal<Idx>
where
    Rest: IsList,
    Locals: SetAt<Idx, Value> + IsList,
    ESetAtResult<<Locals as SetAt<Idx, Value>>::Output, Rest, Memory, Frames, Idx>: Eval,
{
    type Output = Evaluate<ESetAtResult<<Locals as SetAt<Idx, Value>>::Output, Rest, Memory, Frames, Idx>>;
}

impl<Idx, Locals, Memory, Frames> TyFn<VmState<Nil, Locals, Memory, Frames>> for LSetLocal<Idx> {
    type Output = ThrowVm<VmFx<VmState<Nil, Locals, Memory, Frames>, VmTrace, VmTrap>, StackUnderflow>;
}

pub struct ESetAtResult<Locals, Rest, Memory, Frames, Idx>(pub PhantomData<(Locals, Rest, Memory, Frames, Idx)>);

impl<Locals, Rest, Memory, Frames, Idx> Eval for ESetAtResult<SetLocalOk<Locals>, Rest, Memory, Frames, Idx>
where
    Locals: IsList,
    Rest: IsList,
{
    type Output = VmState<Rest, Locals, Memory, Frames>;
}

impl<Idx, Rest, Memory, Frames> Eval for ESetAtResult<MissingLocal<Idx>, Rest, Memory, Frames, Idx>
where
    Rest: IsList,
{
    type Output = ThrowVm<VmFx<VmState<Rest, Nil, Memory, Frames>, VmTrace, VmTrap>, BadLocalIndex<Idx>>;
}

pub trait MemoryGet<Idx> {
    type Output;
}

pub struct FoundMemory<Value>(pub PhantomData<Value>);
pub struct MissingMemory<Idx>(pub PhantomData<Idx>);

impl<Idx> MemoryGet<Idx> for Nil {
    type Output = MissingMemory<Idx>;
}

impl<Head, Tail> MemoryGet<typenum::U0> for Array<Head, Tail>
where
    Tail: IsList,
{
    type Output = FoundMemory<Head>;
}

impl<Head, Tail, N, B> MemoryGet<typenum::UInt<N, B>> for Array<Head, Tail>
where
    Tail: MemoryGet<typenum::Sub1<typenum::UInt<N, B>>> + IsList,
    typenum::UInt<N, B>: core::ops::Sub<typenum::B1>,
    typenum::Sub1<typenum::UInt<N, B>>: typenum::Unsigned,
{
    type Output = <Tail as MemoryGet<typenum::Sub1<typenum::UInt<N, B>>>>::Output;
}

pub trait MemorySet<Idx, Value> {
    type Output;
}

pub struct SetMemoryOk<Memory>(pub PhantomData<Memory>);

impl<Idx, Value> MemorySet<Idx, Value> for Nil {
    type Output = MissingMemory<Idx>;
}

impl<Head, Tail, Value> MemorySet<typenum::U0, Value> for Array<Head, Tail>
where
    Tail: IsList,
{
    type Output = SetMemoryOk<Array<Value, Tail>>;
}

impl<Head, Tail, N, B, Value> MemorySet<typenum::UInt<N, B>, Value> for Array<Head, Tail>
where
    Tail: MemorySet<typenum::Sub1<typenum::UInt<N, B>>, Value> + IsList,
    typenum::UInt<N, B>: core::ops::Sub<typenum::B1>,
    typenum::Sub1<typenum::UInt<N, B>>: typenum::Unsigned,
    <Tail as MemorySet<typenum::Sub1<typenum::UInt<N, B>>, Value>>::Output: MemorySetTailResult,
{
    type Output = <<Tail as MemorySet<typenum::Sub1<typenum::UInt<N, B>>, Value>>::Output as MemorySetTailResult>::WithHead<Head>;
}

pub trait MemorySetTailResult {
    type WithHead<Head>;
}

impl<Memory> MemorySetTailResult for SetMemoryOk<Memory>
where
    Memory: IsList,
{
    type WithHead<Head> = SetMemoryOk<Array<Head, Memory>>;
}

impl<Idx> MemorySetTailResult for MissingMemory<Idx> {
    type WithHead<Head> = MissingMemory<Idx>;
}

pub struct LLoad;

impl<Locals, Memory, Frames> TyFn<VmState<Nil, Locals, Memory, Frames>> for LLoad {
    type Output = ThrowVm<VmFx<VmState<Nil, Locals, Memory, Frames>, VmTrace, VmTrap>, StackUnderflow>;
}

impl<Addr, Rest, Locals, Memory, Frames> TyFn<VmState<Array<Addr, Rest>, Locals, Memory, Frames>> for LLoad
where
    Rest: IsList,
    Addr: Eval,
    Evaluate<Addr>: typenum::Unsigned,
    Memory: MemoryGet<Evaluate<Addr>>,
    ELoadResult<<Memory as MemoryGet<Evaluate<Addr>>>::Output, Rest, Locals, Memory, Frames, Evaluate<Addr>>: Eval,
{
    type Output = Evaluate<ELoadResult<<Memory as MemoryGet<Evaluate<Addr>>>::Output, Rest, Locals, Memory, Frames, Evaluate<Addr>>>;
}

pub struct ELoadResult<Value, Rest, Locals, Memory, Frames, Idx>(pub PhantomData<(Value, Rest, Locals, Memory, Frames, Idx)>);

impl<Value, Rest, Locals, Memory, Frames, Idx> Eval for ELoadResult<FoundMemory<Value>, Rest, Locals, Memory, Frames, Idx>
where
    Rest: IsList,
{
    type Output = VmState<Array<Value, Rest>, Locals, Memory, Frames>;
}

impl<Idx, Rest, Locals, Memory, Frames> Eval
    for ELoadResult<MissingMemory<Idx>, Rest, Locals, Memory, Frames, Idx>
{
    type Output = ThrowVm<VmFx<VmState<Rest, Locals, Memory, Frames>, VmTrace, VmTrap>, BadMemoryIndex<Idx>>;
}

pub struct LStore;

impl<Locals, Memory, Frames> TyFn<VmState<Nil, Locals, Memory, Frames>> for LStore {
    type Output = ThrowVm<VmFx<VmState<Nil, Locals, Memory, Frames>, VmTrace, VmTrap>, StackUnderflow>;
}

impl<Value, Locals, Memory, Frames> TyFn<VmState<Array<Value, Nil>, Locals, Memory, Frames>> for LStore {
    type Output = ThrowVm<VmFx<VmState<Array<Value, Nil>, Locals, Memory, Frames>, VmTrace, VmTrap>, StackUnderflow>;
}

impl<Value, Addr, Rest, Locals, Memory, Frames> TyFn<VmState<Array<Value, Array<Addr, Rest>>, Locals, Memory, Frames>>
    for LStore
where
    Rest: IsList,
    Addr: Eval,
    Evaluate<Addr>: typenum::Unsigned,
    Memory: MemorySet<Evaluate<Addr>, Value> + IsList,
    EStoreResult<<Memory as MemorySet<Evaluate<Addr>, Value>>::Output, Rest, Locals, Frames, Evaluate<Addr>>: Eval,
{
    type Output =
        Evaluate<EStoreResult<<Memory as MemorySet<Evaluate<Addr>, Value>>::Output, Rest, Locals, Frames, Evaluate<Addr>>>;
}

pub struct EStoreResult<Memory, Rest, Locals, Frames, Idx>(pub PhantomData<(Memory, Rest, Locals, Frames, Idx)>);

impl<Memory, Rest, Locals, Frames, Idx> Eval for EStoreResult<SetMemoryOk<Memory>, Rest, Locals, Frames, Idx>
where
    Memory: IsList,
    Rest: IsList,
{
    type Output = VmState<Rest, Locals, Memory, Frames>;
}

impl<Idx, Rest, Locals, Frames> Eval for EStoreResult<MissingMemory<Idx>, Rest, Locals, Frames, Idx>
where
    Rest: IsList,
{
    type Output = ThrowVm<VmFx<VmState<Rest, Locals, Nil, Frames>, VmTrace, VmTrap>, BadMemoryIndex<Idx>>;
}

pub struct LCall<TargetProg>(pub PhantomData<TargetProg>);

impl<TargetProg, Stack, Locals, Memory, Frames> TyFn<VmState<Stack, Locals, Memory, Frames>> for LCall<TargetProg>
where
    Frames: IsList,
{
    type Output = VmState<Stack, Nil, Memory, Array<CallFrame<Locals, TargetProg>, Frames>>;
}

pub struct LReturn;

impl<Stack, Locals, Memory> TyFn<VmState<Stack, Locals, Memory, Nil>> for LReturn {
    type Output = ThrowVm<VmFx<VmState<Stack, Locals, Memory, Nil>, VmTrace, VmTrap>, ReturnUnderflow>;
}

impl<Stack, Locals, Memory, CallerLocals, Continuation, RestFrames> TyFn<
    VmState<Stack, Locals, Memory, Array<CallFrame<CallerLocals, Continuation>, RestFrames>>,
> for LReturn
where
    RestFrames: IsList,
{
    type Output = VmState<Stack, CallerLocals, Memory, RestFrames>;
}

#[derive(Debug)]
pub struct HostRequest<Sig>(pub PhantomData<Sig>);

pub type ComposeInstr<F, Instr, Func> = Then<F, PushTrace<F, Instr>, ModifyVm<F, crate::composed::vm::state::VmState<(), (), (), ()>, Func>>;

macro_rules! impl_stateful_instr {
    ($inst:ty, $func:ty) => {
        impl<Stack, Locals, Memory, Frames, Trace, Trap, Req> InterpInstr<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>>
            for $inst
        where
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>: Monad
                + crate::composed::core::MonadWriter<VmTrace>
                + crate::composed::core::MonadState<VmState<Stack, Locals, Memory, Frames>>,
            PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, $inst>: Sized,
            ModifyVm<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, VmState<Stack, Locals, Memory, Frames>, $func>: Sized,
            Then<
                VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
                PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, $inst>,
                ModifyVm<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, VmState<Stack, Locals, Memory, Frames>, $func>,
            >: Eval,
        {
            type Output = Evaluate<
                Then<
                    VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
                    PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, $inst>,
                    ModifyVm<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, VmState<Stack, Locals, Memory, Frames>, $func>,
                >,
            >;
        }
    };
}

impl<V, Stack, Locals, Memory, Frames, Trace, Trap, Req>
    InterpInstr<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>> for OpPush<V>
where
    VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>: Monad
        + crate::composed::core::MonadWriter<VmTrace>
        + crate::composed::core::MonadState<VmState<Stack, Locals, Memory, Frames>>,
    PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpPush<V>>: Sized,
    ModifyVm<
        VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
        VmState<Stack, Locals, Memory, Frames>,
        LPushValue<V>,
    >: Sized,
    Then<
        VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
        PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpPush<V>>,
        ModifyVm<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            VmState<Stack, Locals, Memory, Frames>,
            LPushValue<V>,
        >,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpPush<V>>,
            ModifyVm<
                VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
                VmState<Stack, Locals, Memory, Frames>,
                LPushValue<V>,
            >,
        >,
    >;
}

impl_stateful_instr!(OpDrop, LDropTop);
impl_stateful_instr!(OpPop, LDropTop);
impl_stateful_instr!(OpDup, LDupTop);
impl_stateful_instr!(OpSwap, LSwapTop);
impl_stateful_instr!(OpSub, LBinaryStep<OpSub>);
#[cfg(feature = "nightly")]
impl_stateful_instr!(OpEq, LBinaryStep<OpEq>);
#[cfg(feature = "nightly")]
impl_stateful_instr!(OpNeq, LBinaryStep<OpNeq>);
impl_stateful_instr!(OpLt, LBinaryStep<OpLt>);
impl_stateful_instr!(OpGt, LBinaryStep<OpGt>);
impl_stateful_instr!(OpAnd, LBinaryStep<OpAnd>);
impl_stateful_instr!(OpOr, LBinaryStep<OpOr>);
impl_stateful_instr!(OpNot, LUnaryNot);
impl_stateful_instr!(OpLet, LLet);
impl_stateful_instr!(OpDropLocal, LDropLocal);
// `Dup` and `Swap` use the same state-function alias pattern in tests; v1 keeps them out of the composed comparison set.

impl<Stack, Locals, Memory, Frames, Trace, Trap, Req>
    InterpInstr<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>> for OpAdd
where
    VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>: Monad
        + crate::composed::core::MonadWriter<VmTrace>
        + crate::composed::core::MonadState<VmState<Stack, Locals, Memory, Frames>>
        + crate::composed::core::MonadError<VmTrap>,
    Bind<
        VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
        crate::composed::vm::effects::GetVm<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            VmState<Stack, Locals, Memory, Frames>,
        >,
        LRunFallible<
            LBinaryStep<OpAdd>,
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            VmState<Stack, Locals, Memory, Frames>,
        >,
    >: Eval,
    Then<
        VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
        PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpAdd>,
        Bind<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            crate::composed::vm::effects::GetVm<
                VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
                VmState<Stack, Locals, Memory, Frames>,
            >,
            LRunFallible<
                LBinaryStep<OpAdd>,
                VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
                VmState<Stack, Locals, Memory, Frames>,
            >,
        >,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpAdd>,
            Bind<
                VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
                crate::composed::vm::effects::GetVm<
                    VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
                    VmState<Stack, Locals, Memory, Frames>,
                >,
                LRunFallible<
                    LBinaryStep<OpAdd>,
                    VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
                    VmState<Stack, Locals, Memory, Frames>,
                >,
            >,
        >,
    >;
}

impl<Idx, Stack, Locals, Memory, Frames, Trace, Trap, Req> InterpInstr<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>>
    for OpGetLocal<Idx>
where
    VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>: Monad
        + crate::composed::core::MonadWriter<VmTrace>
        + crate::composed::core::MonadState<VmState<Stack, Locals, Memory, Frames>>,
    Then<
        VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
        PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpGetLocal<Idx>>,
        ModifyVm<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            VmState<Stack, Locals, Memory, Frames>,
            LGetLocal<Evaluate<Idx>>,
        >,
    >: Eval,
    Idx: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpGetLocal<Idx>>,
            ModifyVm<
                VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
                VmState<Stack, Locals, Memory, Frames>,
                LGetLocal<Evaluate<Idx>>,
            >,
        >,
    >;
}

impl<Idx, Stack, Locals, Memory, Frames, Trace, Trap, Req> InterpInstr<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>>
    for OpSetLocal<Idx>
where
    VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>: Monad
        + crate::composed::core::MonadWriter<VmTrace>
        + crate::composed::core::MonadState<VmState<Stack, Locals, Memory, Frames>>,
    Then<
        VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
        PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpSetLocal<Idx>>,
        ModifyVm<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            VmState<Stack, Locals, Memory, Frames>,
            LSetLocal<Evaluate<Idx>>,
        >,
    >: Eval,
    Idx: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpSetLocal<Idx>>,
            ModifyVm<
                VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
                VmState<Stack, Locals, Memory, Frames>,
                LSetLocal<Evaluate<Idx>>,
            >,
        >,
    >;
}

impl<Stack, Locals, Memory, Frames, Trace, Trap, Req> InterpInstr<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>>
    for OpLoad
where
    VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>: Monad
        + crate::composed::core::MonadWriter<VmTrace>
        + crate::composed::core::MonadState<VmState<Stack, Locals, Memory, Frames>>,
    Then<
        VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
        PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpLoad>,
        ModifyVm<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, VmState<Stack, Locals, Memory, Frames>, LLoad>,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpLoad>,
            ModifyVm<
                VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
                VmState<Stack, Locals, Memory, Frames>,
                LLoad,
            >,
        >,
    >;
}

impl<Stack, Locals, Memory, Frames, Trace, Trap, Req> InterpInstr<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>>
    for OpStore
where
    VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>: Monad
        + crate::composed::core::MonadWriter<VmTrace>
        + crate::composed::core::MonadState<VmState<Stack, Locals, Memory, Frames>>,
    Then<
        VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
        PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpStore>,
        ModifyVm<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, VmState<Stack, Locals, Memory, Frames>, LStore>,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpStore>,
            ModifyVm<
                VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
                VmState<Stack, Locals, Memory, Frames>,
                LStore,
            >,
        >,
    >;
}

impl<TargetProg, Stack, Locals, Memory, Frames, Trace, Trap, Req>
    InterpInstr<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>> for OpCall<TargetProg>
where
    VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>: Monad
        + crate::composed::core::MonadWriter<VmTrace>
        + crate::composed::core::MonadState<VmState<Stack, Locals, Memory, Frames>>,
    Then<
        VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
        PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpCall<TargetProg>>,
        ModifyVm<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            VmState<Stack, Locals, Memory, Frames>,
            LCall<TargetProg>,
        >,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpCall<TargetProg>>,
            ModifyVm<
                VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
                VmState<Stack, Locals, Memory, Frames>,
                LCall<TargetProg>,
            >,
        >,
    >;
}

impl<Stack, Locals, Memory, Frames, Trace, Trap, Req>
    InterpInstr<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>> for OpReturn
where
    VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>: Monad
        + crate::composed::core::MonadWriter<VmTrace>
        + crate::composed::core::MonadState<VmState<Stack, Locals, Memory, Frames>>,
    Then<
        VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
        PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpReturn>,
        ModifyVm<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, VmState<Stack, Locals, Memory, Frames>, LReturn>,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpReturn>,
            ModifyVm<
                VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
                VmState<Stack, Locals, Memory, Frames>,
                LReturn,
            >,
        >,
    >;
}

impl<Sig, Stack, Locals, Memory, Frames, Trace, Trap, Req>
    InterpInstr<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>> for OpHostCall<Sig>
where
    VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>: Monad
        + crate::composed::core::MonadWriter<VmTrace>
        + crate::composed::core::MonadSuspend<VmRequest>,
    Then<
        VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
        PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpHostCall<Sig>>,
        YieldVm<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, HostRequest<Sig>>,
    >: Eval,
{
    type Output = Evaluate<
        Then<
            VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>,
            PushTrace<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, OpHostCall<Sig>>,
            YieldVm<VmFx<VmState<Stack, Locals, Memory, Frames>, Trace, Trap, Req>, HostRequest<Sig>>,
        >,
    >;
}
