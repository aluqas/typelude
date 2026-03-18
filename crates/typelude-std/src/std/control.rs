//! **Control Flow Adapters**
//!
//! Bridge layer between pure lambda calculus combinators and practical Rust
//! types.
//!
//! - `EIf`: Practical conditional using `True`/`False`
//! - `EWhile`: Practical loop recursion over type states
use std::marker::PhantomData;

use typelude_std::core::{EApp, Eval, Evaluate, TyFn};

use crate::std::prim::bool::{False, True};

/// Practical If expression.
///
/// Evaluates `Cond` and dispatches to `Then` or `Else` branch.
///
/// # Example
///
/// ```ignore
/// type Result = Evaluate<EIf<True, U1, U0>>; // = U1
/// ```
pub struct EIf<Cond, Then, Else>(PhantomData<(Cond, Then, Else)>);

crate::helper_if! {
    #[doc(hidden)]
    pub trait EIfHelper<Then, Else>;
    on True where [Then: Eval] => Evaluate<Then>;
    on False where [Else: Eval] => Evaluate<Else>;
}

impl<Cond, Then, Else> Eval for EIf<Cond, Then, Else>
where
    Cond: Eval,
    Evaluate<Cond>: EIfHelper<Then, Else>,
{
    type Output = <Evaluate<Cond> as EIfHelper<Then, Else>>::Output;
}

/// Practical While expression.
///
/// `Pred` is a predicate that returns `True`/`False`.
/// `Step` is a function that transforms the state.
/// `State` is the initial/current state.
pub struct EWhile<Pred, Step, State>(PhantomData<(Pred, Step, State)>);

#[doc(hidden)]
pub trait EWhileHelper<Pred, Step, State> {
    type Output;
}

impl<Pred, Step, State> EWhileHelper<Pred, Step, State> for False
where
    State: Eval,
{
    type Output = Evaluate<State>;
}

impl<Pred, Step, State> EWhileHelper<Pred, Step, State> for True
where
    State: Eval,
    Step: Eval,
    Evaluate<Step>: TyFn<Evaluate<State>>,
    EApp<Step, State>: Eval,
    Pred: Eval,
    Evaluate<Pred>: TyFn<Evaluate<EApp<Step, State>>>,
    EApp<Pred, EApp<Step, State>>: Eval,
    Evaluate<EApp<Pred, EApp<Step, State>>>: EWhileHelper<Pred, Step, Evaluate<EApp<Step, State>>>,
{
    type Output = <Evaluate<EApp<Pred, EApp<Step, State>>> as EWhileHelper<
        Pred,
        Step,
        Evaluate<EApp<Step, State>>,
    >>::Output;
}

impl<Pred, Step, State> Eval for EWhile<Pred, Step, State>
where
    Pred: Eval,
    State: Eval,
    Evaluate<Pred>: TyFn<Evaluate<State>>,
    EApp<Pred, State>: Eval,
    Evaluate<EApp<Pred, State>>: EWhileHelper<Pred, Step, Evaluate<State>>,
{
    type Output =
        <Evaluate<EApp<Pred, State>> as EWhileHelper<Pred, Step, Evaluate<State>>>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typelude_std::core::ELit;
    use typenum::{U0, U1};

    use super::*;

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
