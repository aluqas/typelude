//! **Control Flow Adapters**
//!
//! Bridge layer between pure lambda calculus combinators and practical Rust
//! types.
//!
//! - `IntoBool`: Convert bool-like types to `LTrue`/`LFalse`
//! - `EIf`: Practical conditional using the `IntoBool` adapter
//! - `EWhile`: Practical loop using `LWhile` internally
//!
use std::marker::PhantomData;

use typelude_std::core::{EApp, Eval, Evaluate, TyFn};

use crate::{
    lambda::{
        LApp,
        church::{LFalse, LTrue, LWhile2},
    },
    std::prim::bool::IntoBool,
};
/// Practical If expression.
///
/// Evaluates `Cond`, converts to Church boolean via `IntoBool`,
/// then dispatches to `Then` or `Else` branch.
///
/// # Example
///
/// ```ignore
/// type Result = Evaluate<EIf<True, U1, U0>>; // = U1
/// ```
pub struct EIf<Cond, Then, Else>(PhantomData<(Cond, Then, Else)>);

// Helper for If dispatch based on converted Church boolean.
crate::helper_if! {
    #[doc(hidden)]
    pub trait EIfHelper<Then, Else>;
    on LTrue where [Then: Eval] => Evaluate<Then>;
    on LFalse where [Else: Eval] => Evaluate<Else>;
}

impl<Cond, Then, Else> Eval for EIf<Cond, Then, Else>
where
    Cond: Eval,
    Evaluate<Cond>: IntoBool,
    <Evaluate<Cond> as IntoBool>::Output: EIfHelper<Then, Else>,
{
    type Output = <<Evaluate<Cond> as IntoBool>::Output as EIfHelper<Then, Else>>::Output;
}

/// Practical While expression.
///
/// Wraps the pure `LWhile` combinator with `IntoBool` conversion.
///
/// `Pred` is a predicate that returns `True`/`False`.
/// `Step` is a function that transforms the state.
/// `State` is the initial/current state.
///
pub struct EWhile<Pred, Step, State>(PhantomData<(Pred, Step, State)>);

/// Adapter to convert Bool predicate output to Church boolean (via `EApp`).
pub struct ChurchifyPred<Pred>(PhantomData<Pred>);

impl<Pred> Eval for ChurchifyPred<Pred> {
    type Output = ChurchifyPred<Pred>;
}

// ChurchifyPred<Pred> S -> Church Boolean
impl<Pred, S> Eval for LApp<ChurchifyPred<Pred>, S>
where
    Pred: Eval,
    S: Eval,
    Evaluate<Pred>: TyFn<Evaluate<S>>,
    EApp<Pred, S>: Eval,
    Evaluate<EApp<Pred, S>>: IntoBool,
{
    type Output = <Evaluate<EApp<Pred, S>> as IntoBool>::Output;
}

/// Adapter for Step function in lambda world (via `EApp`).
pub struct LambdifyStep<Step>(PhantomData<Step>);

impl<Step> Eval for LambdifyStep<Step> {
    type Output = LambdifyStep<Step>;
}

// LambdifyStep<Step> S -> Step::Output (evaluated)
impl<Step, S> Eval for LApp<LambdifyStep<Step>, S>
where
    Step: Eval,
    S: Eval,
    Evaluate<Step>: TyFn<Evaluate<S>>,
    EApp<Step, S>: Eval,
{
    type Output = Evaluate<EApp<Step, S>>;
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

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_std::core::ELit;
    use typenum::{U0, U1};

    use super::*;
    use crate::std::prim::bool::{False, True};

    #[test]
    fn test_eif_true() {
        type Result = Evaluate<EIf<ELit<True>, ELit<U1>, ELit<U0>>>;
        assert_type_eq_all!(Result, U1);
    }

    #[test]
    fn test_eif_false() {
        type Result = Evaluate<EIf<ELit<False>, ELit<U1>, ELit<U0>>>;
        assert_type_eq_all!(Result, U0);
    }
}
