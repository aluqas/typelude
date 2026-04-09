//! Small-step runtime loop for the pure VM semantics.
//!
//! Execution order is fixed:
//! 1. record the dispatched instruction in the source trace
//! 2. record the lowered/core instruction in the core trace when applicable
//! 3. interpret the pure `Op<VmState>` result into state/trap/suspend effects
//! 4. recurse from the committed state

use core::marker::PhantomData;

use typelude_std::{
    core::{Eval, Evaluate},
    effect::{RunEither, RunId, RunState, RunSuspend, RunWriter},
};

use crate::vm::{
    protocol::request::HostRequest,
    runtime::{
        driver::{AppendOutcomeTrace, BuildOutcome, BuildRunAction, ResumeState},
        effects::{
            trace::{VmCoreTrace, VmSourceTrace},
            trap::VmTrap,
        },
        outcome::Suspended,
    },
};

/// Runs a VM program to completion, trap, or suspension.
pub struct RunVm<
    State,
    SourceTrace = VmSourceTrace,
    CoreTrace = VmCoreTrace,
    Trap = VmTrap,
    Req = super::effects::io::VmRequest,
>(pub PhantomData<(State, SourceTrace, CoreTrace, Trap, Req)>);

pub struct RunVmAction<RootState, CurrentState, SourceTrace, CoreTrace, Trap, Req>(
    pub PhantomData<(RootState, CurrentState, SourceTrace, CoreTrace, Trap, Req)>,
);

pub struct RunVmStack<State, SourceTrace, CoreTrace, Trap, Req>(
    pub PhantomData<(State, SourceTrace, CoreTrace, Trap, Req)>,
);

/// Resumes a previously suspended VM by pushing the host response onto the
/// stack and continuing.
pub struct ResumeVm<Outcome, Response>(pub PhantomData<(Outcome, Response)>);

impl<RootState, CurrentState, SourceTrace, CoreTrace, Trap, Req> Eval
    for RunVmAction<RootState, CurrentState, SourceTrace, CoreTrace, Trap, Req>
where
    CurrentState: BuildRunAction<RootState, SourceTrace, CoreTrace, Trap, Req>,
    <CurrentState as BuildRunAction<RootState, SourceTrace, CoreTrace, Trap, Req>>::Output: Eval,
{
    type Output = Evaluate<
        <CurrentState as BuildRunAction<RootState, SourceTrace, CoreTrace, Trap, Req>>::Output,
    >;
}

impl<State, SourceTrace, CoreTrace, Trap, Req> Eval
    for RunVmStack<State, SourceTrace, CoreTrace, Trap, Req>
where
    State: Eval,
    Evaluate<State>: BuildRunAction<Evaluate<State>, SourceTrace, CoreTrace, Trap, Req>,
    RunId<
        RunWriter<
            RunWriter<
                RunState<
                    Evaluate<State>,
                    RunEither<
                        RunSuspend<
                            RunVmAction<
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
    >: Eval,
{
    type Output = Evaluate<
        RunId<
            RunWriter<
                RunWriter<
                    RunState<
                        Evaluate<State>,
                        RunEither<
                            RunSuspend<
                                RunVmAction<
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
    >;
}

impl<State, SourceTrace, CoreTrace, Trap, Req> Eval
    for RunVm<State, SourceTrace, CoreTrace, Trap, Req>
where
    State: Eval,
    RunVmStack<State, SourceTrace, CoreTrace, Trap, Req>: Eval,
    Evaluate<RunVmStack<State, SourceTrace, CoreTrace, Trap, Req>>: BuildOutcome<Evaluate<State>>,
{
    type Output =
        <Evaluate<RunVmStack<State, SourceTrace, CoreTrace, Trap, Req>> as BuildOutcome<
            Evaluate<State>,
        >>::Output;
}

impl<Sig, Args, Response, State, ExistingTrace> Eval
    for ResumeVm<Suspended<HostRequest<Sig, Args, Response>, State, ExistingTrace>, Response>
where
    State: ResumeState<Response>,
    RunVm<<State as ResumeState<Response>>::Output>: Eval,
    Evaluate<RunVm<<State as ResumeState<Response>>::Output>>: AppendOutcomeTrace<ExistingTrace>,
{
    type Output =
        <Evaluate<RunVm<<State as ResumeState<Response>>::Output>> as AppendOutcomeTrace<
            ExistingTrace,
        >>::Output;
}
