//! Church Pairs
//!
//! Church encoding of pairs:
//! - `Pair<X, Y>`: λf. f x y

use std::marker::PhantomData;

use super::bool::{LFalse, LTrue};
use crate::{
    eval::{Eval, Evaluate},
    lambda::{LApp, Lambda},
};

// =========================================================================
// Church Pairs
// =========================================================================

/// Pair: λx y. λf. f x y
pub struct LPair;

impl Lambda for LPair {
    type Output = LPair;
}

// Pair X -> Pair1<X>
pub struct LPair1<X>(PhantomData<X>);
impl<X> Lambda for LPair1<X> {
    type Output = LPair1<X>;
}

impl<X> Lambda for LApp<LPair, X>
where
    X: Eval,
{
    type Output = LPair1<Evaluate<X>>;
}

// Pair1<X> Y -> Pair2<X, Y> (The actual pair value)
pub struct LPair2<X, Y>(PhantomData<(X, Y)>);
impl<X, Y> Lambda for LPair2<X, Y> {
    type Output = LPair2<X, Y>;
}

impl<X, Y> Lambda for LApp<LPair1<X>, Y>
where
    Y: Eval,
{
    type Output = LPair2<X, Evaluate<Y>>;
}

// Pair2<X, Y> F -> F X Y
impl<X, Y, F> Lambda for LApp<LPair2<X, Y>, F>
where
    X: Eval,
    Y: Eval,
    F: Eval,
    LApp<F, X>: Lambda,
    LApp<<LApp<F, X> as Lambda>::Output, Y>: Lambda,
{
    type Output = <LApp<<LApp<F, X> as Lambda>::Output, Y> as Lambda>::Output;
}

// =========================================================================
// Projections (Fst, Snd)
// =========================================================================

// Fst P -> P True
pub struct LFst;
impl Lambda for LFst {
    type Output = LFst;
}

impl<P> Lambda for LApp<LFst, P>
where
    P: Eval,
    LApp<P, LTrue>: Lambda,
{
    type Output = <LApp<P, LTrue> as Lambda>::Output;
}

// Snd P -> P False
pub struct LSnd;
impl Lambda for LSnd {
    type Output = LSnd;
}

impl<P> Lambda for LApp<LSnd, P>
where
    P: Eval,
    LApp<P, LFalse>: Lambda,
{
    type Output = <LApp<P, LFalse> as Lambda>::Output;
}

// Helpers for internal use (Evaluated via Lambda)
// Note: LSndEval was used in numeral.rs, we can redefine it or just use LApp chain in numeral.rs
// Since we are refactoring numeral.rs to use LApp, we don't strictly need this type alias if we write it out.
// But for compatibility with partially refactored code, we'll see.
// pub(super) type LSndEval<P> = Evaluate<LApp<LSnd, P>>;
