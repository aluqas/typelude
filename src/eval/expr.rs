//! **Expression Types**
//!
//! Defines the syntax for type-level expressions.
//! - `ELit`: Literal
//! - `EApp`: Function Application (formerly `EApply`)
//! - `EIf`: Conditional Branch
//! - `EWhile`: Loop

use std::marker::PhantomData;

use super::{Eval, Evaluate};
use crate::kernel::{
    bool::{TyFalse, TyTrue},
    traits::Apply,
};

//
// ELit: Literal Expression
//

/// Literal Expression - Embeds a concrete type as an expression
///
/// # Example
/// ```ignore
/// type Five = ELit<typenum::U5>;
/// assert_type_eq!(Evaluator<Five>, typenum::U5);
/// ```
pub struct ELit<T>(PhantomData<T>);

impl<T> Eval for ELit<T> {
    type Output = T;
}

//
// EApp: Function Application (formerly EApply)
//

/// 1-argument function application
///
/// Use `Call` alias if preferred.
pub struct EApp<Op, Arg>(PhantomData<(Op, Arg)>);

impl<Op, Arg> Eval for EApp<Op, Arg>
where
    Op: Apply<Arg>,
    Op::Output: Eval,
{
    type Output = Evaluate<Op::Output>;
}

/// 2-argument function application alias
pub type EApp2<Op, A, B> = EApp<Op, (A, B)>;

/// 3-argument function application alias
pub type EApp3<Op, A, B, C> = EApp<Op, (A, B, C)>;

/// 3-argument function application
///
/// # Example
/// ```ignore
/// type SetExpr = EApply3<FSet, ELit<Array>, ELit<Index>, ELit<Value>>;
/// ```
pub struct EApply3<F, Arg1, Arg2, Arg3>(PhantomData<(F, Arg1, Arg2, Arg3)>);

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
/// - `Pred`: Condition (State → TyTrue/TyFalse)
/// - `Step`: Update function (State → NextState)
/// - `State`: Current state
///
/// # Example
/// ```ignore
/// // while (x < 10) { x = x + 1 }
/// type Result = Evaluator<EWhile<IsLessThan10, PlusOne, U1>>;
/// // Result = U10
/// ```
pub struct EWhile<Pred, Step, State>(PhantomData<(Pred, Step, State)>);

#[doc(hidden)]
pub trait WhileHelper<Pred, Step, State> {
    type Output;
}

// Condition == True: Recurse
impl<Pred, Step, State> WhileHelper<Pred, Step, State> for TyTrue
where
    Step: Apply<State>,
    Step::Output: Eval,
    EWhile<Pred, Step, AppliedOutput<Step, State>>: Eval,
{
    type Output = <EWhile<Pred, Step, AppliedOutput<Step, State>> as Eval>::Output;
}

// Condition == False: Terminate
impl<Pred, Step, State> WhileHelper<Pred, Step, State> for TyFalse
where
    State: Eval,
{
    type Output = Evaluate<State>;
}

impl<Pred, Step, State> Eval for EWhile<Pred, Step, State>
where
    Pred: Apply<State>,
    AppliedOutput<Pred, State>: Eval,
    Evaluate<AppliedOutput<Pred, State>>: WhileHelper<Pred, Step, State>,
{
    type Output = <Evaluate<AppliedOutput<Pred, State>> as WhileHelper<Pred, Step, State>>::Output;
}
