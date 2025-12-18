//! Church Numerals and Arithmetic
//!
//! Church encoding of natural numbers and arithmetic operations:
//! - `Zero`: λf x. x
//! - `Succ<N>`: λn f x. f (n f x)

use std::marker::PhantomData;

// use super::{
//     pair::LPair2,
// };
use crate::{
    eval::{Eval, Evaluate},
    lambda::{
        LApp, Lambda,
        traits::{LNat, LTerm},
    },
};

// =========================================================================
// Church Numerals
// =========================================================================

/// Zero: λf x. x
pub struct LZero;
impl LTerm for LZero {}
impl LNat for LZero {}

impl Lambda for LZero {
    type Output = LZero;
}

/// Succ: λn f x. f (n f x)
pub struct LSucc<N>(PhantomData<N>);
impl<N> LTerm for LSucc<N> {}
impl<N> LNat for LSucc<N> {}

impl<N> Lambda for LSucc<N> {
    type Output = LSucc<N>;
}

// Partial Application States
pub struct LZero1<F>(PhantomData<F>);
pub struct LSucc1<N, F>(PhantomData<(N, F)>);

// --- Zero Logic ---
// Zero F -> Zero1<F>
impl<F> Lambda for LApp<LZero, F>
where
    F: Eval,
{
    type Output = LZero1<Evaluate<F>>;
}

// Zero1<F> X -> X
impl<F, X> Lambda for LApp<LZero1<F>, X>
where
    X: Eval,
{
    type Output = Evaluate<X>;
}

// --- Succ Logic ---
// Succ<N> F -> Succ1<N, F>
impl<N, F> Lambda for LApp<LSucc<N>, F>
where
    F: Eval,
{
    type Output = LSucc1<N, Evaluate<F>>;
}

// Succ1<N, F> X -> F (N F X)
impl<N, F, X> Lambda for LApp<LSucc1<N, F>, X>
where
    N: Eval,
    F: Eval + Clone,
    X: Eval,
    // (N F X)
    LApp<N, F>: Lambda, // Ensure N applies to F
    LApp<<LApp<N, F> as Lambda>::Output, X>: Lambda, // Ensure (N F) applies to X
    // F (N F X)
    LApp<F, <LApp<<LApp<N, F> as Lambda>::Output, X> as Lambda>::Output>: Lambda,
{
    type Output = <LApp<F, <LApp<<LApp<N, F> as Lambda>::Output, X> as Lambda>::Output> as Lambda>::Output;
}

/// SuccGen: Generates Succ<N> from N
pub struct LSuccGen;

// SuccGen N -> Succ<N>
impl<N> Lambda for LApp<LSuccGen, N>
where
    N: Eval,
{
    type Output = LSucc<Evaluate<N>>;
}

// =========================================================================
// Arithmetic Operations
// =========================================================================

// --- Add: \m n f x. m f (n f x) ---

pub struct LAdd;
pub struct LAdd1<M>(PhantomData<M>);
pub struct LAdd2<M, N>(PhantomData<(M, N)>);
pub struct LAdd3<M, N, F>(PhantomData<(M, N, F)>);

impl Lambda for LAdd { type Output = LAdd; }
impl<M> Lambda for LAdd1<M> { type Output = LAdd1<M>; }
impl<M, N> Lambda for LAdd2<M, N> { type Output = LAdd2<M, N>; }
impl<M, N, F> Lambda for LAdd3<M, N, F> { type Output = LAdd3<M, N, F>; }

// Implementations commented out to avoid recursion overflow
/*
// Add M -> Add1<M>
impl<M> Lambda for LApp<LAdd, M>
where
    M: Eval,
{
    type Output = LAdd1<Evaluate<M>>;
}

// Add1<M> N -> Add2<M, N>
impl<M, N> Lambda for LApp<LAdd1<M>, N>
where
    N: Eval,
{
    type Output = LAdd2<M, Evaluate<N>>;
}

// Add2<M, N> F -> Add3<M, N, F>
impl<M, N, F> Lambda for LApp<LAdd2<M, N>, F>
where
    F: Eval,
{
    type Output = LAdd3<M, N, Evaluate<F>>;
}

// Add3<M, N, F> X -> M F (N F X)
impl<M, N, F, X> Lambda for LApp<LAdd3<M, N, F>, X>
where
    M: Eval,
    N: Eval,
    F: Eval + Clone,
    X: Eval,
    // n f x
    LApp<N, F>: Lambda,
    LApp<<LApp<N, F> as Lambda>::Output, X>: Lambda,
    // m f (n f x)
    LApp<M, F>: Lambda,
    LApp<<LApp<M, F> as Lambda>::Output, <LApp<<LApp<N, F> as Lambda>::Output, X> as Lambda>::Output>: Lambda,
{
    type Output = <LApp<<LApp<M, F> as Lambda>::Output, <LApp<<LApp<N, F> as Lambda>::Output, X> as Lambda>::Output> as Lambda>::Output;
}
*/

// --- Mul: \m n f x. m (n f) x ---

pub struct LMul;
pub struct LMul1<M>(PhantomData<M>);
pub struct LMul2<M, N>(PhantomData<(M, N)>);
pub struct LMul3<M, NF>(PhantomData<(M, NF)>);

impl Lambda for LMul { type Output = LMul; }
impl<M> Lambda for LMul1<M> { type Output = LMul1<M>; }
impl<M, N> Lambda for LMul2<M, N> { type Output = LMul2<M, N>; }
impl<M, NF> Lambda for LMul3<M, NF> { type Output = LMul3<M, NF>; }

/*
// Mul M -> Mul1<M>
impl<M> Lambda for LApp<LMul, M>
where
    M: Eval,
{
    type Output = LMul1<Evaluate<M>>;
}

// Mul1<M> N -> Mul2<M, N>
impl<M, N> Lambda for LApp<LMul1<M>, N>
where
    N: Eval,
{
    type Output = LMul2<M, Evaluate<N>>;
}

// Mul2<M, N> F -> Mul3<M, NF> where NF = N F
impl<M, N, F> Lambda for LApp<LMul2<M, N>, F>
where
    M: Eval,
    N: Eval,
    F: Eval,
    LApp<N, F>: Lambda, // Evaluate (N F) immediately? No, it's a function composition usually
{
    // Note: Church Mul is \m n f x. m (n f) x.
    // (n f) is the composition of n and f.
    // If N is a Church Numeral, (N F) is a function that applies F, N times.
    // We store this composed function (N F) as the new "F" for M.
    type Output = LMul3<M, <LApp<N, Evaluate<F>> as Lambda>::Output>;
}

// Mul3<M, NF> X -> M NF X
impl<M, NF, X> Lambda for LApp<LMul3<M, NF>, X>
where
    M: Eval,
    NF: Eval,
    X: Eval,
    LApp<M, NF>: Lambda,
    LApp<<LApp<M, NF> as Lambda>::Output, X>: Lambda,
{
    type Output = <LApp<<LApp<M, NF> as Lambda>::Output, X> as Lambda>::Output;
}
*/

// --- Exp ---
pub struct LExp;
pub struct LExp1<M>(PhantomData<M>);
impl Lambda for LExp { type Output = LExp; }
impl<M> Lambda for LExp1<M> { type Output = LExp1<M>; }

/*
// Exp M -> Exp1<M>
impl<M> Lambda for LApp<LExp, M> where M: Eval {
    type Output = LExp1<Evaluate<M>>;
}

// Exp1<M> N -> N M
impl<M, N> Lambda for LApp<LExp1<M>, N>
where
    M: Eval,
    N: Eval,
    LApp<N, M>: Lambda,
{
    type Output = <LApp<N, M> as Lambda>::Output;
}
*/

// =========================================================================
// Predecessor and Subtraction
// =========================================================================

// --- Pred ---
pub struct LPred;
pub struct LPredStep;
impl Lambda for LPred { type Output = LPred; }
impl Lambda for LPredStep { type Output = LPredStep; }

/*
impl<N> Lambda for LApp<LPred, N>
where
    N: Eval,
    LApp<LPredStep, N>: Lambda, // Wait, Pred is usually \n. ...
    // Standard Pred: \n. fst (n (\p. pair (snd p) (succ (snd p))) (pair zero zero))
    // We need Step function and Initial Pair.
    // Let's rely on helper structs.
    // Step: LPredStep
    // Init: LPair<LZero, LZero>
    LApp<N, LPredStep>: Lambda,
    LApp<<LApp<N, LPredStep> as Lambda>::Output, super::pair::LPair2<LZero, LZero>>: Lambda,
    // Result is a Pair. Get Fst.
    // We need LFst to be applicable to the result.
    LApp<super::pair::LFst, <LApp<<LApp<N, LPredStep> as Lambda>::Output, super::pair::LPair2<LZero, LZero>> as Lambda>::Output>: Lambda,
{
    type Output = <LApp<super::pair::LFst, <LApp<<LApp<N, LPredStep> as Lambda>::Output, super::pair::LPair2<LZero, LZero>> as Lambda>::Output> as Lambda>::Output;
}

// PredStep P -> Pair (Snd P) (Succ (Snd P))
impl<P> Lambda for LApp<LPredStep, P>
where
    P: Eval + Clone,
    // Get Snd P
    LApp<super::pair::LSnd, P>: Lambda,
    // Get Succ (Snd P) -> Apply LSuccGen to (Snd P)
    LApp<LSuccGen, <LApp<super::pair::LSnd, P> as Lambda>::Output>: Lambda,
{
    type Output = super::pair::LPair2<
        <LApp<super::pair::LSnd, P> as Lambda>::Output,
        <LApp<LSuccGen, <LApp<super::pair::LSnd, P> as Lambda>::Output> as Lambda>::Output
    >;
}
*/

// --- Sub ---
pub struct LSub;
pub struct LSub1<M>(PhantomData<M>);
impl Lambda for LSub { type Output = LSub; }
impl<M> Lambda for LSub1<M> { type Output = LSub1<M>; }

/*
impl<M> Lambda for LApp<LSub, M> where M: Eval { type Output = LSub1<Evaluate<M>>; }

impl<M, N> Lambda for LApp<LSub1<M>, N>
where
    M: Eval,
    N: Eval,
    LApp<N, LPred>: Lambda, // Apply N to Pred (repeat Pred N times)
    LApp<<LApp<N, LPred> as Lambda>::Output, M>: Lambda, // Apply Result to M
{
    type Output = <LApp<<LApp<N, LPred> as Lambda>::Output, M> as Lambda>::Output;
}
*/

#[cfg(test)]
mod tests {
    // use static_assertions::assert_type_eq_all;

    // use super::*;

    // Helper alias
    // type App<F, A> = Evaluate<LApp<F, A>>;

    // Numbers
    // type One = LSucc<LZero>;
    // type Two = LSucc<One>;

    /*
    #[test]
    fn test_church_numerals_basic() {
        #[derive(Clone)]
        struct F;
        impl Lambda for F { type Output = F; }

        struct F1<T>(std::marker::PhantomData<T>);
        impl<T> Lambda for F1<T> { type Output = F1<T>; }

        #[derive(Clone)]
        struct X;
        impl Lambda for X { type Output = X; }

        // Define F X -> F1<X>
        impl<X: Eval> Lambda for LApp<F, X> {
            type Output = F1<Evaluate<X>>;
        }
        impl<X: Eval, Y: Eval> Lambda for LApp<F1<X>, Y> {
             type Output = F1<Evaluate<Y>>; // Dummy behavior
        }

        // Zero F X -> X
        type ResZero = App<App<LZero, F>, X>;
        assert_type_eq_all!(ResZero, X);

        // One F X -> F X -> F1<X>
        type ResOne = App<App<One, F>, X>;
        assert_type_eq_all!(ResOne, F1<X>);
    }
    */
}
