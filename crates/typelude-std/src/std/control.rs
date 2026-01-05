//! **Control Flow Adapters**
//!
//! Bridge layer between pure lambda calculus combinators and practical Rust types.
//!
//! - `ToChurch`: Convert `TyTrue`/`TyFalse` to `LTrue`/`LFalse`
//! - `EIf`: Practical conditional using `ToChurch` adapter
//! - `EWhile`: Practical loop using `LWhile` internally
//!
//! # Architecture
//!
//! ```text
//! User Code → std/control (this module) → lambda/church (pure)
//!              ↓                            ↓
//!           TyTrue/TyFalse              LTrue/LFalse
//! ```

use std::marker::PhantomData;

use typelude_core::{Apply, Eval, Evaluate};

use crate::{
    data::primitives::bool::{False, True},
    lambda::{
        LApp, Lambda,
        church::{LFalse, LTrue, LWhile2},
    },
};
/// Convert type-level booleans to Church booleans.
///
/// This adapter bridges the gap between practical boolean types
/// (`True`/`False`) and pure lambda calculus (`LTrue`/`LFalse`).
pub trait ToChurch {
    type Church: Lambda;
}

impl ToChurch for True {
    type Church = LTrue;
}

impl ToChurch for False {
    type Church = LFalse;
}
/// Practical If expression.
///
/// Evaluates `Cond`, converts to Church boolean via `ToChurch`,
/// then dispatches to `Then` or `Else` branch.
///
/// # Example
///
/// ```ignore
/// type Result = Evaluate<EIf<True, U1, U0>>; // = U1
/// ```
pub struct EIf<Cond, Then, Else>(PhantomData<(Cond, Then, Else)>);

/// Helper for If dispatch based on converted Church boolean.
pub trait EIfHelper<Then, Else> {
    type Output;
}

// LTrue -> evaluate Then
impl<Then, Else> EIfHelper<Then, Else> for LTrue
where
    Then: Eval,
{
    type Output = Evaluate<Then>;
}

// LFalse -> evaluate Else
impl<Then, Else> EIfHelper<Then, Else> for LFalse
where
    Else: Eval,
{
    type Output = Evaluate<Else>;
}

impl<Cond, Then, Else> Eval for EIf<Cond, Then, Else>
where
    Cond: Eval,
    Evaluate<Cond>: ToChurch,
    <Evaluate<Cond> as ToChurch>::Church: EIfHelper<Then, Else>,
{
    type Output = <<Evaluate<Cond> as ToChurch>::Church as EIfHelper<Then, Else>>::Output;
}

// Op wrapper for Apply pattern
pub struct OpIf;

impl<Cond, Then, Else> Apply<(Cond, Then, Else)> for OpIf {
    type Output = EIf<Cond, Then, Else>;
}
/// Practical While expression.
///
/// Wraps the pure `LWhile` combinator with `ToChurch` conversion.
///
/// `Pred` is a predicate that returns `True`/`False`.
/// `Step` is a function that transforms the state.
/// `State` is the initial/current state.
///
/// # Semantics
///
/// ```text
/// while (pred state == True) { state = step(state) }
/// return state
/// ```
pub struct EWhile<Pred, Step, State>(PhantomData<(Pred, Step, State)>);

/// Adapter to convert Bool predicate output to Church boolean.
pub struct ChurchifyPred<Pred>(PhantomData<Pred>);

impl<Pred> Lambda for ChurchifyPred<Pred> {
    type Output = ChurchifyPred<Pred>;
}

impl<Pred> Eval for ChurchifyPred<Pred> {
    type Output = Self;
}

// ChurchifyPred<Pred> S -> Church Boolean
impl<Pred, S> Lambda for LApp<ChurchifyPred<Pred>, S>
where
    Pred: Eval,
    S: Eval,
    // Pred applied to S
    Pred: Apply<Evaluate<S>>,
    // Pred(S) returns Bool
    <Pred as Apply<Evaluate<S>>>::Output: Eval,
    Evaluate<<Pred as Apply<Evaluate<S>>>::Output>: ToChurch,
{
    type Output = <Evaluate<<Pred as Apply<Evaluate<S>>>::Output> as ToChurch>::Church;
}

/// Adapter for Step function in lambda world.
pub struct LambdifyStep<Step>(PhantomData<Step>);

impl<Step> Lambda for LambdifyStep<Step> {
    type Output = LambdifyStep<Step>;
}

impl<Step> Eval for LambdifyStep<Step> {
    type Output = Self;
}

// LambdifyStep<Step> S -> Step::Output (evaluated)
impl<Step, S> Lambda for LApp<LambdifyStep<Step>, S>
where
    Step: Eval,
    S: Eval,
    Step: Apply<Evaluate<S>>,
    <Step as Apply<Evaluate<S>>>::Output: Eval,
{
    type Output = Evaluate<<Step as Apply<Evaluate<S>>>::Output>;
}

impl<Pred, Step, State> Eval for EWhile<Pred, Step, State>
where
    Pred: Eval,
    Step: Eval,
    State: Eval,
    // Use LWhile2 directly with adapted pred and step
    LApp<LWhile2<ChurchifyPred<Pred>, LambdifyStep<Step>>, Evaluate<State>>: Lambda,
{
    type Output =
        <LApp<LWhile2<ChurchifyPred<Pred>, LambdifyStep<Step>>, Evaluate<State>> as Lambda>::Output;
}

// Op wrapper
pub struct OpWhile;

impl<Pred, Step, State> Apply<(Pred, Step, State)> for OpWhile {
    type Output = EWhile<Pred, Step, State>;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typenum::{U0, U1, U2, U3};

    use super::*;
    use crate::std::ops::{ELt, ESub};

    #[test]
    fn test_eif_true() {
        type Result = Evaluate<EIf<True, U1, U0>>;
        assert_type_eq_all!(Result, U1);
    }

    #[test]
    fn test_eif_false() {
        type Result = Evaluate<EIf<False, U1, U0>>;
        assert_type_eq_all!(Result, U0);
    }

    // Note: EWhile tests require Apply-compatible predicates/steps
    // which need more infrastructure. Basic EIf tests validate ToChurch bridge.
}
