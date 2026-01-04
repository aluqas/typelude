//! **Expression Types**
//!
//! Defines the syntax for type-level expressions.
//! - `EIf`: Conditional Branch
//! - `EWhile`: Loop

use std::marker::PhantomData;

use crate::{
    eval::{Eval, Evaluate},
    kernel::{
        bool::{TyFalse, TyTrue},
        traits::Apply,
    },
};

// Wait, the previous `expr.rs` implemented `EIf` and `EWhile` directly.
// The new plan says `EIf` should be generic.
// But for now, I should restore the functionality.
// `EIf` in `expr.rs` used `TyTrue`/`TyFalse` directly.

//
// EIf: Conditional Expression
//

pub struct EIf<Cond, Then, Else>(PhantomData<(Cond, Then, Else)>);

/// Helper implemented based on Cond result (True/False)
#[doc(hidden)]
pub trait IfHelperLocal<Then, Else> {
    type Output;
}

impl<Then, Else> IfHelperLocal<Then, Else> for TyTrue
where
    Then: Eval,
{
    type Output = Then::Output;
}

impl<Then, Else> IfHelperLocal<Then, Else> for TyFalse
where
    Else: Eval,
{
    type Output = Else::Output;
}

impl<Cond, Then, Else> Eval for EIf<Cond, Then, Else>
where
    Cond: Eval,
    Evaluate<Cond>: IfHelperLocal<Then, Else>,
{
    type Output = <Evaluate<Cond> as IfHelperLocal<Then, Else>>::Output;
}

//
// EWhile: Loop Expression
//

type AppliedOutput<Op, A> = <Op as Apply<A>>::Output;

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
