use core::marker::PhantomData;

use typelude_col::{TArr, TTerm};
use typelude_std::{
    core::{Apply, Eval, Evaluate, Op},
    effect::{
        Bind, Done as SDone, Err as EErr, Monad, MonadError, MonadState, MonadSuspend,
        MonadWriter, Ok as EOk, Pair, Pure, Unit, Yielded,
    },
};

use crate::{
    opcode::{
        control::{OpCall, OpIf, OpReturn, OpWhile},
        host::OpHostCall,
        local::{OpDropLocal, OpGetLocal, OpLet, OpSetLocal},
        memory::{OpLoad, OpStore},
        numeric::{OpAdd, OpAnd, OpEq, OpGt, OpLt, OpNeq, OpNot, OpOr, OpSub},
        stack::{OpDrop, OpDup, OpPop, OpPush, OpSwap},
    },
    vm::{
        runtime::{
            effects::{
                io::YieldVm,
                stack::VmFx,
                state_ops::{GetVm, PutVm, Then},
                trace::{AppendTraceBundle, PushCoreTrace, PushSourceTrace, TraceBundle},
                trap::ThrowVm,
            },
            outcome::{Done, Raised, Suspended},
            run::RunVmAction,
        },
        semantics::{
            state::VmState,
            step::{StepContinue, StepSuspend, StepTrap},
        },
        value::Lit,
    },
};

pub trait BuildRunAction<RootState, SourceTrace, CoreTrace, Trap, Req> {
    type Output;
}

impl<RootState, Stack, Locals, Memory, Frames, SourceTrace, CoreTrace, Trap, Req>
    BuildRunAction<RootState, SourceTrace, CoreTrace, Trap, Req>
    for VmState<Stack, Locals, Memory, Frames, TTerm>
where
    VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>: Monad,
{
    type Output = Pure<VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>, Unit>;
}

pub trait RecordCoreTrace<RootState, SourceTrace, CoreTrace, Trap, Req> {
    type Output;
}

macro_rules! impl_record_core_trace {
    ($($inst:ty),+ $(,)?) => {
        $(
            impl<RootState, SourceTrace, CoreTrace, Trap, Req>
                RecordCoreTrace<RootState, SourceTrace, CoreTrace, Trap, Req> for $inst
            {
                type Output = PushCoreTrace<RootState, $inst, SourceTrace, CoreTrace, Trap, Req>;
            }
        )+
    };
}

impl_record_core_trace!(
    OpDrop,
    OpDup,
    OpPop,
    OpAdd,
    OpSub,
    OpEq,
    OpNeq,
    OpLt,
    OpGt,
    OpNot,
    OpAnd,
    OpOr,
    OpLet,
    OpDropLocal,
    OpLoad,
    OpStore,
    OpSwap,
    OpReturn
);

impl<V, RootState, SourceTrace, CoreTrace, Trap, Req>
    RecordCoreTrace<RootState, SourceTrace, CoreTrace, Trap, Req> for OpPush<V>
{
    type Output = PushCoreTrace<RootState, OpPush<V>, SourceTrace, CoreTrace, Trap, Req>;
}

impl<Idx, RootState, SourceTrace, CoreTrace, Trap, Req>
    RecordCoreTrace<RootState, SourceTrace, CoreTrace, Trap, Req> for OpGetLocal<Idx>
{
    type Output = PushCoreTrace<RootState, OpGetLocal<Idx>, SourceTrace, CoreTrace, Trap, Req>;
}

impl<Idx, RootState, SourceTrace, CoreTrace, Trap, Req>
    RecordCoreTrace<RootState, SourceTrace, CoreTrace, Trap, Req> for OpSetLocal<Idx>
{
    type Output = PushCoreTrace<RootState, OpSetLocal<Idx>, SourceTrace, CoreTrace, Trap, Req>;
}

impl<ThenProg, ElseProg, RootState, SourceTrace, CoreTrace, Trap, Req>
    RecordCoreTrace<RootState, SourceTrace, CoreTrace, Trap, Req> for OpIf<ThenProg, ElseProg>
{
    type Output =
        PushCoreTrace<RootState, OpIf<ThenProg, ElseProg>, SourceTrace, CoreTrace, Trap, Req>;
}

impl<TargetProg, RootState, SourceTrace, CoreTrace, Trap, Req>
    RecordCoreTrace<RootState, SourceTrace, CoreTrace, Trap, Req> for OpCall<TargetProg>
{
    type Output = PushCoreTrace<RootState, OpCall<TargetProg>, SourceTrace, CoreTrace, Trap, Req>;
}

impl<Sig, RootState, SourceTrace, CoreTrace, Trap, Req>
    RecordCoreTrace<RootState, SourceTrace, CoreTrace, Trap, Req> for OpHostCall<Sig>
{
    type Output = PushCoreTrace<RootState, OpHostCall<Sig>, SourceTrace, CoreTrace, Trap, Req>;
}

impl<CondProg, BodyProg, RootState, SourceTrace, CoreTrace, Trap, Req>
    RecordCoreTrace<RootState, SourceTrace, CoreTrace, Trap, Req> for OpWhile<CondProg, BodyProg>
where
    VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>: Monad,
{
    type Output = Pure<VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>, Unit>;
}

pub struct InterpretStep<F, RootState, Trap, Req>(pub PhantomData<(F, RootState, Trap, Req)>);

impl<F, RootState, Trap, Req, NextState> Op<StepContinue<NextState>>
    for InterpretStep<F, RootState, Trap, Req>
where
    F: MonadState<RootState>,
{
    type Output = PutVm<F, RootState, NextState>;
}

impl<F, RootState, Trap, Req, Reason> Op<StepTrap<Reason>>
    for InterpretStep<F, RootState, Trap, Req>
where
    F: MonadError<Trap>,
{
    type Output = ThrowVm<F, Reason, Trap>;
}

impl<F, RootState, Trap, Req, Request, NextState> Op<StepSuspend<Request, NextState>>
    for InterpretStep<F, RootState, Trap, Req>
where
    F: Monad + MonadState<RootState> + MonadSuspend<Req>,
{
    type Output = Then<F, PutVm<F, RootState, NextState>, YieldVm<F, Request, Req>>;
}

pub struct InterpretLoggedStep<F, RootState, Trap, Req, Step>(
    pub PhantomData<(F, RootState, Trap, Req, Step)>,
);

impl<F, RootState, Trap, Req, Step, A> Op<A> for InterpretLoggedStep<F, RootState, Trap, Req, Step>
where
    Apply<InterpretStep<F, RootState, Trap, Req>, Step>: Eval,
{
    type Output = Apply<InterpretStep<F, RootState, Trap, Req>, Step>;
}

pub struct Resume<RootState, SourceTrace, CoreTrace, Trap, Req>(
    pub PhantomData<(RootState, SourceTrace, CoreTrace, Trap, Req)>,
);

impl<RootState, SourceTrace, CoreTrace, Trap, Req, A> Op<A>
    for Resume<RootState, SourceTrace, CoreTrace, Trap, Req>
where
    VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>: Monad + MonadState<RootState>,
    Bind<
        VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>,
        GetVm<VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>, RootState>,
        Continue<RootState, SourceTrace, CoreTrace, Trap, Req>,
    >: Eval,
{
    type Output = Bind<
        VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>,
        GetVm<VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>, RootState>,
        Continue<RootState, SourceTrace, CoreTrace, Trap, Req>,
    >;
}

pub struct Continue<RootState, SourceTrace, CoreTrace, Trap, Req>(
    pub PhantomData<(RootState, SourceTrace, CoreTrace, Trap, Req)>,
);

impl<RootState, SourceTrace, CoreTrace, Trap, Req, CurrentState> Op<CurrentState>
    for Continue<RootState, SourceTrace, CoreTrace, Trap, Req>
{
    type Output = RunVmAction<RootState, CurrentState, SourceTrace, CoreTrace, Trap, Req>;
}

impl<RootState, Stack, Locals, Memory, Frames, Inst, Rest, SourceTrace, CoreTrace, Trap, Req>
    BuildRunAction<RootState, SourceTrace, CoreTrace, Trap, Req>
    for VmState<Stack, Locals, Memory, Frames, TArr<Inst, Rest>>
where
    VmState<Stack, Locals, Memory, Frames, TArr<Inst, Rest>>: Eval,
    Inst: Op<VmState<Stack, Locals, Memory, Frames, TArr<Inst, Rest>>>
        + RecordCoreTrace<RootState, SourceTrace, CoreTrace, Trap, Req>,
    Apply<Inst, VmState<Stack, Locals, Memory, Frames, TArr<Inst, Rest>>>: Eval,
    VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>: Monad
        + MonadState<RootState>
        + MonadWriter<SourceTrace>
        + MonadError<Trap>
        + MonadSuspend<Req>,
    Bind<
        VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>,
        Bind<
            VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>,
            Then<
                VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>,
                PushSourceTrace<
                    VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>,
                    Inst,
                    SourceTrace,
                >,
                <Inst as RecordCoreTrace<RootState, SourceTrace, CoreTrace, Trap, Req>>::Output,
            >,
            InterpretLoggedStep<
                VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>,
                RootState,
                Trap,
                Req,
                Evaluate<Apply<Inst, VmState<Stack, Locals, Memory, Frames, TArr<Inst, Rest>>>>,
            >,
        >,
        Resume<RootState, SourceTrace, CoreTrace, Trap, Req>,
    >: Eval,
{
    type Output = Bind<
        VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>,
        Bind<
            VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>,
            Then<
                VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>,
                PushSourceTrace<
                    VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>,
                    Inst,
                    SourceTrace,
                >,
                <Inst as RecordCoreTrace<RootState, SourceTrace, CoreTrace, Trap, Req>>::Output,
            >,
            InterpretLoggedStep<
                VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>,
                RootState,
                Trap,
                Req,
                Evaluate<Apply<Inst, VmState<Stack, Locals, Memory, Frames, TArr<Inst, Rest>>>>,
            >,
        >,
        Resume<RootState, SourceTrace, CoreTrace, Trap, Req>,
    >;
}

pub struct ResolveRunOutcome<State>(pub PhantomData<State>);
pub struct ResolveEitherOutcome<State, Trace>(pub PhantomData<(State, Trace)>);

impl<State, Trace, A> Op<EOk<SDone<A>>> for ResolveEitherOutcome<State, Trace> {
    type Output = Done<A, State, Trace>;
}

impl<State, Trace, Reason> Op<EErr<Reason>> for ResolveEitherOutcome<State, Trace> {
    type Output = Raised<Reason, State, Trace>;
}

impl<State, Trace, Request> Op<EOk<Yielded<Request>>> for ResolveEitherOutcome<State, Trace> {
    type Output = Suspended<Request, State, Trace>;
}

impl<State, SourceTrace, CoreTrace, Result> Op<Pair<Result, TraceBundle<SourceTrace, CoreTrace>>>
    for ResolveRunOutcome<State>
where
    Apply<ResolveEitherOutcome<State, TraceBundle<SourceTrace, CoreTrace>>, Result>: Eval,
{
    type Output =
        Evaluate<Apply<ResolveEitherOutcome<State, TraceBundle<SourceTrace, CoreTrace>>, Result>>;
}

pub trait BuildOutcome<State> {
    type Output;
}

impl<State, Result, EndState, SourceTrace, CoreTrace> BuildOutcome<State>
    for Pair<Pair<Pair<Result, EndState>, SourceTrace>, CoreTrace>
where
    Apply<ResolveRunOutcome<EndState>, Pair<Result, TraceBundle<SourceTrace, CoreTrace>>>: Eval,
{
    type Output = Evaluate<
        Apply<ResolveRunOutcome<EndState>, Pair<Result, TraceBundle<SourceTrace, CoreTrace>>>,
    >;
}

/// Pushes a host response value onto the stack of a suspended VM state.
pub trait ResumeState<Response> {
    type Output;
}

impl<Stack, Locals, Memory, Frames, Program, Response> ResumeState<Response>
    for VmState<Stack, Locals, Memory, Frames, Program>
{
    type Output = VmState<TArr<Lit<Response>, Stack>, Locals, Memory, Frames, Program>;
}

/// Appends newly produced traces to a previously suspended outcome.
pub trait AppendOutcomeTrace<ExistingTrace> {
    type Output;
}

impl<A, State, NewTrace, ExistingTrace> AppendOutcomeTrace<ExistingTrace>
    for Done<A, State, NewTrace>
where
    ExistingTrace: AppendTraceBundle<NewTrace>,
{
    type Output = Done<A, State, <ExistingTrace as AppendTraceBundle<NewTrace>>::Output>;
}

impl<Reason, State, NewTrace, ExistingTrace> AppendOutcomeTrace<ExistingTrace>
    for Raised<Reason, State, NewTrace>
where
    ExistingTrace: AppendTraceBundle<NewTrace>,
{
    type Output = Raised<Reason, State, <ExistingTrace as AppendTraceBundle<NewTrace>>::Output>;
}

impl<Request, State, NewTrace, ExistingTrace> AppendOutcomeTrace<ExistingTrace>
    for Suspended<Request, State, NewTrace>
where
    ExistingTrace: AppendTraceBundle<NewTrace>,
{
    type Output =
        Suspended<Request, State, <ExistingTrace as AppendTraceBundle<NewTrace>>::Output>;
}
