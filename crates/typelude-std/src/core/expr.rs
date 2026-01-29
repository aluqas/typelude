//! **Core Expression AST Nodes**
//!
//! Defines the foundational types for type-level evaluation and application.

use std::marker::PhantomData;

use crate::{Apply, Eval, Evaluate};

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

// Allow ELit<T> to act as a function T
impl<T, A> Apply<A> for ELit<T>
where
    T: Apply<A>,
{
    type Output = <T as Apply<A>>::Output;
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
    Evaluate<Ef>: Apply<Evaluate<Ea>>,
{
    type Output = <Evaluate<Ef> as Apply<Evaluate<Ea>>>::Output;
}

// Allow EApp<Ef, Ea> to act as a function if it evaluates to one.
impl<Ef, Ea, A> Apply<A> for EApp<Ef, Ea>
where
    EApp<Ef, Ea>: Eval,
    Evaluate<EApp<Ef, Ea>>: Apply<A>,
{
    type Output = <Evaluate<EApp<Ef, Ea>> as Apply<A>>::Output;
}
