use core::marker::PhantomData;

use typelude_std::core::{Eval, Evaluate, TyFn};

use crate::composed::{
    core::{
        Done as SDone, Err as EErr, Ok as EOk, Pair, Yielded, either_t::ERunEither, id::ERunId,
        state_t::ERunState, suspend_t::ERunSuspend, writer_t::ERunWriter,
    },
    vm::outcome::{Done, Raised, Suspended},
};

pub struct ERunVm<State, MA>(pub PhantomData<(State, MA)>);
pub struct ERunVmStack<State, MA>(pub PhantomData<(State, MA)>);

pub struct LRunOutcome<State>(pub PhantomData<State>);

pub struct LRunEitherOutcome<State, Trace>(pub PhantomData<(State, Trace)>);

impl<State, Trace, A> TyFn<EOk<SDone<A>>> for LRunEitherOutcome<State, Trace> {
    type Output = Done<A, State, Trace>;
}

impl<State, Trace, Reason> TyFn<EErr<Reason>> for LRunEitherOutcome<State, Trace> {
    type Output = Raised<Reason, State, Trace>;
}

impl<State, Trace, Req> TyFn<EOk<Yielded<Req>>> for LRunEitherOutcome<State, Trace> {
    type Output = Suspended<Req, State, Trace>;
}

impl<State, Trace, Result> TyFn<Pair<Result, Trace>> for LRunOutcome<State>
where
    LRunEitherOutcome<State, Trace>: TyFn<Result>,
{
    type Output = <LRunEitherOutcome<State, Trace> as TyFn<Result>>::Output;
}

impl<State, MA> Eval for ERunVmStack<State, MA>
where
    ERunId<ERunWriter<ERunState<State, ERunEither<ERunSuspend<MA>>>>>: Eval,
{
    type Output = Evaluate<ERunId<ERunWriter<ERunState<State, ERunEither<ERunSuspend<MA>>>>>>;
}

impl<State, MA> Eval for ERunVm<State, MA>
where
    ERunVmStack<State, MA>: Eval,
    Evaluate<ERunVmStack<State, MA>>: TyFnHelper<State>,
{
    type Output = <Evaluate<ERunVmStack<State, MA>> as TyFnHelper<State>>::Output;
}

pub trait TyFnHelper<State> {
    type Output;
}

impl<State, Result, EndState, Trace> TyFnHelper<State> for Pair<Pair<Result, EndState>, Trace>
where
    LRunOutcome<EndState>: TyFn<Pair<Result, Trace>>,
{
    type Output = <LRunOutcome<EndState> as TyFn<Pair<Result, Trace>>>::Output;
}
