//! **Expression Types**
//!
//! Defines the syntax for type-level expressions.
//! - `ELit`: Literal
//! - `EApply`, `EApply2`: Function Application
//! - `EIf`: Conditional Branch
//! - `EWhile`: Loop

use std::marker::PhantomData;

use super::{Evaluable, Evaluator};
use crate::std::{
    bool::{TyFalse, TyTrue},
    traits::EFunction,
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

impl<T> Evaluable for ELit<T> {
    type Output = T;
}

//
// EApply: Function Application
//

/// 1-argument function application
///
/// # Example
/// ```ignore
/// type LenExpr = EApply<FLen, ELit<MyArray>>;
/// ```
pub struct EApply<F, A>(PhantomData<(F, A)>);

/// 2-argument function application
///
/// # Example
/// ```ignore
/// type ConcatExpr = EApply2<FConcat, ELit<ArrayA>, ELit<ArrayB>>;
/// ```
pub struct EApply2<F, A, B>(PhantomData<(F, A, B)>);

/// 3-argument function application
///
/// # Example
/// ```ignore
/// type SetExpr = EApply3<FSet, ELit<Array>, ELit<Index>, ELit<Value>>;
/// ```
pub struct EApply3<F, A, B, C>(PhantomData<(F, A, B, C)>);

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
pub trait _EIfHelper<Then, Else> {
    type Output;
}

impl<Then, Else> _EIfHelper<Then, Else> for TyTrue
where
    Then: Evaluable,
{
    type Output = Then::Output;
}

impl<Then, Else> _EIfHelper<Then, Else> for TyFalse
where
    Else: Evaluable,
{
    type Output = Else::Output;
}

impl<Cond, Then, Else> Evaluable for EIf<Cond, Then, Else>
where
    Cond: Evaluable,
    Evaluator<Cond>: _EIfHelper<Then, Else>,
{
    type Output = <Evaluator<Cond> as _EIfHelper<Then, Else>>::Output;
}

//
// EWhile: Loop Expression
//

type AppliedOutput<F, A> = <F as EFunction<A>>::Output;

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
pub trait _EWhileHelper<Pred, Step, State> {
    type Output;
}

// Condition == True: Recurse
impl<Pred, Step, State> _EWhileHelper<Pred, Step, State> for TyTrue
where
    Step: EFunction<State>,
    Step::Output: Evaluable,
    EWhile<Pred, Step, AppliedOutput<Step, State>>: Evaluable,
{
    type Output = <EWhile<Pred, Step, AppliedOutput<Step, State>> as Evaluable>::Output;
}

// Condition == False: Terminate
impl<Pred, Step, State> _EWhileHelper<Pred, Step, State> for TyFalse
where
    State: Evaluable,
{
    type Output = Evaluator<State>;
}

impl<Pred, Step, State> Evaluable for EWhile<Pred, Step, State>
where
    Pred: EFunction<State>,
    AppliedOutput<Pred, State>: Evaluable,
    Evaluator<AppliedOutput<Pred, State>>: _EWhileHelper<Pred, Step, State>,
{
    type Output =
        <Evaluator<AppliedOutput<Pred, State>> as _EWhileHelper<Pred, Step, State>>::Output;
}
