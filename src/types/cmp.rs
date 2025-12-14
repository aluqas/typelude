//! **Comparison Operations**
//!
//! `typenum` を利用した大小比較演算の実装。

use typenum::{IsGreater, IsGreaterOrEqual, IsLess, IsLessOrEqual};

use crate::eval::{EApply2, Evaluable, Evaluator};
use crate::func::{FGe, FGt, FLe, FLt};
use crate::types::bool::{ToTyBoolOut, TyFalse, TyTrue};

// =============================================================================
// Comparison Operations
// =============================================================================

// --- FLt: A < B ---
impl<A, B> Evaluable for EApply2<FLt, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: IsLess<Evaluator<B>>,
    <Evaluator<A> as IsLess<Evaluator<B>>>::Output: crate::types::bool::ToTyBool,
{
    type Output = ToTyBoolOut<<Evaluator<A> as IsLess<Evaluator<B>>>::Output>;
}

// --- FLe: A <= B ---
impl<A, B> Evaluable for EApply2<FLe, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: IsLessOrEqual<Evaluator<B>>,
    <Evaluator<A> as IsLessOrEqual<Evaluator<B>>>::Output: crate::types::bool::ToTyBool,
{
    type Output = ToTyBoolOut<<Evaluator<A> as IsLessOrEqual<Evaluator<B>>>::Output>;
}

// --- FGt: A > B ---
impl<A, B> Evaluable for EApply2<FGt, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: IsGreater<Evaluator<B>>,
    <Evaluator<A> as IsGreater<Evaluator<B>>>::Output: crate::types::bool::ToTyBool,
{
    type Output = ToTyBoolOut<<Evaluator<A> as IsGreater<Evaluator<B>>>::Output>;
}

// --- FGe: A >= B ---
impl<A, B> Evaluable for EApply2<FGe, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: IsGreaterOrEqual<Evaluator<B>>,
    <Evaluator<A> as IsGreaterOrEqual<Evaluator<B>>>::Output: crate::types::bool::ToTyBool,
{
    type Output = ToTyBoolOut<<Evaluator<A> as IsGreaterOrEqual<Evaluator<B>>>::Output>;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::ELit;
    use crate::func::{EGe, EGt, ELe, ELt};
    use static_assertions::assert_type_eq_all;
    use typenum::{N1, P1, P2, U1, U2, U3};

    #[test]
    fn test_compare_unsigned() {
        // 1 < 2 -> True
        assert_type_eq_all!(Evaluator<ELt<ELit<U1>, ELit<U2>>>, TyTrue);
        // 2 < 1 -> False
        assert_type_eq_all!(Evaluator<ELt<ELit<U2>, ELit<U1>>>, TyFalse);

        // 1 <= 1 -> True
        assert_type_eq_all!(Evaluator<ELe<ELit<U1>, ELit<U1>>>, TyTrue);
        // 2 <= 1 -> False
        assert_type_eq_all!(Evaluator<ELe<ELit<U2>, ELit<U1>>>, TyFalse);

        // 2 > 1 -> True
        assert_type_eq_all!(Evaluator<EGt<ELit<U2>, ELit<U1>>>, TyTrue);
        // 1 > 2 -> False
        assert_type_eq_all!(Evaluator<EGt<ELit<U1>, ELit<U2>>>, TyFalse);

        // 2 >= 2 -> True
        assert_type_eq_all!(Evaluator<EGe<ELit<U2>, ELit<U2>>>, TyTrue);
        // 1 >= 2 -> False
        assert_type_eq_all!(Evaluator<EGe<ELit<U1>, ELit<U2>>>, TyFalse);
    }

    #[test]
    fn test_compare_signed() {
        // -1 < 1 -> True
        assert_type_eq_all!(Evaluator<ELt<ELit<N1>, ELit<P1>>>, TyTrue);
        // 2 > -1 -> True
        assert_type_eq_all!(Evaluator<EGt<ELit<P2>, ELit<N1>>>, TyTrue);
    }
}
