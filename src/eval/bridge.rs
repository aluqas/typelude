use std::marker::PhantomData;

use crate::{
    eval::{Eval, Evaluate},
    lambda::Apply,
};

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
// EApp: Classic Application (Compatible with existing code)
// =========================================================================

/// **Function Application**: `EApp<Op, Arg>`
///
/// This applies `Op` to `Arg`.
/// It expects `Op` to implement `Apply<Arg>`.
/// The result of the application must be an expression that implements `Eval`.
///
/// This is primarily used when `Op` is a raw type (e.g. `OpAdd`) and `Arg` is a type.
/// It behaves somewhat like a macro expansion: Op(Arg) -> Expr -> Value.
pub struct EApp<Op, Arg>(PhantomData<(Op, Arg)>);

impl<Op, Arg> Eval for EApp<Op, Arg>
where
    Op: Apply<Arg>,
    Op::Output: Eval,
{
    type Output = Evaluate<Op::Output>;
}

// =========================================================================
// ECall: Call-by-Value Application (New, for Lambda integration)
// =========================================================================

/// **Call-by-Value Application**: `ECall<Ef, Ea>`
///
/// 1. Evaluate `Ef` -> `F_val`
/// 2. Evaluate `Ea` -> `A_val`
/// 3. Apply `F_val` to `A_val` -> `Result`
///
/// This is safer for higher-order programming where functions are expressions.
pub struct ECall<Ef, Ea>(PhantomData<(Ef, Ea)>);

impl<Ef, Ea> Eval for ECall<Ef, Ea>
where
    Ef: Eval,
    Ea: Eval,
    Evaluate<Ef>: Apply<Evaluate<Ea>>,
{
    type Output = <Evaluate<Ef> as Apply<Evaluate<Ea>>>::Output;
}

// Allow ECall<Ef, Ea> to act as a function if it evaluates to one.
impl<Ef, Ea, A> Apply<A> for ECall<Ef, Ea>
where
    ECall<Ef, Ea>: Eval,
    Evaluate<ECall<Ef, Ea>>: Apply<A>,
{
    type Output = <Evaluate<ECall<Ef, Ea>> as Apply<A>>::Output;
}

// =========================================================================
// ELazyCall: Call-by-Name Application
// =========================================================================

/// **Call-by-Name Application**: `ELazyCall<Ef, Ea>`
///
/// 1. Evaluate `Ef` -> `F_val`
/// 2. Apply `F_val` to `Ea` (unevaluated) -> `Result`
pub struct ELazyCall<Ef, Ea>(PhantomData<(Ef, Ea)>);

impl<Ef, Ea> Eval for ELazyCall<Ef, Ea>
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
