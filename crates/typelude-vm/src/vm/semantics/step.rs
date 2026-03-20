use core::marker::PhantomData;

use typelude_std::core::Eval;

#[derive(Debug)]
pub struct StepContinue<State>(pub PhantomData<State>);

#[derive(Debug)]
pub struct StepTrap<Reason>(pub PhantomData<Reason>);

#[derive(Debug)]
pub struct StepSuspend<Request, State>(pub PhantomData<(Request, State)>);

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
