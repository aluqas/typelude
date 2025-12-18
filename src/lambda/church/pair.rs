//! Church Pairs
//!
//! Church encoding of pairs:
//! - `Pair<X, Y>`: λf. f x y

use std::marker::PhantomData;

use super::bool::{False, True};
use crate::kernel::traits::Apply;
use crate::lambda::Lambda;

// =========================================================================
// Church Pairs
// =========================================================================

/// Pair: λx y. λf. f x y
pub struct Pair<X, Y>(PhantomData<(X, Y)>);

impl<X, Y> Lambda for Pair<X, Y> {
    type Output = Pair<X, Y>;
}

/// Pure Fst/Snd Aliases
pub type PureFst<P> = <P as Apply<True>>::Output;
pub type PureSnd<P> = <P as Apply<False>>::Output;

/// Fst Struct
pub struct Fst<P>(PhantomData<P>);

impl<P> Lambda for Fst<P>
where
    P: Apply<True>,
{
    type Output = PureFst<P>;
}

/// Snd Struct
pub struct Snd<P>(PhantomData<P>);

impl<P> Lambda for Snd<P>
where
    P: Apply<False>,
{
    type Output = PureSnd<P>;
}

// Helpers for internal use
pub(super) type SndEval<P> = <P as Apply<False>>::Output;

// Pair<X, Y> f -> f X Y
impl<X, Y, F> Apply<F> for Pair<X, Y>
where
    F: Apply<X>,
    <F as Apply<X>>::Output: Apply<Y>,
{
    type Output = <<F as Apply<X>>::Output as Apply<Y>>::Output;
}
