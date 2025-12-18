use std::marker::PhantomData;

use super::Apply;

// =========================================================================
// The Fundamental Combinators
// =========================================================================

/// The Identity Combinator `I`
/// Term: \x. x
/// Rule: I x -> x
pub struct I;

/// The Constant Combinator `K`
/// Term: \x y. x
/// Rule: K x y -> x
pub struct K;

/// The Substitution Combinator `S`
/// Term: \x y z. (x z) (y z)
/// Rule: S x y z -> (x z) (y z)
pub struct S;

// =========================================================================
// Partial Application States (The "Pending" Computation)
// =========================================================================

/// K applied to one argument: `K x`
pub struct K1<X>(PhantomData<X>);

/// S applied to one argument: `S x`
pub struct S1<X>(PhantomData<X>);

/// S applied to two arguments: `S x y`
pub struct S2<X, Y>(PhantomData<(X, Y)>);

// =========================================================================
// Reduction Rules (The "Logic")
// =========================================================================

// --- I Combinator ---
// I x -> x
impl<X> Apply<X> for I {
    type Output = X;
}

// --- K Combinator ---
// K x -> K1<x> (Partial)
impl<X> Apply<X> for K {
    type Output = K1<X>;
}

// K1<x> y -> x (Reduction)
impl<X, Y> Apply<Y> for K1<X> {
    type Output = X;
}

// --- S Combinator ---
// S x -> S1<x> (Partial)
impl<X> Apply<X> for S {
    type Output = S1<X>;
}

// S1<x> y -> S2<x, y> (Partial)
impl<X, Y> Apply<Y> for S1<X> {
    type Output = S2<X, Y>;
}

// S2<x, y> z -> (x z) (y z) (Reduction)
// This is the most complex rule. It requires Eval/Apply steps.
// Since `Apply` is our primitive, we define the Output as the application Result.
//
// Logic:
// 1. Let `XZ` = `x` applied to `z`
// 2. Let `YZ` = `y` applied to `z`
// 3. Result = `XZ` applied to `YZ`
impl<X, Y, Z> Apply<Z> for S2<X, Y>
where
    X: Apply<Z>,                 // x z is valid
    Y: Apply<Z>,                 // y z is valid
    X::Output: Apply<Y::Output>, // (x z) (y z) is valid
{
    type Output = <X::Output as Apply<Y::Output>>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;

    type App<F, A> = <F as Apply<A>>::Output;

    #[test]
    fn test_identity_combinator() {
        struct X;
        assert_type_eq_all!(App<I, X>, X);
    }

    #[test]
    fn test_k_combinator() {
        struct X;
        struct Y;
        assert_type_eq_all!(App<App<K, X>, Y>, X);
    }

    #[test]
    fn test_s_combinator_basic() {
        struct X;
        type SKK = App<App<S, K>, K>;
        assert_type_eq_all!(App<SKK, X>, X);
    }

    #[test]
    fn test_church_booleans() {
        struct A;
        struct B;
        type True = K;
        type False = App<K, I>;
        assert_type_eq_all!(App<App<True, A>, B>, A);
        assert_type_eq_all!(App<App<False, A>, B>, B);
    }

    #[test]
    fn test_associativity_check() {
        struct F;
        struct A;
        struct B;
        struct F1<X>(std::marker::PhantomData<X>);
        struct F2<X, Y>(std::marker::PhantomData<(X, Y)>);

        impl<X> Apply<X> for F {
            type Output = F1<X>;
        }
        impl<X, Y> Apply<Y> for F1<X> {
            type Output = F2<X, Y>;
        }

        assert_type_eq_all!(App<App<F, A>, B>, F2<A, B>);
    }
}
