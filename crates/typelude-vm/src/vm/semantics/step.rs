//! Pure instruction-step results.

use core::marker::PhantomData;

use typelude_std::core::Eval;

/// A successful instruction step that commits `State` and continues execution.
#[derive(Debug)]
pub struct StepContinue<State>(pub PhantomData<State>);

/// A trapped instruction step that leaves the pre-step state committed.
#[derive(Debug)]
pub struct StepTrap<Reason>(pub PhantomData<Reason>);

/// A suspended instruction step that commits `State` before yielding `Request`.
#[derive(Debug)]
pub struct StepSuspend<Request, State>(pub PhantomData<(Request, State)>);

/// Pure instruction semantics from a current state to a single small-step
/// result.
pub trait StepInstr<State> {
    type Output;
}

impl<State> Eval for StepContinue<State> {
    type Output = Self;
}

impl<Reason> Eval for StepTrap<Reason> {
    type Output = Self;
}

impl<Request, State> Eval for StepSuspend<Request, State> {
    type Output = Self;
}
