//! **Integer Operations**
//!
//! `typenum` を利用した整数演算の実装。

use std::ops::{Add, Div, Mul, Rem, Sub};
use typenum::{Integer, Unsigned, Z0, NInt, PInt, UInt, UTerm, Bit};

use crate::eval::{EApply2, Evaluable, Evaluator};
use crate::func::{FAdd, FDiv, FMul, FRem, FSub};

// =============================================================================
// Evaluable Implementation for Typenum
// =============================================================================

impl Evaluable for Z0 {
    type Output = Z0;
}

impl<U: Unsigned, B: Bit> Evaluable for UInt<U, B> {
    type Output = UInt<U, B>;
}

impl Evaluable for UTerm {
    type Output = UTerm;
}

impl<U: Unsigned + typenum::NonZero> Evaluable for PInt<U> {
    type Output = PInt<U>;
}

impl<U: Unsigned + typenum::NonZero> Evaluable for NInt<U> {
    type Output = NInt<U>;
}

// =============================================================================
// Arithmetic Operations
// =============================================================================

// --- FAdd: A + B ---
impl<A, B> Evaluable for EApply2<FAdd, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: Add<Evaluator<B>>,
{
    type Output = <Evaluator<A> as Add<Evaluator<B>>>::Output;
}

// --- FSub: A - B ---
impl<A, B> Evaluable for EApply2<FSub, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: Sub<Evaluator<B>>,
{
    type Output = <Evaluator<A> as Sub<Evaluator<B>>>::Output;
}

// --- FMul: A * B ---
impl<A, B> Evaluable for EApply2<FMul, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: Mul<Evaluator<B>>,
{
    type Output = <Evaluator<A> as Mul<Evaluator<B>>>::Output;
}

// --- FDiv: A / B ---
impl<A, B> Evaluable for EApply2<FDiv, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: Div<Evaluator<B>>,
{
    type Output = <Evaluator<A> as Div<Evaluator<B>>>::Output;
}

// --- FRem: A % B ---
impl<A, B> Evaluable for EApply2<FRem, A, B>
where
    A: Evaluable,
    B: Evaluable,
    Evaluator<A>: Rem<Evaluator<B>>,
{
    type Output = <Evaluator<A> as Rem<Evaluator<B>>>::Output;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::ELit;
    use crate::func::{EAdd, EDiv, EMul, ERem, ESub};
    use static_assertions::assert_type_eq_all;
    use typenum::{N3, P2, P3, P4, P6, U2, U3, U4, U5, U6};

    #[test]
    fn test_arithmetic_unsigned() {
        // 2 + 3 = 5
        assert_type_eq_all!(Evaluator<EAdd<ELit<U2>, ELit<U3>>>, U5);
        // 5 - 2 = 3
        assert_type_eq_all!(Evaluator<ESub<ELit<U5>, ELit<U2>>>, U3);
        // 2 * 3 = 6
        assert_type_eq_all!(Evaluator<EMul<ELit<U2>, ELit<U3>>>, U6);
        // 6 / 2 = 3
        assert_type_eq_all!(Evaluator<EDiv<ELit<U6>, ELit<U2>>>, U3);
        // 5 % 2 = 1
        assert_type_eq_all!(Evaluator<ERem<ELit<U5>, ELit<U2>>>, typenum::U1);
    }

    #[test]
    fn test_arithmetic_signed() {
        // -3 + 2 = -1
        assert_type_eq_all!(Evaluator<EAdd<ELit<N3>, ELit<P2>>>, typenum::N1);
        // 2 * 3 = 6 (P6)
        assert_type_eq_all!(Evaluator<EMul<ELit<P2>, ELit<P3>>>, P6);
    }

    #[test]
    fn test_composition() {
        // (2 + 3) * 4 = 20
        assert_type_eq_all!(
            Evaluator<EMul<EAdd<ELit<U2>, ELit<U3>>, ELit<U4>>>,
            typenum::U20
        );
    }
}
