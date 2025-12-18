//! Curry and Uncurry transformations
//!
//! - `Curry<F>`: Convert `((A, B) -> C)` to `(A -> B -> C)`
//! - `Uncurry<F>`: Convert `(A -> B -> C)` to `((A, B) -> C)`

use std::marker::PhantomData;

use super::{Apply, Lambda};

// =========================================================================
// Curry: ((A, B) -> C) -> A -> B -> C
// =========================================================================

/// Curry a function that takes a tuple into a curried function.
///
/// `Curry<F>` transforms `F: Apply<(A, B)>` into a two-argument curried form.
pub struct LCurry<F>(PhantomData<F>);

impl<F> Lambda for LCurry<F> {
    type Output = LCurry<F>;
}

/// Partially applied Curry: waiting for second argument.
pub struct LCurry1<F, A>(PhantomData<(F, A)>);

impl<F, A> Lambda for LCurry1<F, A> {
    type Output = LCurry1<F, A>;
}

// Curry<F> A -> Curry1<F, A>
impl<F, A> Apply<A> for LCurry<F> {
    type Output = LCurry1<F, A>;
}

// Curry1<F, A> B -> F(A, B)
impl<F, A, B> Apply<B> for LCurry1<F, A>
where
    F: Apply<(A, B)>,
{
    type Output = <F as Apply<(A, B)>>::Output;
}

// =========================================================================
// Uncurry: (A -> B -> C) -> (A, B) -> C
// =========================================================================

/// Uncurry a curried function into one that takes a tuple.
///
/// `Uncurry<F>` transforms `F: Apply<A, Output: Apply<B>>` into tuple form.
pub struct LUncurry<F>(PhantomData<F>);

impl<F> Lambda for LUncurry<F> {
    type Output = LUncurry<F>;
}

// Uncurry<F> (A, B) -> (F A) B
impl<F, A, B> Apply<(A, B)> for LUncurry<F>
where
    F: Apply<A>,
    <F as Apply<A>>::Output: Apply<B>,
{
    type Output = <<F as Apply<A>>::Output as Apply<B>>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;

    // Test function: TupleAdd (A, B) -> Result<A, B>
    struct TupleAdd;
    struct TupleResult<A, B>(PhantomData<(A, B)>);

    impl<A, B> Apply<(A, B)> for TupleAdd {
        type Output = TupleResult<A, B>;
    }

    // Test curried function: CurriedAdd A -> CurriedAdd1<A>
    struct CurriedAdd;
    struct CurriedAdd1<A>(PhantomData<A>);
    struct CurriedResult<A, B>(PhantomData<(A, B)>);

    impl<A> Apply<A> for CurriedAdd {
        type Output = CurriedAdd1<A>;
    }
    impl<A, B> Apply<B> for CurriedAdd1<A> {
        type Output = CurriedResult<A, B>;
    }

    struct X;
    struct Y;

    #[test]
    fn test_curry() {
        // Curry<TupleAdd> X Y == TupleResult<X, Y>
        type Curried = LCurry<TupleAdd>;
        type Step1 = <Curried as Apply<X>>::Output;
        type Result = <Step1 as Apply<Y>>::Output;
        assert_type_eq_all!(Result, TupleResult<X, Y>);
    }

    #[test]
    fn test_uncurry() {
        // Uncurry<CurriedAdd> (X, Y) == CurriedResult<X, Y>
        type Uncurried = LUncurry<CurriedAdd>;
        type Result = <Uncurried as Apply<(X, Y)>>::Output;
        assert_type_eq_all!(Result, CurriedResult<X, Y>);
    }

    #[test]
    fn test_curry_uncurry_roundtrip() {
        // Uncurry<Curry<TupleAdd>> (X, Y) == TupleResult<X, Y>
        type Roundtrip = LUncurry<LCurry<TupleAdd>>;
        type Result = <Roundtrip as Apply<(X, Y)>>::Output;
        assert_type_eq_all!(Result, TupleResult<X, Y>);
    }
}
