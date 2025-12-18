use std::marker::PhantomData;

use crate::{
    kernel::traits::Apply,
    lambda::{Lambda, monad::Bind},
};

/// Left<L>: Represents the Error case or Left side of Either.
/// Church encoding: \l r. l L
pub struct Left<L>(PhantomData<L>);
impl<L> Lambda for Left<L> {
    type Output = Left<L>;
}

/// Right<R>: Represents the Success case or Right side of Either.
/// Church encoding: \l r. r R
pub struct Right<R>(PhantomData<R>);
impl<R> Lambda for Right<R> {
    type Output = Right<R>;
}

// =========================================================================
// Bind Implementation for Monad
// =========================================================================

// Left<L> >>= k  -->  Left<L> (Short-circuit error)
impl<L, K> Bind<K> for Left<L> {
    type Output = Left<L>;
}

// Right<R> >>= k  -->  k R (Apply continuation)
impl<R, K> Bind<K> for Right<R>
where
    K: Apply<R>,
{
    type Output = <K as Apply<R>>::Output;
}

// =========================================================================
// Church Encoding Apply Implementation
// =========================================================================

// Left<L> \l -> Left1<L, l>
impl<L, HandlL> Apply<HandlL> for Left<L> {
    type Output = Left1<L, HandlL>;
}

pub struct Left1<L, HandlL>(PhantomData<(L, HandlL)>);

// Left1<L, l> \r -> l L
impl<L, HandlL, HandlR> Apply<HandlR> for Left1<L, HandlL>
where
    HandlL: Apply<L>,
{
    type Output = <HandlL as Apply<L>>::Output;
}

// Right<R> \l -> Right1<R, l>
impl<R, HandlL> Apply<HandlL> for Right<R> {
    type Output = Right1<R, HandlL>;
}

pub struct Right1<R, HandlL>(PhantomData<(R, HandlL)>);

// Right1<R, l> \r -> r R
impl<R, HandlL, HandlR> Apply<HandlR> for Right1<R, HandlL>
where
    HandlR: Apply<R>,
{
    type Output = <HandlR as Apply<R>>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::lambda::church::{Succ, Zero};

    struct RightAddOne;
    impl<X> Apply<X> for RightAddOne {
        type Output = Right<Succ<X>>;
    }

    struct FailAtStep;
    impl<X> Apply<X> for FailAtStep {
        type Output = Left<Succ<X>>;
    }

    #[test]
    fn test_either_right() {
        type RightInput = Right<Zero>;
        type RightResult = <RightInput as Bind<RightAddOne>>::Output;
        assert_type_eq_all!(RightResult, Right<Succ<Zero>>);
    }

    #[test]
    fn test_either_left_short_circuit() {
        type LeftInput = Left<Zero>;
        type LeftOutput = <LeftInput as Bind<RightAddOne>>::Output;
        assert_type_eq_all!(LeftOutput, Left<Zero>);
    }

    #[test]
    fn test_either_right_to_left() {
        type RightToLeftInput = Right<Zero>;
        type RightToLeftResult = <RightToLeftInput as Bind<FailAtStep>>::Output;
        assert_type_eq_all!(RightToLeftResult, Left<Succ<Zero>>);
    }
}
