//! **Control Flow Combinators (Pure Lambda Calculus)**
//!
//! Pure lambda calculus implementations of control flow:
//! - `LWhile` - Pure while loop combinator
//! - `LFor` - Pure iteration combinator (future)
//!
//! These are Church-encoded and work only with Church booleans (LTrue/LFalse).
//! For use with TyTrue/TyFalse, see `std/control.rs` adapters.

use std::marker::PhantomData;

use typelude_core::{Eval, Evaluate};

use super::bool::{LFalse, LTrue};
use crate::{
    impl_eval_for_lambda, impl_eval_for_lambda_generic,
    lambda::{LApp, Lambda},
};
/// Pure While combinator (curried).
///
/// Semantics: `while pred body state = if (pred state) then while pred body
/// (body state) else state`
///
/// Usage: `LApp<LApp<LApp<LWhile, Pred>, Body>, State>`
pub struct LWhile;

impl Lambda for LWhile {
    type Output = LWhile;
}
impl_eval_for_lambda!(LWhile);

// Partial application: LWhile Pred -> LWhile1<Pred>
pub struct LWhile1<Pred>(PhantomData<Pred>);

impl<Pred> Lambda for LWhile1<Pred> {
    type Output = LWhile1<Pred>;
}
impl_eval_for_lambda_generic!(LWhile1, [Pred]);

impl<Pred> Lambda for LApp<LWhile, Pred>
where
    Pred: Eval,
{
    type Output = LWhile1<Evaluate<Pred>>;
}

// Partial application: LWhile1<Pred> Body -> LWhile2<Pred, Body>
pub struct LWhile2<Pred, Body>(PhantomData<(Pred, Body)>);

impl<Pred, Body> Lambda for LWhile2<Pred, Body> {
    type Output = LWhile2<Pred, Body>;
}
impl_eval_for_lambda_generic!(LWhile2, [Pred, Body]);

impl<Pred, Body> Lambda for LApp<LWhile1<Pred>, Body>
where
    Body: Eval,
{
    type Output = LWhile2<Pred, Evaluate<Body>>;
}
/// Helper trait for while loop dispatch based on condition result.
pub trait LWhileHelper<Pred, Body, State> {
    type Output;
}

// Condition == LTrue: Execute body, recurse
impl<Pred, Body, State> LWhileHelper<Pred, Body, State> for LTrue
where
    // body(state) -> newState
    LApp<Body, State>: Lambda,
    // while pred body newState -> result (recursive call)
    LApp<LWhile2<Pred, Body>, <LApp<Body, State> as Lambda>::Output>: Lambda,
{
    type Output =
        <LApp<LWhile2<Pred, Body>, <LApp<Body, State> as Lambda>::Output> as Lambda>::Output;
}

// Condition == LFalse: Return current state
impl<Pred, Body, State> LWhileHelper<Pred, Body, State> for LFalse {
    type Output = State;
}

// Full application: LWhile2<Pred, Body> State -> if (pred state) recurse else
// state
impl<Pred, Body, State> Lambda for LApp<LWhile2<Pred, Body>, State>
where
    // pred(state) -> LTrue/LFalse
    LApp<Pred, State>: Lambda,
    <LApp<Pred, State> as Lambda>::Output: LWhileHelper<Pred, Body, State>,
{
    type Output =
        <<LApp<Pred, State> as Lambda>::Output as LWhileHelper<Pred, Body, State>>::Output;
}
/// Pure For combinator: iterate over Church list, applying function to each
/// element.
///
/// Semantics: `for f list = foldr (\x acc -> f x >> acc) () list`
///
/// (To be implemented when Church lists are needed)
pub struct LFor;

impl Lambda for LFor {
    type Output = LFor;
}
impl_eval_for_lambda!(LFor);

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::lambda::church::{LSucc, LZero};

    // Helper: Apply alias
    type App<F, A> = Evaluate<LApp<F, A>>;

    // Predicate: IsZero (returns LTrue if zero, LFalse otherwise)
    pub struct LIsZero;
    impl Lambda for LIsZero {
        type Output = LIsZero;
    }
    impl_eval_for_lambda!(LIsZero);

    // IsZero Zero -> True
    impl Lambda for LApp<LIsZero, LZero> {
        type Output = LTrue;
    }

    // IsZero (Succ n) -> False
    impl<N> Lambda for LApp<LIsZero, LSucc<N>> {
        type Output = LFalse;
    }

    // NotZero: negation of IsZero
    pub struct LNotZero;
    impl Lambda for LNotZero {
        type Output = LNotZero;
    }
    impl_eval_for_lambda!(LNotZero);

    impl Lambda for LApp<LNotZero, LZero> {
        type Output = LFalse;
    }

    impl<N> Lambda for LApp<LNotZero, LSucc<N>> {
        type Output = LTrue;
    }

    // Decrement: Pred
    pub struct LDecr;
    impl Lambda for LDecr {
        type Output = LDecr;
    }
    impl_eval_for_lambda!(LDecr);

    impl Lambda for LApp<LDecr, LZero> {
        type Output = LZero;
    }

    impl<N> Lambda for LApp<LDecr, LSucc<N>> {
        type Output = N;
    }

    #[test]
    fn test_while_countdown() {
        // while (notZero) (decr) 2 -> 0
        type Two = LSucc<LSucc<LZero>>;
        type WhilePred = LNotZero;
        type WhileBody = LDecr;

        // Apply: LWhile NotZero Decr 2
        type Result = App<App<App<LWhile, WhilePred>, WhileBody>, Two>;

        assert_type_eq_all!(Result, LZero);
    }

    #[test]
    fn test_while_already_zero() {
        // while (notZero) (decr) 0 -> 0 (no iteration)
        type WhilePred = LNotZero;
        type WhileBody = LDecr;

        type Result = App<App<App<LWhile, WhilePred>, WhileBody>, LZero>;

        assert_type_eq_all!(Result, LZero);
    }
}
