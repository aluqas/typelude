//! **Evaluation Infrastructure**
//!
//! Provides core infrastructure for type-level computation.

mod expr;

pub use expr::*;

// -----------------------------------------------------------------------------
// Sealed Trait (for internal use)
// -----------------------------------------------------------------------------

/// Sealed trait pattern - Prevents external crate implementations
///
/// This trait is for internal implementation/use only, do not use directly.
#[doc(hidden)]
pub trait Sealed {}

// -----------------------------------------------------------------------------
// Core Evaluable Trait
// -----------------------------------------------------------------------------

/// Trait to evaluate type-level expressions
///
/// Types implementing `Evaluable` can get evaluation results via `Evaluator<T>`.
pub trait Eval {
    type Output;
}

/// Type alias to obtain evaluation results of expressions
///
/// # Example
/// ```ignore
/// type Result = Evaluator<EIf<ELit<TyTrue>, ELit<i32>, ELit<f64>>>;
/// // Result = i32
/// ```
pub type Evaluate<T> = <T as Eval>::Output;

#[cfg(test)]
mod tests {
    use static_assertions::{assert_type_eq_all, assert_type_ne_all};

    use crate::{
        eval::{EIf, ELit, EWhile, Evaluate},
        kernel::{
            array::{Cons, TyArray, TyNil},
            bool::{TyFalse, TyTrue},
            traits::Apply,
        },
    };

    // Define simple equality check for testing
    #[allow(dead_code)]
    pub struct OpEq;
    impl<L, R> Apply<(L, R)> for OpEq {
        type Output = TyFalse;
    }
    // Specialize for equal types? Rust specialization is unstable.
    // Hack for test: Use a concrete impl for specific types
    #[allow(dead_code)]
    struct TestOpEq;
    impl Apply<(TyTrue, TyTrue)> for TestOpEq {
        type Output = TyTrue;
    }
    impl Apply<(TyTrue, TyFalse)> for TestOpEq {
        type Output = TyFalse;
    }

    // Instead of full Eq engine, let's just test EIf directly with literals.

    #[test]
    fn test_eval_if() {
        assert_type_eq_all!(Evaluate<EIf<ELit<TyTrue>, ELit<i32>, ELit<f64>>>, i32);
        assert_type_eq_all!(Evaluate<EIf<ELit<TyFalse>, ELit<i32>, ELit<f64>>>, f64);
        assert_type_ne_all!(Evaluate<EIf<ELit<TyTrue>, ELit<i32>, ELit<()>>>, ());
    }

    #[test]
    fn test_eval_while() {
        // Condition: IsNotEmpty
        struct IsNotEmpty;
        impl Apply<ELit<TyNil>> for IsNotEmpty {
            type Output = TyFalse;
        }
        impl<H, T> Apply<ELit<TyArray<H, T>>> for IsNotEmpty
        where
            T: Cons,
        {
            type Output = TyTrue;
        }

        // Step: GetTail
        struct GetTail;
        impl<H, T> Apply<ELit<TyArray<H, T>>> for GetTail
        where
            T: Cons,
        {
            type Output = ELit<T>;
        }

        assert_type_eq_all!(
            Evaluate<
                EWhile<IsNotEmpty, GetTail, ELit<TyArray<i32, TyArray<f64, TyArray<(), TyNil>>>>>,
            >,
            TyNil
        );
    }

    // Removing dependencies on std::int for now to keep tests isolated to eval/kernel
}
