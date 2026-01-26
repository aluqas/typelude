//! **Control Flow Actions**
//!
//! High-level control flow combinators: Branching (`If`) and Recursion (`Fix`).
//! Using `Apply` trait.

use std::marker::PhantomData;

use typelude_core::{Eval, Evaluate};
use typelude_std::{
    traits::Apply,
    model::prim::bool::{False, True},
    std::col::array::Array,
};

use crate::machine::semantics::{
    monad::{LReturn, Unit},
    state::VmState,
};

// =============================================================================
// If (Branching)
// =============================================================================

/// ActBranch<Then, Else>: Pops a Boolean, runs Then if True, Else if False.
pub struct ActBranch<Then, Else>(PhantomData<(Then, Else)>);

impl<Then, Else> Eval for ActBranch<Then, Else> {
    type Output = ActBranch<Then, Else>;
}

// Helper for selection
pub trait SelectAction<Then, Else> {
    type Output;
}
impl<Then, Else> SelectAction<Then, Else> for True {
    type Output = Then;
}
impl<Then, Else> SelectAction<Then, Else> for False {
    type Output = Else;
}

impl<Then, Else, Head, Tail, L, M, C> Apply<VmState<Array<Head, Tail>, L, M, C>>
    for ActBranch<Then, Else>
where
    Head: Eval + SelectAction<Then, Else>,
    Tail: Eval + typelude_std::std::col::array::IsList,
    // Selected Action
    <Head as SelectAction<Then, Else>>::Output: Apply<VmState<Evaluate<Tail>, L, M, C>>,
{
    type Output = <<Head as SelectAction<Then, Else>>::Output as Apply<
        VmState<Evaluate<Tail>, L, M, C>,
    >>::Output;
}

// =============================================================================
// Fix (Recursion / Loop)
// =============================================================================

pub struct ActFix<F>(PhantomData<F>);

impl<F> Eval for ActFix<F> {
    type Output = ActFix<F>;
}

// Apply Fix<F> to State
// Fix<F> S -> F<Fix<F>> S
impl<F, S> Apply<S> for ActFix<F>
where
    F: Apply<ActFix<F>>,                       // F applied to Fix<F>
    <F as Apply<ActFix<F>>>::Output: Apply<S>, // Resulting Action applied to S
{
    type Output = <<F as Apply<ActFix<F>>>::Output as Apply<S>>::Output;
}

// =============================================================================
// While Helper
// =============================================================================

pub struct LoopGen<CondAction, BodyAction>(PhantomData<(CondAction, BodyAction)>);

impl<CondAction, BodyAction> Eval for LoopGen<CondAction, BodyAction> {
    type Output = LoopGen<CondAction, BodyAction>;
}

// ActSeq helper
pub struct ActSeq<First, Second>(PhantomData<(First, Second)>);
impl<First, Second> Eval for ActSeq<First, Second> {
    type Output = ActSeq<First, Second>;
}

// ActSeq S -> First S -> (Val, S') -> Second S' -> Result
impl<First, Second, S, Val, NextState> Apply<S> for ActSeq<First, Second>
where
    First: Apply<S, Output = typelude_std::lambda::church::LPair2<Val, NextState>>,
    Val: Eval,
    NextState: Eval,

    // Apply Second to NextState (Pre-Evaluated/Normalized)
    // We already have NextState as a type variable, so it is "Evaluated" in the sense that it's extracted.
    // If NextState comes from LPair2, it is usually already evaluated (since Actions return (Unit, State)).
    // But to be safe and ensure normalization:
    Evaluate<NextState>: Eval,
    Second: Apply<Evaluate<NextState>>,
{
    type Output = <Second as Apply<Evaluate<NextState>>>::Output;
}



// LoopGen<C, B> Loop -> Action
impl<CondAction, BodyAction, Loop> Apply<Loop> for LoopGen<CondAction, BodyAction> {
    type Output = ActSeq<CondAction, ActBranch<ActSeq<BodyAction, Loop>, LReturn<Unit>>>;
}

pub type ActWhile<CondAction, BodyAction> = ActFix<LoopGen<CondAction, BodyAction>>;
