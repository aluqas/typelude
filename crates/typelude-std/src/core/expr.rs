//! **Core Expression AST Nodes**
//!
//! Defines the foundational types for type-level evaluation and application.

use std::marker::PhantomData;

use crate::{Eval, Evaluate, core::TyFn};

/// **Identity Evaluator**: Lifts a value `T` into an expression `ELit<T>`.
///
/// Evaluation just returns `T`.
///
/// # Examples
///
/// ```ignore
/// type Res = Evaluate<ELit<i32>>; // i32
/// ```
pub struct ELit<T>(PhantomData<T>);

impl<T> Eval for ELit<T> {
    type Output = T;
}

// Allow ELit<T> to act as a function T.
impl<T, A> TyFn<A> for ELit<T>
where
    T: TyFn<A>,
{
    type Output = <T as TyFn<A>>::Output;
}

/// **Call-by-Value Application**: `EApp<Ef, Ea>`
///
/// 1. Evaluate `Ef` -> `F_val`
/// 2. Evaluate `Ea` -> `A_val`
/// 3. Apply `F_val` to `A_val` -> `Result`
///
/// This is safer for higher-order programming where functions are expressions.
///
/// # Examples
///
/// ```ignore
/// type Res = Evaluate<EApp<ELit<OpAdd>, ELit<U1>>>;
/// ```
pub struct EApp<Ef, Ea>(PhantomData<(Ef, Ea)>);

impl<Ef, Ea> Eval for EApp<Ef, Ea>
where
    Ef: Eval,
    Ea: Eval,
    Evaluate<Ef>: TyFn<Evaluate<Ea>>,
{
    type Output = <Evaluate<Ef> as TyFn<Evaluate<Ea>>>::Output;
}

// Allow EApp<Ef, Ea> to act as a function if it evaluates to one.
impl<Ef, Ea, A> TyFn<A> for EApp<Ef, Ea>
where
    EApp<Ef, Ea>: Eval,
    Evaluate<EApp<Ef, Ea>>: TyFn<A>,
{
    type Output = <Evaluate<EApp<Ef, Ea>> as TyFn<A>>::Output;
}

/// Sugar for calling a first-class type-level function `F` with argument `A`.
pub type ECall<F, A> = EApp<ELit<F>, A>;

/// Sugar for calling a binary first-class type-level function `F`.
pub type ECall2<F, A, B> = EApp<ECall<F, A>, B>;

/// Sugar for calling a ternary first-class type-level function `F`.
pub type ECall3<F, A, B, C> = EApp<ECall2<F, A, B>, C>;
