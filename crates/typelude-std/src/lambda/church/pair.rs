//! Church Pairs
//!
//! Church encoding of pairs:
//! - `Pair<X, Y>`: λf. f x y

use std::marker::PhantomData;

use typelude_core::{Eval, Evaluate};

use super::bool::{LFalse, LTrue};
use crate::{
    impl_eval_for_lambda, impl_eval_for_lambda_generic,
    lambda::LApp,
};
/// Pair: λx y. λf. f x y
pub struct LPair;

impl_eval_for_lambda!(LPair);

// Pair X -> Pair1<X>
pub struct LPair1<X>(PhantomData<X>);
impl_eval_for_lambda_generic!(LPair1, [X]);

impl<X> Eval for LApp<LPair, X>
where
    X: Eval,
{
    type Output = LPair1<Evaluate<X>>;
}

// Pair1<X> Y -> Pair2<X, Y> (The actual pair value)
pub struct LPair2<X, Y>(PhantomData<(X, Y)>);
impl_eval_for_lambda_generic!(LPair2, [X, Y]);

impl<X, Y> Eval for LApp<LPair1<X>, Y>
where
    Y: Eval,
{
    type Output = LPair2<X, Evaluate<Y>>;
}

// Pair2<X, Y> F -> F X Y
// IMPORTANT: With the new Eval rules, we must ensure Eval constraints are met.
// LApp<F, X> must be Eval.
impl<X, Y, F> Eval for LApp<LPair2<X, Y>, F>
where
    X: Eval,
    Y: Eval,
    F: Eval,
    // (F X)
    LApp<F, X>: Eval,
    // ((F X) Y)
    LApp<Evaluate<LApp<F, X>>, Y>: Eval,
{
    type Output = Evaluate<LApp<Evaluate<LApp<F, X>>, Y>>;
}
// Fst P -> P True
pub struct LFst;
impl_eval_for_lambda!(LFst);

impl<P> Eval for LApp<LFst, P>
where
    P: Eval,
    // P True
    LApp<P, LTrue>: Eval,
{
    type Output = Evaluate<LApp<P, LTrue>>;
}

// Snd P -> P False
pub struct LSnd;
impl_eval_for_lambda!(LSnd);

impl<P> Eval for LApp<LSnd, P>
where
    P: Eval,
    // P False
    LApp<P, LFalse>: Eval,
{
    type Output = Evaluate<LApp<P, LFalse>>;
}
