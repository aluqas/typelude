//! **Lambda Traits**
//!
//! Traits specific to pure functional programming context.
//!
//! - `Lambda`: Pure lambda term trait with automatic `Eval` implementation
//! - `Bind`: Monadic bind operation (`m >>= f`)

use crate::eval::Eval;

// =========================================================================
// Lambda Trait
// =========================================================================

/// **Pure Lambda Term Trait**
///
/// Types implementing this trait represent pure lambda calculus terms
/// or operations derived strictly from them.
///
/// Implementing `Lambda` automatically provides `Eval` via blanket impl.
pub trait Lambda {
    type Output;
}

/// Blanket impl: Lambda types automatically implement Eval
impl<T: Lambda> Eval for T {
    type Output = <T as Lambda>::Output;
}

// =========================================================================
// Deep Embedding: AST Node for Application
// =========================================================================

use std::marker::PhantomData;

use crate::kernel::traits::Apply;

/// **Application AST Node**
///
/// `App<F, A>` represents the application of function `F` to argument `A`.
/// It is a "Lazy" representation (Deep Embedding) that does not compute
/// the result until `Eval` is invoked.
pub struct App<F, A>(PhantomData<(F, A)>);

/// Evaluation Logic for App:
/// 1. Evaluate F -> F_val
/// 2. Evaluate A -> A_val
/// 3. Apply A_val to F_val -> Result_Thunk
/// 4. Evaluate Result_Thunk -> Final_Result
impl<F, A> Lambda for App<F, A>
where
    F: Eval,
    A: Eval,
    F::Output: Apply<A::Output>,
    <F::Output as Apply<A::Output>>::Output: Eval,
{
    type Output = <<F::Output as Apply<A::Output>>::Output as Eval>::Output;
}

// =========================================================================
// Bind Trait (Monad)
// =========================================================================

/// Bind Trait: `m >>= f`
///
/// `Self` is the Monad (e.g., `Option<T>`).
/// `F` is the Binder function (`T -> Option<U>`).
/// `Output` is the result Monad (`Option<U>`).
pub trait LBind<F> {
    type Output;
}

// Note: `Pure` is usually specific to the Monad type constructor itself
// (e.g. `Id<T>`, `State<S, A>`), so we might not need a universal trait for it
// unless we want generic code over Monads.
// =========================================================================
// Kind System (Marker Traits)
// =========================================================================

/// Base trait for all Lambda Terms.
pub trait LTerm {}

/// Marker for Church Booleans.
pub trait LBool: LTerm {}

/// Marker for Church Numerals.
pub trait LNat: LTerm {}

/// Marker for Church Lists.
pub trait LList: LTerm {}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::{
        eval::Evaluate,
        lambda::church::{LSucc, LZero},
    };

    #[test]
    fn test_app_deep_embedding() {
        // Test 1: Simple Identity
        // App<I, A> -> A
        struct I;
        impl Lambda for I {
            type Output = I;
        }
        impl<X> Apply<X> for I {
            type Output = X;
        }

        struct A;
        impl Lambda for A {
            type Output = A;
        }

        type Res1 = Evaluate<App<I, A>>;
        assert_type_eq_all!(Res1, A);

        /*
        // Test 2: Add using Church Numerals
        // App<App<Add, One>, One> -> Two
        use crate::lambda::church::numeral::{LAdd, LSuccGen};

        // Use concrete Church types which implement Eval
        type One = LSucc<LZero>;
        type Two = LSucc<One>;

        // Using App struct (Lazy AST)
        type Expr = App<App<LAdd, One>, One>;

        // Evaluate it
        type Result = Evaluate<Expr>;

        assert_type_eq_all!(Result, Two);
        */
    }
}
