//! Either Monad
//!
//! `Left<L>` / `Right<R>` for error handling with short-circuit behavior.

use std::marker::PhantomData;

use typelude_core::{Eval, Evaluate};

use crate::lambda::{LApp, traits::LBind};

/// Left<L>: Represents the Error case or Left side of Either.
/// Church encoding: λl r. l L
pub struct LLeft<L>(PhantomData<L>);

impl<L> Eval for LLeft<L> {
    type Output = LLeft<L>;
}

/// Right<R>: Represents the Success case or Right side of Either.
/// Church encoding: λl r. r R
pub struct LRight<R>(PhantomData<R>);

impl<R> Eval for LRight<R> {
    type Output = LRight<R>;
}
// Left<L> >>= k  -->  Left<L> (Short-circuit error)
impl<L, K> LBind<K> for LLeft<L> {
    type Output = LLeft<L>;
}

// Right<R> >>= k  -->  k R (Apply continuation)
impl<R, K> LBind<K> for LRight<R>
where
    K: Eval,
    R: Eval,
    LApp<K, R>: Eval,
{
    type Output = Evaluate<LApp<K, R>>;
}
// Left<L> HandlL -> Left1<L, HandlL>
impl<L, HandlL> Eval for LApp<LLeft<L>, HandlL>
where
    L: Eval,
    HandlL: Eval,
{
    type Output = LLeft1<Evaluate<L>, Evaluate<HandlL>>;
}

pub struct LLeft1<L, HandlL>(PhantomData<(L, HandlL)>);
impl<L, HandlL> Eval for LLeft1<L, HandlL> {
    type Output = LLeft1<L, HandlL>;
}

// Left1<L, HandlL> HandlR -> HandlL L
impl<L, HandlL, HandlR> Eval for LApp<LLeft1<L, HandlL>, HandlR>
where
    L: Eval,
    HandlL: Eval,
    HandlR: Eval,
    LApp<HandlL, L>: Eval,
{
    type Output = Evaluate<LApp<HandlL, L>>;
}

// Right<R> HandlL -> Right1<R, HandlL>
impl<R, HandlL> Eval for LApp<LRight<R>, HandlL>
where
    R: Eval,
    HandlL: Eval,
{
    type Output = LRight1<Evaluate<R>, Evaluate<HandlL>>;
}

pub struct LRight1<R, HandlL>(PhantomData<(R, HandlL)>);
impl<R, HandlL> Eval for LRight1<R, HandlL> {
    type Output = LRight1<R, HandlL>;
}

// Right1<R, HandlL> HandlR -> HandlR R
impl<R, HandlL, HandlR> Eval for LApp<LRight1<R, HandlL>, HandlR>
where
    R: Eval,
    HandlL: Eval,
    HandlR: Eval,
    LApp<HandlR, R>: Eval,
{
    type Output = Evaluate<LApp<HandlR, R>>;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::lambda::church::{LSucc, LZero};

    #[derive(Clone)]
    struct RightAddOne;
    impl Eval for RightAddOne {
        type Output = RightAddOne;
    }

    impl<X> Eval for LApp<RightAddOne, X>
    where
        X: crate::lambda::traits::LNat + Eval,
    {
        type Output = LRight<LSucc<Evaluate<X>>>;
    }

    #[derive(Clone)]
    struct FailAtStep;
    impl Eval for FailAtStep {
        type Output = FailAtStep;
    }

    impl<X> Eval for LApp<FailAtStep, X>
    where
        X: crate::lambda::traits::LNat + Eval,
    {
        type Output = LLeft<LSucc<Evaluate<X>>>;
    }

    #[test]
    fn test_either_right() {
        type RightInput = LRight<LZero>;
        type RightResult = <RightInput as LBind<RightAddOne>>::Output;
        assert_type_eq_all!(RightResult, LRight<LSucc<LZero>>);
    }

    #[test]
    fn test_either_left_short_circuit() {
        type LeftInput = LLeft<LZero>;
        type LeftOutput = <LeftInput as LBind<RightAddOne>>::Output;
        assert_type_eq_all!(LeftOutput, LLeft<LZero>);
    }

    #[test]
    fn test_either_right_to_left() {
        type RightToLeftInput = LRight<LZero>;
        type RightToLeftResult = <RightToLeftInput as LBind<FailAtStep>>::Output;
        assert_type_eq_all!(RightToLeftResult, LLeft<LSucc<LZero>>);
    }
}
