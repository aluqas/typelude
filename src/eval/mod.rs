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
pub trait Evaluable {
    type Output;
}

/// Type alias to obtain evaluation results of expressions
///
/// # Example
/// ```ignore
/// type Result = Evaluator<EIf<ELit<TyTrue>, ELit<i32>, ELit<f64>>>;
/// // Result = i32
/// ```
pub type Evaluator<T> = <T as Evaluable>::Output;

#[cfg(test)]
mod tests {
    use static_assertions::{assert_type_eq_all, assert_type_ne_all};

    use crate::{
        eval::{EIf, ELit, EWhile, Evaluator},
        std::{
            array::{Cons, TyArray, TyNil},
            bool::{ToTyBoolOut, TyFalse, TyTrue},
            cmp::{EEq, ENotEq},
            traits::EFunction,
        },
    };

    #[test]
    fn test_eeq() {
        // 同じ型 → TyTrue
        assert_type_eq_all!(Evaluator<EEq<ELit<i32>, ELit<i32>>>, TyTrue);
        assert_type_eq_all!(Evaluator<EEq<ELit<TyTrue>, ELit<TyTrue>>>, TyTrue);
        assert_type_eq_all!(Evaluator<EEq<ELit<String>, ELit<String>>>, TyTrue);

        // 異なる型 → TyFalse
        assert_type_eq_all!(Evaluator<EEq<ELit<i32>, ELit<f64>>>, TyFalse);
        assert_type_eq_all!(Evaluator<EEq<ELit<TyTrue>, ELit<TyFalse>>>, TyFalse);
        assert_type_eq_all!(Evaluator<EEq<ELit<i32>, ELit<String>>>, TyFalse);

        // ENotEq: 同じ型 → TyFalse
        assert_type_eq_all!(Evaluator<ENotEq<ELit<i32>, ELit<i32>>>, TyFalse);
        assert_type_eq_all!(Evaluator<ENotEq<ELit<TyTrue>, ELit<TyTrue>>>, TyFalse);

        // ENotEq: 異なる型 → TyTrue
        assert_type_eq_all!(Evaluator<ENotEq<ELit<i32>, ELit<f64>>>, TyTrue);
        assert_type_eq_all!(Evaluator<ENotEq<ELit<TyTrue>, ELit<TyFalse>>>, TyTrue);
    }

    #[test]
    fn test_eval_if() {
        assert_type_eq_all!(Evaluator<EIf<ELit<TyTrue>, ELit<i32>, ELit<f64>>>, i32);
        assert_type_eq_all!(Evaluator<EIf<ELit<TyFalse>, ELit<i32>, ELit<f64>>>, f64);
        assert_type_ne_all!(Evaluator<EIf<ELit<TyTrue>, ELit<i32>, ELit<()>>>, ());
    }

    #[test]
    fn test_eval_while() {
        // Condition: IsNotEmpty
        struct IsNotEmpty;
        impl EFunction<TyNil> for IsNotEmpty {
            type Output = TyFalse;
        }
        impl<H, T> EFunction<TyArray<H, T>> for IsNotEmpty
        where
            T: Cons,
        {
            type Output = TyTrue;
        }

        // Step: GetTail
        struct GetTail;
        impl<H, T> EFunction<TyArray<H, T>> for GetTail
        where
            T: Cons,
        {
            type Output = ELit<T>;
        }

        assert_type_eq_all!(
            Evaluator<EWhile<IsNotEmpty, GetTail, TyArray<i32, TyArray<f64, TyArray<(), TyNil>>>>>,
            TyNil
        );
    }

    #[test]
    fn while_loop_plus_one() {
        use typenum::{Add1, IsLess, U1, U10, Unsigned};

        use crate::std::conv::TyFrom;

        struct IsLessThan10;
        impl<T> EFunction<T> for IsLessThan10
        where
            T: IsLess<U10>,
            bool: TyFrom<<T as IsLess<U10>>::Output>,
        {
            type Output = ToTyBoolOut<<T as IsLess<U10>>::Output>;
        }

        struct PlusOne;
        impl<T> EFunction<T> for PlusOne
        where
            T: std::ops::Add<typenum::B1>,
            Add1<T>: Unsigned,
        {
            type Output = ELit<Add1<T>>;
        }

        type Result = Evaluator<EWhile<IsLessThan10, PlusOne, U1>>;
        assert_type_eq_all!(Result, U10);
    }
}
