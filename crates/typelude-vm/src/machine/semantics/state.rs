//! **Machine State & Lenses**
//!
//! Defines the structure of the virtual machine state and provides
//! Lambda-based Lenses (Getters/Setters) to access its components.

use std::marker::PhantomData;

use typelude_core::{Eval, Evaluate};
use typelude_std::{
    traits::Apply, // Use Apply instead of LApp impl
};

// =============================================================================
// Machine State Definition
// =============================================================================

#[derive(Debug)]
pub struct VmState<Stack, Locals, Memory, CallStack>(
    pub PhantomData<(Stack, Locals, Memory, CallStack)>,
);

impl<S, L, M, C> Eval for VmState<S, L, M, C> {
    type Output = Self;
}

// =============================================================================
// Lenses (Getters)
// =============================================================================

// --- GetStack ---
pub struct LGetStack;
impl Eval for LGetStack {
    type Output = Self;
}

// LGetStack VmState<S...> -> S
impl<S, L, M, C> Apply<VmState<S, L, M, C>> for LGetStack
where
    S: Eval,
{
    type Output = Evaluate<S>;
}

// --- GetLocals ---
pub struct LGetLocals;
impl Eval for LGetLocals {
    type Output = Self;
}
impl<S, L, M, C> Apply<VmState<S, L, M, C>> for LGetLocals
where
    L: Eval,
{
    type Output = Evaluate<L>;
}

// --- GetMemory ---
pub struct LGetMemory;
impl Eval for LGetMemory {
    type Output = Self;
}
impl<S, L, M, C> Apply<VmState<S, L, M, C>> for LGetMemory
where
    M: Eval,
{
    type Output = Evaluate<M>;
}

// --- GetCallStack ---
pub struct LGetCallStack;
impl Eval for LGetCallStack {
    type Output = Self;
}
impl<S, L, M, C> Apply<VmState<S, L, M, C>> for LGetCallStack
where
    C: Eval,
{
    type Output = Evaluate<C>;
}

// =============================================================================
// Lenses (Setters)
// =============================================================================

// --- SetStack ---
pub struct LSetStack<NewS>(PhantomData<NewS>);
impl<NewS> Eval for LSetStack<NewS> {
    type Output = Self;
}

// SetStack<New> VmState<Old...> -> VmState<New, ...>
impl<NewS, S, L, M, C> Apply<VmState<S, L, M, C>> for LSetStack<NewS>
where
    NewS: Eval,
{
    type Output = VmState<Evaluate<NewS>, L, M, C>;
}

// --- SetLocals ---
pub struct LSetLocals<NewL>(PhantomData<NewL>);
impl<NewL> Eval for LSetLocals<NewL> {
    type Output = Self;
}
impl<NewL, S, L, M, C> Apply<VmState<S, L, M, C>> for LSetLocals<NewL>
where
    NewL: Eval,
{
    type Output = VmState<S, Evaluate<NewL>, M, C>;
}

// --- SetMemory ---
pub struct LSetMemory<NewM>(PhantomData<NewM>);
impl<NewM> Eval for LSetMemory<NewM> {
    type Output = Self;
}
impl<NewM, S, L, M, C> Apply<VmState<S, L, M, C>> for LSetMemory<NewM>
where
    NewM: Eval,
{
    type Output = VmState<S, L, Evaluate<NewM>, C>;
}

// --- SetCallStack ---
pub struct LSetCallStack<NewC>(PhantomData<NewC>);
impl<NewC> Eval for LSetCallStack<NewC> {
    type Output = Self;
}
impl<NewC, S, L, M, C> Apply<VmState<S, L, M, C>> for LSetCallStack<NewC>
where
    NewC: Eval,
{
    type Output = VmState<S, L, M, Evaluate<NewC>>;
}
