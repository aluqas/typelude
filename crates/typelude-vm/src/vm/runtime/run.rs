//! Small-step runtime loop for the pure VM semantics.
//!
//! Execution order is fixed:
//! 1. record the dispatched instruction in the source trace
//! 2. record the lowered/core instruction in the core trace when applicable
//! 3. interpret the pure `StepInstr` result into state/trap/suspend effects
//! 4. recurse from the committed state

use core::marker::PhantomData;

use typelude_std::{
    core::{ELit, Eval, Evaluate, TyFn},
    std::col::array::{Array, IsList, Nil},
};

use crate::{
    core::{
        either_t::ERunEither,
        id::ERunId,
        state_t::ERunState,
        suspend_t::{ERunSuspend, RunSuspend},
        traits::{
            Bind, Done as SDone, Err as EErr, Monad, MonadError, MonadState, MonadSuspend,
            MonadWriter, Ok as EOk, Pair, Pure, Unit, Yielded,
        },
        writer_t::ERunWriter,
    },
    opcode::{
        control::{OpCall, OpIf, OpReturn, OpWhile},
        host::OpHostCall,
        local::{OpDropLocal, OpGetLocal, OpLet, OpSetLocal},
        memory::{OpLoad, OpStore},
        numeric::{OpAdd, OpAnd, OpEq, OpGt, OpLt, OpNeq, OpNot, OpOr, OpSub},
        stack::{OpDrop, OpDup, OpPop, OpPush, OpSwap},
    },
    vm::{
        protocol::request::HostRequest,
        runtime::{
            effects::{
                io::{VmRequest, YieldVm},
                stack::VmFx,
                state_ops::{GetVm, PutVm, Then},
                trace::{
                    AppendTraceBundle, PushCoreTrace, PushSourceTrace, TraceBundle,
                    VmCoreTrace, VmSourceTrace,
                },
                trap::{ThrowVm, VmTrap},
            },
            outcome::{Done, Raised, Suspended},
        },
        semantics::{
            helpers::value::AsValueExpr,
            state::VmState,
            step::{StepContinue, StepInstr, StepSuspend, StepTrap},
        },
    },
};

/// Runs a VM program to completion, trap, or suspension.
pub struct ERunVm<
    State,
    SourceTrace = VmSourceTrace,
    CoreTrace = VmCoreTrace,
    Trap = VmTrap,
    Req = VmRequest,
>(pub PhantomData<(State, SourceTrace, CoreTrace, Trap, Req)>);

pub struct ERunVmAction<RootState, CurrentState, SourceTrace, CoreTrace, Trap, Req>(
    pub PhantomData<(RootState, CurrentState, SourceTrace, CoreTrace, Trap, Req)>,
);

pub struct ERunVmStack<State, SourceTrace, CoreTrace, Trap, Req>(
    pub PhantomData<(State, SourceTrace, CoreTrace, Trap, Req)>,
);

/// Resumes a previously suspended VM by pushing the host response onto the stack and continuing.
pub struct EResumeVm<Outcome, Response>(pub PhantomData<(Outcome, Response)>);

pub trait BuildRunAction<RootState, SourceTrace, CoreTrace, Trap, Req> {
    type Output;
}

impl<RootState, CurrentState, SourceTrace, CoreTrace, Trap, Req>
    BuildRunAction<RootState, SourceTrace, CoreTrace, Trap, Req> for ELit<CurrentState>
where
    CurrentState: BuildRunAction<RootState, SourceTrace, CoreTrace, Trap, Req>,
{
    type Output =
        <CurrentState as BuildRunAction<RootState, SourceTrace, CoreTrace, Trap, Req>>::Output;
}

impl<RootState, Stack, Locals, Memory, Frames, SourceTrace, CoreTrace, Trap, Req>
    BuildRunAction<RootState, SourceTrace, CoreTrace, Trap, Req>
    for VmState<Stack, Locals, Memory, Frames, Nil>
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
    type Output =
        PushCoreTrace<RootState, OpGetLocal<Idx>, SourceTrace, CoreTrace, Trap, Req>;
}

impl<Idx, RootState, SourceTrace, CoreTrace, Trap, Req>
    RecordCoreTrace<RootState, SourceTrace, CoreTrace, Trap, Req> for OpSetLocal<Idx>
{
    type Output =
        PushCoreTrace<RootState, OpSetLocal<Idx>, SourceTrace, CoreTrace, Trap, Req>;
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
    type Output =
        PushCoreTrace<RootState, OpCall<TargetProg>, SourceTrace, CoreTrace, Trap, Req>;
}

impl<Sig, RootState, SourceTrace, CoreTrace, Trap, Req>
    RecordCoreTrace<RootState, SourceTrace, CoreTrace, Trap, Req> for OpHostCall<Sig>
{
    type Output =
        PushCoreTrace<RootState, OpHostCall<Sig>, SourceTrace, CoreTrace, Trap, Req>;
}

impl<CondProg, BodyProg, RootState, SourceTrace, CoreTrace, Trap, Req>
    RecordCoreTrace<RootState, SourceTrace, CoreTrace, Trap, Req> for OpWhile<CondProg, BodyProg>
where
    VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>: Monad,
{
    type Output = Pure<VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>, Unit>;
}

impl<RootState, Stack, Locals, Memory, Frames, Inst, Rest, SourceTrace, CoreTrace, Trap, Req>
    BuildRunAction<RootState, SourceTrace, CoreTrace, Trap, Req>
    for VmState<Stack, Locals, Memory, Frames, Array<Inst, Rest>>
where
    Rest: IsList,
    Inst: StepInstr<VmState<Stack, Locals, Memory, Frames, Array<Inst, Rest>>>
        + RecordCoreTrace<RootState, SourceTrace, CoreTrace, Trap, Req>,
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
                <Inst as RecordCoreTrace<
                    RootState,
                    SourceTrace,
                    CoreTrace,
                    Trap,
                    Req,
                >>::Output,
            >,
            LApplyLoggedStep<
                VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>,
                RootState,
                Trap,
                Req,
                <Inst as StepInstr<
                    VmState<Stack, Locals, Memory, Frames, Array<Inst, Rest>>,
                >>::Output,
            >,
        >,
        LResume<RootState, SourceTrace, CoreTrace, Trap, Req>,
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
                <Inst as RecordCoreTrace<
                    RootState,
                    SourceTrace,
                    CoreTrace,
                    Trap,
                    Req,
                >>::Output,
            >,
            LApplyLoggedStep<
                VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>,
                RootState,
                Trap,
                Req,
                <Inst as StepInstr<
                    VmState<Stack, Locals, Memory, Frames, Array<Inst, Rest>>,
                >>::Output,
            >,
        >,
        LResume<RootState, SourceTrace, CoreTrace, Trap, Req>,
    >;
}

impl<RootState, CurrentState, SourceTrace, CoreTrace, Trap, Req> Eval
    for ERunVmAction<RootState, CurrentState, SourceTrace, CoreTrace, Trap, Req>
where
    CurrentState: BuildRunAction<RootState, SourceTrace, CoreTrace, Trap, Req>,
    <CurrentState as BuildRunAction<RootState, SourceTrace, CoreTrace, Trap, Req>>::Output: Eval,
{
    type Output = Evaluate<
        <CurrentState as BuildRunAction<RootState, SourceTrace, CoreTrace, Trap, Req>>::Output,
    >;
}

impl<RootState, CurrentState, SourceTrace, CoreTrace, Trap, Req> RunSuspend
    for ERunVmAction<RootState, CurrentState, SourceTrace, CoreTrace, Trap, Req>
where
    ERunVmAction<RootState, CurrentState, SourceTrace, CoreTrace, Trap, Req>: Eval,
    Evaluate<ERunVmAction<RootState, CurrentState, SourceTrace, CoreTrace, Trap, Req>>:
        RunSuspend,
{
    type Output = <Evaluate<
        ERunVmAction<RootState, CurrentState, SourceTrace, CoreTrace, Trap, Req>,
    > as RunSuspend>::Output;
}

pub struct LApplyStep<F, RootState, Trap, Req>(pub PhantomData<(F, RootState, Trap, Req)>);

impl<F, RootState, Trap, Req, NextState> TyFn<StepContinue<NextState>>
    for LApplyStep<F, RootState, Trap, Req>
where
    F: MonadState<RootState>,
{
    type Output = PutVm<F, RootState, NextState>;
}

impl<F, RootState, Trap, Req, Reason> TyFn<StepTrap<Reason>>
    for LApplyStep<F, RootState, Trap, Req>
where
    F: MonadError<Trap>,
{
    type Output = ThrowVm<F, Reason, Trap>;
}

impl<F, RootState, Trap, Req, Request, NextState> TyFn<StepSuspend<Request, NextState>>
    for LApplyStep<F, RootState, Trap, Req>
where
    F: Monad + MonadState<RootState> + MonadSuspend<Req>,
    Then<F, PutVm<F, RootState, NextState>, YieldVm<F, Request, Req>>: Sized,
{
    type Output = Then<F, PutVm<F, RootState, NextState>, YieldVm<F, Request, Req>>;
}

pub struct LApplyLoggedStep<F, RootState, Trap, Req, Step>(
    pub PhantomData<(F, RootState, Trap, Req, Step)>,
);

impl<F, RootState, Trap, Req, Step, A> TyFn<A> for LApplyLoggedStep<F, RootState, Trap, Req, Step>
where
    LApplyStep<F, RootState, Trap, Req>: TyFn<Step>,
{
    type Output = <LApplyStep<F, RootState, Trap, Req> as TyFn<Step>>::Output;
}

pub struct LResume<RootState, SourceTrace, CoreTrace, Trap, Req>(
    pub PhantomData<(RootState, SourceTrace, CoreTrace, Trap, Req)>,
);

impl<RootState, SourceTrace, CoreTrace, Trap, Req, A> TyFn<A>
    for LResume<RootState, SourceTrace, CoreTrace, Trap, Req>
where
    VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>: Monad + MonadState<RootState>,
    Bind<
        VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>,
        GetVm<VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>, RootState>,
        LContinue<RootState, SourceTrace, CoreTrace, Trap, Req>,
    >: Eval,
{
    type Output = Bind<
        VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>,
        GetVm<VmFx<RootState, SourceTrace, CoreTrace, Trap, Req>, RootState>,
        LContinue<RootState, SourceTrace, CoreTrace, Trap, Req>,
    >;
}

pub struct LContinue<RootState, SourceTrace, CoreTrace, Trap, Req>(
    pub PhantomData<(RootState, SourceTrace, CoreTrace, Trap, Req)>,
);

impl<RootState, SourceTrace, CoreTrace, Trap, Req, CurrentState> TyFn<CurrentState>
    for LContinue<RootState, SourceTrace, CoreTrace, Trap, Req>
where
    ERunVmAction<RootState, CurrentState, SourceTrace, CoreTrace, Trap, Req>: Eval,
{
    type Output = ERunVmAction<RootState, CurrentState, SourceTrace, CoreTrace, Trap, Req>;
}

pub struct LRunOutcome<State>(pub PhantomData<State>);
pub struct LRunEitherOutcome<State, Trace>(pub PhantomData<(State, Trace)>);

impl<State, Trace, A> TyFn<EOk<SDone<A>>> for LRunEitherOutcome<State, Trace> {
    type Output = Done<A, State, Trace>;
}

impl<State, Trace, Reason> TyFn<EErr<Reason>> for LRunEitherOutcome<State, Trace> {
    type Output = Raised<Reason, State, Trace>;
}

impl<State, Trace, Request> TyFn<EOk<Yielded<Request>>> for LRunEitherOutcome<State, Trace> {
    type Output = Suspended<Request, State, Trace>;
}

impl<State, SourceTrace, CoreTrace, Result>
    TyFn<Pair<Result, TraceBundle<SourceTrace, CoreTrace>>> for LRunOutcome<State>
where
    LRunEitherOutcome<State, TraceBundle<SourceTrace, CoreTrace>>: TyFn<Result>,
{
    type Output = <LRunEitherOutcome<State, TraceBundle<SourceTrace, CoreTrace>> as TyFn<
        Result,
    >>::Output;
}

pub trait BuildOutcome<State> {
    type Output;
}

impl<State, Result, EndState, SourceTrace, CoreTrace> BuildOutcome<State>
    for Pair<Pair<Pair<Result, EndState>, SourceTrace>, CoreTrace>
where
    LRunOutcome<EndState>: TyFn<Pair<Result, TraceBundle<SourceTrace, CoreTrace>>>,
{
    type Output = <LRunOutcome<EndState> as TyFn<
        Pair<Result, TraceBundle<SourceTrace, CoreTrace>>,
    >>::Output;
}

impl<State, SourceTrace, CoreTrace, Trap, Req> Eval
    for ERunVmStack<State, SourceTrace, CoreTrace, Trap, Req>
where
    State: Eval,
    Evaluate<State>: BuildRunAction<Evaluate<State>, SourceTrace, CoreTrace, Trap, Req>,
    <Evaluate<State> as BuildRunAction<
        Evaluate<State>,
        SourceTrace,
        CoreTrace,
        Trap,
        Req,
    >>::Output: Eval,
    ERunId<
        ERunWriter<
            ERunWriter<
                ERunState<
                    Evaluate<State>,
                    ERunEither<
                        ERunSuspend<
                            Evaluate<
                                ERunVmAction<
                                    Evaluate<State>,
                                    Evaluate<State>,
                                    SourceTrace,
                                    CoreTrace,
                                    Trap,
                                    Req,
                                >,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    >: Eval,
{
    type Output = Evaluate<
        ERunId<
            ERunWriter<
                ERunWriter<
                    ERunState<
                        Evaluate<State>,
                        ERunEither<
                            ERunSuspend<
                                Evaluate<
                                    ERunVmAction<
                                        Evaluate<State>,
                                        Evaluate<State>,
                                        SourceTrace,
                                        CoreTrace,
                                        Trap,
                                        Req,
                                    >,
                                >,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    >;
}

impl<State, SourceTrace, CoreTrace, Trap, Req> Eval
    for ERunVm<State, SourceTrace, CoreTrace, Trap, Req>
where
    State: Eval,
    ERunVmStack<State, SourceTrace, CoreTrace, Trap, Req>: Eval,
    Evaluate<ERunVmStack<State, SourceTrace, CoreTrace, Trap, Req>>: BuildOutcome<Evaluate<State>>,
{
    type Output = <Evaluate<
        ERunVmStack<State, SourceTrace, CoreTrace, Trap, Req>,
    > as BuildOutcome<Evaluate<State>>>::Output;
}

/// Pushes a host response value onto the stack of a suspended VM state.
pub trait ResumeState<Response> {
    type Output;
}

impl<Stack, Locals, Memory, Frames, Program, Response> ResumeState<Response>
    for VmState<Stack, Locals, Memory, Frames, Program>
where
    Stack: IsList,
    Response: AsValueExpr,
{
    type Output = VmState<
        Array<<Response as AsValueExpr>::Output, Stack>,
        Locals,
        Memory,
        Frames,
        Program,
    >;
}

/// Appends newly produced traces to a previously suspended outcome.
pub trait AppendOutcomeTrace<ExistingTrace> {
    type Output;
}

impl<A, State, NewTrace, ExistingTrace> AppendOutcomeTrace<ExistingTrace> for Done<A, State, NewTrace>
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
    type Output = Raised<
        Reason,
        State,
        <ExistingTrace as AppendTraceBundle<NewTrace>>::Output,
    >;
}

impl<Request, State, NewTrace, ExistingTrace> AppendOutcomeTrace<ExistingTrace>
    for Suspended<Request, State, NewTrace>
where
    ExistingTrace: AppendTraceBundle<NewTrace>,
{
    type Output = Suspended<
        Request,
        State,
        <ExistingTrace as AppendTraceBundle<NewTrace>>::Output,
    >;
}

impl<Sig, Args, Response, State, ExistingTrace> Eval
    for EResumeVm<Suspended<HostRequest<Sig, Args, Response>, State, ExistingTrace>, Response>
where
    State: ResumeState<Response>,
    ERunVm<ELit<<State as ResumeState<Response>>::Output>>: Eval,
    Evaluate<ERunVm<ELit<<State as ResumeState<Response>>::Output>>>:
        AppendOutcomeTrace<ExistingTrace>,
{
    type Output = <Evaluate<
        ERunVm<ELit<<State as ResumeState<Response>>::Output>>,
    > as AppendOutcomeTrace<ExistingTrace>>::Output;
}
