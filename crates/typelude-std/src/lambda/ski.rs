use std::marker::PhantomData;

use typelude_core::{Eval, Evaluate};

use super::{LApp, Lambda};
use crate::{impl_eval_for_lambda, impl_eval_for_lambda_generic};
/// The Identity Combinator `I`
/// Term: \x. x
/// Rule: I x -> x
pub struct I;
impl Lambda for I {
    type Output = I;
}
impl_eval_for_lambda!(I);

/// The Constant Combinator `K`
/// Term: \x y. x
/// Rule: K x y -> x
pub struct K;
impl Lambda for K {
    type Output = K;
}
impl_eval_for_lambda!(K);

/// The Substitution Combinator `S`
/// Term: \x y z. (x z) (y z)
/// Rule: S x y z -> (x z) (y z)
pub struct S;
impl Lambda for S {
    type Output = S;
}
impl_eval_for_lambda!(S);
/// K applied to one argument: `K x`
pub struct K1<X>(PhantomData<X>);
impl<X> Lambda for K1<X> {
    type Output = K1<X>;
}
impl_eval_for_lambda_generic!(K1, [X]);

/// S applied to one argument: `S x`
pub struct S1<X>(PhantomData<X>);
impl<X> Lambda for S1<X> {
    type Output = S1<X>;
}
impl_eval_for_lambda_generic!(S1, [X]);

/// S applied to two arguments: `S x y`
pub struct S2<X, Y>(PhantomData<(X, Y)>);
impl<X, Y> Lambda for S2<X, Y> {
    type Output = S2<X, Y>;
}
impl_eval_for_lambda_generic!(S2, [X, Y]);
// Note: We use strict Call-by-Value strategy for arguments.
// Arguments X, Y, Z are typically expected to be evaluated before being stored in state structs,
// but the Lambda impl for LApp<Combinator, Arg> enforces this by calling Evaluate<Arg>.

// --- I Combinator ---
// I x -> x
impl<X> Lambda for LApp<I, X>
where
    X: Eval,
{
    type Output = Evaluate<X>;
}

// --- K Combinator ---
// K x -> K1<x> (Partial)
impl<X> Lambda for LApp<K, X>
where
    X: Eval,
{
    type Output = K1<Evaluate<X>>;
}

// K1<x> y -> x (Reduction)
impl<X, Y> Lambda for LApp<K1<X>, Y>
where
    X: Eval,
{
    type Output = Evaluate<X>;
}

// --- S Combinator ---
// S x -> S1<x> (Partial)
impl<X> Lambda for LApp<S, X>
where
    X: Eval,
{
    type Output = S1<Evaluate<X>>;
}

// S1<x> y -> S2<x, y> (Partial)
impl<X, Y> Lambda for LApp<S1<X>, Y>
where
    X: Eval,
    Y: Eval,
{
    type Output = S2<Evaluate<X>, Evaluate<Y>>;
}

// S2<x, y> z -> (x z) (y z) (Reduction)
impl<X, Y, Z> Lambda for LApp<S2<X, Y>, Z>
where
    X: Eval,
    Y: Eval,
    Z: Eval + Clone,
    LApp<Evaluate<X>, Evaluate<Z>>: Eval,
    LApp<Evaluate<Y>, Evaluate<Z>>: Eval,
    LApp<Evaluate<LApp<Evaluate<X>, Evaluate<Z>>>, Evaluate<LApp<Evaluate<Y>, Evaluate<Z>>>>: Eval,
{
    type Output = Evaluate<
        LApp<Evaluate<LApp<Evaluate<X>, Evaluate<Z>>>, Evaluate<LApp<Evaluate<Y>, Evaluate<Z>>>>,
    >;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;

    // Helper alias for Evaluate<LApp<F, A>>
    type App<F, A> = Evaluate<LApp<F, A>>;

    #[test]
    fn test_identity_combinator() {
        #[derive(Clone)]
        struct X;
        impl Lambda for X {
            type Output = X;
        }
        impl_eval_for_lambda!(X);

        assert_type_eq_all!(App<I, X>, X);
    }

    #[test]
    fn test_k_combinator() {
        #[derive(Clone)]
        struct X;
        impl Lambda for X {
            type Output = X;
        }
        impl_eval_for_lambda!(X);
        #[derive(Clone)]
        struct Y;
        impl Lambda for Y {
            type Output = Y;
        }
        impl_eval_for_lambda!(Y);

        // K X -> K1<X>
        // (K X) Y -> X
        assert_type_eq_all!(App<App<K, X>, Y>, X);
    }

    #[test]
    fn test_s_combinator_basic() {
        #[derive(Clone)]
        struct X;
        impl Lambda for X {
            type Output = X;
        }
        impl_eval_for_lambda!(X);

        // SKK X -> (K X) (K X) -> X
        type SKK = App<App<S, K>, K>;
        assert_type_eq_all!(App<SKK, X>, X);
    }

    #[test]
    fn test_church_booleans() {
        #[derive(Clone)]
        struct A;
        impl Lambda for A {
            type Output = A;
        }
        impl_eval_for_lambda!(A);

        #[derive(Clone)]
        struct B;
        impl Lambda for B {
            type Output = B;
        }
        impl_eval_for_lambda!(B);

        type True = K;
        type False = App<K, I>; // K I -> K1<I>

        // True A B -> A
        assert_type_eq_all!(App<App<True, A>, B>, A);

        // False A B -> (K I) A B -> I B -> B
        assert_type_eq_all!(App<App<False, A>, B>, B);
    }

    #[test]
    fn test_associativity_check() {
        struct F;
        impl Lambda for F {
            type Output = F;
        }
        impl_eval_for_lambda!(F);

        struct A;
        impl Lambda for A {
            type Output = A;
        }
        impl_eval_for_lambda!(A);

        struct B;
        impl Lambda for B {
            type Output = B;
        }
        impl_eval_for_lambda!(B);

        struct F1<X>(std::marker::PhantomData<X>);
        impl<X> Lambda for F1<X> {
            type Output = F1<X>;
        }
        impl_eval_for_lambda_generic!(F1, [X]);

        struct F2<X, Y>(std::marker::PhantomData<(X, Y)>);
        impl<X, Y> Lambda for F2<X, Y> {
            type Output = F2<X, Y>;
        }
        impl_eval_for_lambda_generic!(F2, [X, Y]);

        // Define behavior for F
        impl<X: Eval> Lambda for LApp<F, X> {
            type Output = F1<Evaluate<X>>;
        }
        impl<X: Eval, Y: Eval> Lambda for LApp<F1<X>, Y> {
            type Output = F2<X, Evaluate<Y>>;
        }

        assert_type_eq_all!(App<App<F, A>, B>, F2<A, B>);
    }
}
