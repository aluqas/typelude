//! **Expression Types**
//!
//! Defines the syntax for type-level expressions.
//! - `EIf`: Conditional Branch
//! - `EWhile`: Loop
//!
//! Note: `ELit` is imported from `bridge`. `App` is imported from `app`.

use std::marker::PhantomData;

use super::{Eval, Evaluate};
use crate::kernel::{
    bool::{TyFalse, TyTrue},
    traits::Apply,
};

//
// EIf: Conditional Expression
//

/// Conditional Expression
///
/// # Example
/// ```ignore
/// type Result = Evaluator<EIf<ELit<TyTrue>, ELit<i32>, ELit<f64>>>;
/// // Result = i32
/// ```
pub struct EIf<Cond, Then, Else>(PhantomData<(Cond, Then, Else)>);

/// Helper implemented based on Cond result (True/False)
#[doc(hidden)]
pub trait IfHelper<Then, Else> {
    type Output;
}

impl<Then, Else> IfHelper<Then, Else> for TyTrue
where
    Then: Eval,
{
    type Output = Then::Output;
}

impl<Then, Else> IfHelper<Then, Else> for TyFalse
where
    Else: Eval,
{
    type Output = Else::Output;
}

impl<Cond, Then, Else> Eval for EIf<Cond, Then, Else>
where
    Cond: Eval,
    Evaluate<Cond>: IfHelper<Then, Else>,
{
    type Output = <Evaluate<Cond> as IfHelper<Then, Else>>::Output;
}

//
// EWhile: Loop Expression
//

type AppliedOutput<Op, A> = <Op as Apply<A>>::Output;

/// Expression representing a While loop
///
/// - `Pred`: Condition (Val -> TyTrue/TyFalse)
/// - `Step`: Update function (Val -> NextVal)
/// - `State`: Initial State Expression (evaluates to Val)
///
/// Important: `EWhile` normalizes (evaluates) the state at each step.
/// So `Pred` and `Step` receive the *Value*, not the *Expression*.
pub struct EWhile<Pred, Step, State>(PhantomData<(Pred, Step, State)>);

#[doc(hidden)]
pub trait WhileHelper<Pred, Step, Val> {
    type Output;
}

// Condition == True: Recurse
impl<Pred, Step, Val> WhileHelper<Pred, Step, Val> for TyTrue
where
    // Step applies to Value
    Step: Apply<Val>,
    // Step returns an Expression (which we must evaluate for the next loop)
    Step::Output: Eval,
    // Recursive call: Next state is Evaluated Step Output
    // Note: We pass ELit<Val> or just Val?
    // Since EWhile now takes "State: Eval", and Step::Output is Eval,
    // we can pass Step::Output directly as the next State.
    EWhile<Pred, Step, Step::Output>: Eval,
{
    type Output = <EWhile<Pred, Step, Step::Output> as Eval>::Output;
}

// Condition == False: Terminate
impl<Pred, Step, Val> WhileHelper<Pred, Step, Val> for TyFalse {
    type Output = Val;
}

impl<Pred, Step, State> Eval for EWhile<Pred, Step, State>
where
    // 1. Evaluate current State
    State: Eval,
    // 2. Apply Predicate to the Value
    Pred: Apply<Evaluate<State>>,
    // 3. Evaluate Predicate Result (to get TyTrue/TyFalse)
    AppliedOutput<Pred, Evaluate<State>>: Eval,
    // 4. Dispatch based on condition
    Evaluate<AppliedOutput<Pred, Evaluate<State>>>: WhileHelper<Pred, Step, Evaluate<State>>,
{
    type Output = <Evaluate<AppliedOutput<Pred, Evaluate<State>>> as WhileHelper<
        Pred,
        Step,
        Evaluate<State>,
    >>::Output;
}
