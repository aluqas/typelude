use core::marker::PhantomData;

use typelude_std::{
    core::{ELit, Eval, Evaluate, TyFn},
    std::col::array::{Array, IsList, Nil},
};

use crate::{
    core::{
        either_t::ERunEither,
        id::ERunId,
        suspend_t::RunSuspend,
        state_t::ERunState,
        suspend_t::ERunSuspend,
        traits::{
            Bind, Done as SDone, Err as EErr, Monad, MonadState, Ok as EOk, Pair, Pure, Unit,
            Yielded,
        },
        writer_t::ERunWriter,
    },
    vm::algebra::{
        effect::{
            io::VmRequest,
            stack::VmFx,
            state_ops::GetVm,
            trace::VmTrace,
            trap::VmTrap,
        },
        interpret::instr::InterpInstr,
        outcome::{Done, Raised, Suspended},
        state::VmState,
    },
};

pub struct ERunVm<State, Trace = VmTrace, Trap = VmTrap, Req = VmRequest>(
    pub PhantomData<(State, Trace, Trap, Req)>,
);

pub struct ERunVmAction<RootState, CurrentState, Trace, Trap, Req>(
    pub PhantomData<(RootState, CurrentState, Trace, Trap, Req)>,
);

pub struct ERunVmStack<State, Trace, Trap, Req>(pub PhantomData<(State, Trace, Trap, Req)>);

pub trait BuildRunAction<RootState, Trace, Trap, Req> {
    type Output;
}

impl<RootState, CurrentState, Trace, Trap, Req> BuildRunAction<RootState, Trace, Trap, Req>
    for ELit<CurrentState>
where
    CurrentState: BuildRunAction<RootState, Trace, Trap, Req>,
{
    type Output = <CurrentState as BuildRunAction<RootState, Trace, Trap, Req>>::Output;
}

impl<RootState, Stack, Locals, Memory, Frames, Trace, Trap, Req>
    BuildRunAction<RootState, Trace, Trap, Req> for VmState<Stack, Locals, Memory, Frames, Nil>
where
    VmFx<RootState, Trace, Trap, Req>: Monad,
{
    type Output = Pure<VmFx<RootState, Trace, Trap, Req>, Unit>;
}

impl<RootState, Stack, Locals, Memory, Frames, Inst, Rest, Trace, Trap, Req>
    BuildRunAction<RootState, Trace, Trap, Req>
    for VmState<Stack, Locals, Memory, Frames, Array<Inst, Rest>>
where
    VmFx<RootState, Trace, Trap, Req>: Monad + MonadState<RootState>,
    Rest: IsList,
    Inst: InterpInstr<VmFx<RootState, Trace, Trap, Req>>,
    Bind<
        VmFx<RootState, Trace, Trap, Req>,
        <Inst as InterpInstr<VmFx<RootState, Trace, Trap, Req>>>::Output,
        LResume<RootState, Trace, Trap, Req>,
    >: Eval,
{
    type Output = Bind<
        VmFx<RootState, Trace, Trap, Req>,
        <Inst as InterpInstr<VmFx<RootState, Trace, Trap, Req>>>::Output,
        LResume<RootState, Trace, Trap, Req>,
    >;
}

impl<RootState, CurrentState, Trace, Trap, Req> Eval
    for ERunVmAction<RootState, CurrentState, Trace, Trap, Req>
where
    CurrentState: BuildRunAction<RootState, Trace, Trap, Req>,
    <CurrentState as BuildRunAction<RootState, Trace, Trap, Req>>::Output: Eval,
{
    type Output = Evaluate<<CurrentState as BuildRunAction<RootState, Trace, Trap, Req>>::Output>;
}

impl<RootState, CurrentState, Trace, Trap, Req> RunSuspend
    for ERunVmAction<RootState, CurrentState, Trace, Trap, Req>
where
    ERunVmAction<RootState, CurrentState, Trace, Trap, Req>: Eval,
    Evaluate<ERunVmAction<RootState, CurrentState, Trace, Trap, Req>>: RunSuspend,
{
    type Output =
        <Evaluate<ERunVmAction<RootState, CurrentState, Trace, Trap, Req>> as RunSuspend>::Output;
}

pub struct LResume<RootState, Trace, Trap, Req>(pub PhantomData<(RootState, Trace, Trap, Req)>);

impl<RootState, Trace, Trap, Req, A> TyFn<A> for LResume<RootState, Trace, Trap, Req>
where
    VmFx<RootState, Trace, Trap, Req>: Monad + MonadState<RootState>,
    Bind<
        VmFx<RootState, Trace, Trap, Req>,
        GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
        LContinue<RootState, Trace, Trap, Req>,
    >: Eval,
{
    type Output = Bind<
        VmFx<RootState, Trace, Trap, Req>,
        GetVm<VmFx<RootState, Trace, Trap, Req>, RootState>,
        LContinue<RootState, Trace, Trap, Req>,
    >;
}

pub struct LContinue<RootState, Trace, Trap, Req>(
    pub PhantomData<(RootState, Trace, Trap, Req)>,
);

impl<RootState, Trace, Trap, Req, CurrentState> TyFn<CurrentState>
    for LContinue<RootState, Trace, Trap, Req>
where
    ERunVmAction<RootState, CurrentState, Trace, Trap, Req>: Eval,
{
    type Output = ERunVmAction<RootState, CurrentState, Trace, Trap, Req>;
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

impl<State, Trace, Result> TyFn<Pair<Result, Trace>> for LRunOutcome<State>
where
    LRunEitherOutcome<State, Trace>: TyFn<Result>,
{
    type Output = <LRunEitherOutcome<State, Trace> as TyFn<Result>>::Output;
}

pub trait BuildOutcome<State> {
    type Output;
}

impl<State, Result, EndState, Trace> BuildOutcome<State> for Pair<Pair<Result, EndState>, Trace>
where
    LRunOutcome<EndState>: TyFn<Pair<Result, Trace>>,
{
    type Output = <LRunOutcome<EndState> as TyFn<Pair<Result, Trace>>>::Output;
}

impl<State, Trace, Trap, Req> Eval for ERunVmStack<State, Trace, Trap, Req>
where
    State: Eval,
    Evaluate<State>: BuildRunAction<Evaluate<State>, Trace, Trap, Req>,
    <Evaluate<State> as BuildRunAction<Evaluate<State>, Trace, Trap, Req>>::Output: Eval,
    ERunId<
        ERunWriter<
            ERunState<
                Evaluate<State>,
                ERunEither<
                    ERunSuspend<
                        Evaluate<
                            ERunVmAction<
                                Evaluate<State>,
                                Evaluate<State>,
                                Trace,
                                Trap,
                                Req,
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
                ERunState<
                    Evaluate<State>,
                    ERunEither<
                        ERunSuspend<
                            Evaluate<
                                ERunVmAction<Evaluate<State>, Evaluate<State>, Trace, Trap, Req>,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    >;
}

impl<State, Trace, Trap, Req> Eval for ERunVm<State, Trace, Trap, Req>
where
    State: Eval,
    ERunVmStack<State, Trace, Trap, Req>: Eval,
    Evaluate<ERunVmStack<State, Trace, Trap, Req>>: BuildOutcome<Evaluate<State>>,
{
    type Output =
        <Evaluate<ERunVmStack<State, Trace, Trap, Req>> as BuildOutcome<Evaluate<State>>>::Output;
}
