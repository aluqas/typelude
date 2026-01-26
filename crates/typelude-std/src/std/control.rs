//! **Control Flow Adapters**
//!
//! Bridge layer between pure lambda calculus combinators and practical Rust
//! types.
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
    lambda::{
        LApp,
        church::{LFalse, LTrue, LWhile2},
    },
    std::prim::bool::IntoBool,
};
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
    Evaluate<Cond>: IntoBool,
    <Evaluate<Cond> as IntoBool>::Output: EIfHelper<Then, Else>,
{
    type Output = <<Evaluate<Cond> as IntoBool>::Output as EIfHelper<Then, Else>>::Output;
}

// Op wrapper for Apply pattern
pub struct OpIf;

impl<Cond, Then, Else>
    Apply<
        typelude_core::ECons<
            Cond,
            typelude_core::ECons<Then, typelude_core::ECons<Else, typelude_core::ENil>>,
        >,
    > for OpIf
{
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

impl<Pred> Eval for ChurchifyPred<Pred> {
    type Output = ChurchifyPred<Pred>;
}

// ChurchifyPred<Pred> S -> Church Boolean
impl<Pred, S> Eval for LApp<ChurchifyPred<Pred>, S>
where
    Pred: Eval,
    S: Eval,
    // Pred applied to S
    Pred: Apply<Evaluate<S>>,
    // Pred(S) returns Bool
    <Pred as Apply<Evaluate<S>>>::Output: Eval,
    Evaluate<<Pred as Apply<Evaluate<S>>>::Output>: IntoBool,
{
    type Output = <Evaluate<<Pred as Apply<Evaluate<S>>>::Output> as IntoBool>::Output;
}

/// Adapter for Step function in lambda world.
pub struct LambdifyStep<Step>(PhantomData<Step>);

impl<Step> Eval for LambdifyStep<Step> {
    type Output = LambdifyStep<Step>;
}

// LambdifyStep<Step> S -> Step::Output (evaluated)
impl<Step, S> Eval for LApp<LambdifyStep<Step>, S>
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
    LApp<LWhile2<ChurchifyPred<Pred>, LambdifyStep<Step>>, Evaluate<State>>: Eval,
{
    type Output =
        Evaluate<LApp<LWhile2<ChurchifyPred<Pred>, LambdifyStep<Step>>, Evaluate<State>>>;
}

// Op wrapper
pub struct OpWhile;

impl<Pred, Step, State>
    Apply<
        typelude_core::ECons<
            Pred,
            typelude_core::ECons<Step, typelude_core::ECons<State, typelude_core::ENil>>,
        >,
    > for OpWhile
{
    type Output = EWhile<Pred, Step, State>;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_core::{ECons, ENil};
    use typenum::{U0, U1};

    use super::*;
    use crate::std::prim::bool::{False, True};

    #[test]
    fn test_eif_true() {
        use typelude_core::ELit;
        type Result = Evaluate<EIf<ELit<True>, ELit<U1>, ELit<U0>>>;
        assert_type_eq_all!(Result, U1);
    }

    #[test]
    fn test_eif_false() {
        use typelude_core::ELit;
        type Result = Evaluate<EIf<ELit<False>, ELit<U1>, ELit<U0>>>;
        assert_type_eq_all!(Result, U0);
    }

    // Verify Op wrappers use EList correctly
    #[test]
    fn test_op_if_apply() {
        use typelude_core::ELit;
        type Args = ECons<ELit<True>, ECons<ELit<U1>, ECons<ELit<U0>, ENil>>>;
        type Result = <OpIf as Apply<Args>>::Output;
        assert_type_eq_all!(Result, EIf<ELit<True>, ELit<U1>, ELit<U0>>);
    }
}
