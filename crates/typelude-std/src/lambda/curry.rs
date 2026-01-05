//! Curry and Uncurry transformations
//!
//! - `Curry<F>`: Convert `((A, B) -> C)` to `(A -> B -> C)`
//! - `Uncurry<F>`: Convert `(A -> B -> C)` to `((A, B) -> C)`

use std::marker::PhantomData;

use typelude_core::{Eval, Evaluate};

use super::{LApp, Lambda, church::LPair2};
/// Curry a function that takes a tuple into a curried function.
///
/// `Curry<F>` transforms `F: Apply<Pair<A, B>>` into a two-argument curried form.
pub struct LCurry<F>(PhantomData<F>);

impl<F> Lambda for LCurry<F> {
    type Output = LCurry<F>;
}
impl<F> Eval for LCurry<F> {
    type Output = Self;
}

/// Partially applied Curry: waiting for second argument.
pub struct LCurry1<F, A>(PhantomData<(F, A)>);

impl<F, A> Lambda for LCurry1<F, A> {
    type Output = LCurry1<F, A>;
}
impl<F, A> Eval for LCurry1<F, A> {
    type Output = Self;
}

// Curry<F> A -> Curry1<F, A>
impl<F, A> Lambda for LApp<LCurry<F>, A>
where
    F: Eval,
    A: Eval,
{
    type Output = LCurry1<F, Evaluate<A>>;
}

// Curry1<F, A> B -> F(Pair(A, B))
impl<F, A, B> Lambda for LApp<LCurry1<F, A>, B>
where
    F: Eval,
    A: Eval,
    B: Eval,
    LApp<F, LPair2<A, Evaluate<B>>>: Lambda,
{
    type Output = <LApp<F, LPair2<A, Evaluate<B>>> as Lambda>::Output;
}
/// Uncurry a curried function into one that takes a tuple (Church Pair).
///
/// `Uncurry<F>` transforms `F: A -> B -> C` into `F (Pair A B)`.
pub struct LUncurry<F>(PhantomData<F>);

impl<F> Lambda for LUncurry<F> {
    type Output = LUncurry<F>;
}
impl<F> Eval for LUncurry<F> {
    type Output = Self;
}

// Uncurry<F> (Pair A B) -> (F A) B
// We assume the argument is a LPair2<A, B>.
impl<F, A, B> Lambda for LApp<LUncurry<F>, LPair2<A, B>>
where
    F: Eval,
    A: Eval,
    B: Eval,
    // F A
    LApp<F, A>: Lambda,
    // (F A) B
    LApp<<LApp<F, A> as Lambda>::Output, B>: Lambda,
{
    type Output = <LApp<<LApp<F, A> as Lambda>::Output, B> as Lambda>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;

    type App<F, A> = Evaluate<LApp<F, A>>;

    // Test function: TupleAdd (A, B) -> Result<A, B>
    struct TupleAdd;
    struct TupleResult<A, B>(PhantomData<(A, B)>);
    impl Lambda for TupleAdd {
        type Output = TupleAdd;
    }
    impl Eval for TupleAdd {
        type Output = Self;
    }

    impl<A, B> Lambda for TupleResult<A, B> {
        type Output = TupleResult<A, B>;
    }
    impl<A, B> Eval for TupleResult<A, B> {
        type Output = Self;
    }

    impl<A, B> Lambda for LApp<TupleAdd, LPair2<A, B>>
    where
        A: Eval,
        B: Eval,
    {
        type Output = TupleResult<A, B>;
    }

    // Test curried function: CurriedAdd A -> CurriedAdd1<A>
    struct CurriedAdd;
    struct CurriedAdd1<A>(PhantomData<A>);
    struct CurriedResult<A, B>(PhantomData<(A, B)>);
    impl Lambda for CurriedAdd {
        type Output = CurriedAdd;
    }
    impl Eval for CurriedAdd {
        type Output = Self;
    }
    impl<A> Lambda for CurriedAdd1<A> {
        type Output = CurriedAdd1<A>;
    }
    impl<A> Eval for CurriedAdd1<A> {
        type Output = Self;
    }
    impl<A, B> Lambda for CurriedResult<A, B> {
        type Output = CurriedResult<A, B>;
    }
    impl<A, B> Eval for CurriedResult<A, B> {
        type Output = Self;
    }

    impl<A> Lambda for LApp<CurriedAdd, A>
    where
        A: Eval,
    {
        type Output = CurriedAdd1<Evaluate<A>>;
    }
    impl<A, B> Lambda for LApp<CurriedAdd1<A>, B>
    where
        A: Eval,
        B: Eval,
    {
        type Output = CurriedResult<A, Evaluate<B>>;
    }

    #[derive(Clone)]
    struct X;
    impl Lambda for X {
        type Output = X;
    }
    impl Eval for X {
        type Output = Self;
    }
    #[derive(Clone)]
    struct Y;
    impl Lambda for Y {
        type Output = Y;
    }
    impl Eval for Y {
        type Output = Self;
    }

    #[test]
    fn test_curry() {
        // Curry<TupleAdd> X Y == TupleResult<X, Y>
        type Curried = LCurry<TupleAdd>;
        type Step1 = App<Curried, X>;
        type Result = App<Step1, Y>;
        assert_type_eq_all!(Result, TupleResult<X, Y>);
    }

    #[test]
    fn test_uncurry() {
        // Uncurry<CurriedAdd> (X, Y) == CurriedResult<X, Y>
        type Uncurried = LUncurry<CurriedAdd>;
        type Result = App<Uncurried, LPair2<X, Y>>;
        assert_type_eq_all!(Result, CurriedResult<X, Y>);
    }

    #[test]
    fn test_curry_uncurry_roundtrip() {
        // Uncurry<Curry<TupleAdd>> (X, Y) == TupleResult<X, Y>
        type Roundtrip = LUncurry<LCurry<TupleAdd>>;
        type Result = App<Roundtrip, LPair2<X, Y>>;
        assert_type_eq_all!(Result, TupleResult<X, Y>);
    }
}
