use std::marker::PhantomData;

use crate::{Apply, Eval, Evaluate};

// =========================================================================
// Bridges: Connecting Apply (Pure) and Eval (System)
// =========================================================================

/// **Identity Evaluator**: Lifts a value `T` into an expression `ELit<T>`.
/// Evaluation just returns `T`.
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

// =========================================================================
// EApp: Call-by-Value Application (New, for Lambda integration)
// =========================================================================

/// **Call-by-Value Application**: `EApp<Ef, Ea>`
///
/// 1. Evaluate `Ef` -> `F_val`
/// 2. Evaluate `Ea` -> `A_val`
/// 3. Apply `F_val` to `A_val` -> `Result`
///
/// This is safer for higher-order programming where functions are expressions.
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

// =========================================================================
// ELazyApp: Call-by-Name Application
// =========================================================================

/// **Call-by-Name Application**: `ELazyApp<Ef, Ea>`
///
/// 1. Evaluate `Ef` -> `F_val`
/// 2. Apply `F_val` to `Ea` (unevaluated) -> `Result`
pub struct ELazyApp<Ef, Ea>(PhantomData<(Ef, Ea)>);

impl<Ef, Ea> Eval for ELazyApp<Ef, Ea>
where
    Ef: Eval,
    Evaluate<Ef>: Apply<Ea>,
{
    type Output = <Evaluate<Ef> as Apply<Ea>>::Output;
}

// =========================================================================
// EPureApp: Strict Application (No Eval)
// =========================================================================

/// **Pure Application Wrapper**: `EPureApp<F, A>`
///
/// Simply runs `Apply`. Result is NOT re-evaluated.
/// `Evaluate<EPureApp<F, A>>` = `F::Output`.
pub struct EPureApp<F, A>(PhantomData<(F, A)>);

impl<F, A> Eval for EPureApp<F, A>
where
    F: Apply<A>,
{
    type Output = <F as Apply<A>>::Output;
}
