//! Static well-formedness proofs for VM programs.
//!
//! This layer is independent from the runtime effect stack. It reuses pure
//! `Op<VmState>` semantics to prove whether a concrete program/state pair can
//! execute without trapping.

use core::marker::PhantomData;

use typelude_col::{TArr, TTerm};
use typelude_std::core::{Apply, Eval, Evaluate, Op};

use crate::vm::semantics::{
    state::VmState,
    step::{StepContinue, StepSuspend, StepTrap},
};

/// Proof that a state or program can execute without trapping, carrying the
/// next safe state.
#[derive(Debug)]
pub struct WellFormed<State>(pub PhantomData<State>);

/// Proof failure carrying the reason and the state at which the static check
/// failed.
#[derive(Debug)]
pub struct IllFormed<Reason, State>(pub PhantomData<(Reason, State)>);

impl<State> Eval for WellFormed<State> {
    type Output = Self;
}

impl<Reason, State> Eval for IllFormed<Reason, State> {
    type Output = Self;
}

/// Checks whether a single instruction is statically well formed in a given
/// state.
pub trait InstrWellFormed<State> {
    type Output;
}

pub trait StepWellFormed<State> {
    type Output;
}

impl<State, NextState> StepWellFormed<State> for StepContinue<NextState> {
    type Output = WellFormed<NextState>;
}

impl<State, Request, NextState> StepWellFormed<State> for StepSuspend<Request, NextState> {
    type Output = WellFormed<NextState>;
}

impl<State, Reason> StepWellFormed<State> for StepTrap<Reason> {
    type Output = IllFormed<Reason, State>;
}

impl<Inst, State> InstrWellFormed<State> for Inst
where
    Apply<Inst, State>: Eval,
    Inst: Op<State>,
    Evaluate<Apply<Inst, State>>: StepWellFormed<State>,
{
    type Output = <Evaluate<Apply<Inst, State>> as StepWellFormed<State>>::Output;
}

pub trait ContinueProgramWellFormed {
    type Output;
}

impl<State> ContinueProgramWellFormed for WellFormed<State>
where
    State: StateWellFormed,
{
    type Output = <State as StateWellFormed>::Output;
}

impl<Reason, State> ContinueProgramWellFormed for IllFormed<Reason, State> {
    type Output = Self;
}

/// Internal full-state well-formedness proof.
pub trait StateWellFormed {
    type Output;
}

impl<Stack, Locals, Memory, Frames> StateWellFormed
    for VmState<Stack, Locals, Memory, Frames, TTerm>
{
    type Output = WellFormed<Self>;
}

impl<Stack, Locals, Memory, Frames, Inst, Rest> StateWellFormed
    for VmState<Stack, Locals, Memory, Frames, TArr<Inst, Rest>>
where
    Inst: InstrWellFormed<VmState<Stack, Locals, Memory, Frames, TArr<Inst, Rest>>>,
    <Inst as InstrWellFormed<VmState<Stack, Locals, Memory, Frames, TArr<Inst, Rest>>>>::Output:
        ContinueProgramWellFormed,
{
    type Output = <<Inst as InstrWellFormed<
        VmState<Stack, Locals, Memory, Frames, TArr<Inst, Rest>>,
    >>::Output as ContinueProgramWellFormed>::Output;
}

/// Checks whether a concrete program is statically well formed for the supplied
/// starting state.
pub trait ProgramWellFormed<State, Program> {
    type Output;
}

impl<Stack, Locals, Memory, Frames, Program>
    ProgramWellFormed<VmState<Stack, Locals, Memory, Frames, Program>, Program> for Program
where
    VmState<Stack, Locals, Memory, Frames, Program>: StateWellFormed,
{
    type Output = <VmState<Stack, Locals, Memory, Frames, Program> as StateWellFormed>::Output;
}
