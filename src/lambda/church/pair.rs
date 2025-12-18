//! Church Pairs
//!
//! Church encoding of pairs:
//! - `Pair<X, Y>`: λf. f x y

use std::marker::PhantomData;

use super::bool::{LFalse, LTrue};
use crate::{kernel::traits::Apply, lambda::Lambda};

// =========================================================================
// Church Pairs
// =========================================================================

/// Pair: λx y. λf. f x y
pub struct LPair<X, Y>(PhantomData<(X, Y)>);

impl<X, Y> Lambda for LPair<X, Y> {
    type Output = LPair<X, Y>;
}

/// Pure Fst/Snd Aliases
pub type LPureFst<P> = <P as Apply<LTrue>>::Output;
pub type LPureSnd<P> = <P as Apply<LFalse>>::Output;

/// Fst Struct
pub struct LFst<P>(PhantomData<P>);

impl<P> Lambda for LFst<P>
where
    P: Apply<LTrue>,
{
    type Output = LPureFst<P>;
}

/// Snd Struct
pub struct LSnd<P>(PhantomData<P>);

impl<P> Lambda for LSnd<P>
where
    P: Apply<LFalse>,
{
    type Output = LPureSnd<P>;
}

// Helpers for internal use
pub(super) type LSndEval<P> = <P as Apply<LFalse>>::Output;

// Pair<X, Y> f -> f X Y
impl<X, Y, F> Apply<F> for LPair<X, Y>
where
    F: Apply<X>,
    <F as Apply<X>>::Output: Apply<Y>,
{
    type Output = <<F as Apply<X>>::Output as Apply<Y>>::Output;
}
