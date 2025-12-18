//! Church Booleans
//!
//! Church encoding of boolean values:
//! - `True`: λt f. t
//! - `False`: λt f. f

use std::marker::PhantomData;

use crate::{
    kernel::traits::Apply,
    lambda::{
        Lambda,
        traits::{LBool, LTerm},
    },
};

// =========================================================================
// Church Booleans
// =========================================================================

/// Church True: λt f. t
pub struct LTrue;
/// Church False: λt f. f
pub struct LFalse;

impl Lambda for LTrue {
    type Output = LTrue;
}
impl LTerm for LTrue {}
impl LBool for LTrue {}

impl Lambda for LFalse {
    type Output = LFalse;
}
impl LTerm for LFalse {}
impl LBool for LFalse {}

// Partial Application States
pub struct LTrue1<T>(PhantomData<T>);
pub struct LFalse1<T>(PhantomData<T>);

// --- True Implementation ---
// λt f. t
impl<T> Apply<T> for LTrue {
    type Output = LTrue1<T>;
}
impl<T, F> Apply<F> for LTrue1<T> {
    type Output = T;
}

// --- False Implementation ---
// λt f. f
impl<T> Apply<T> for LFalse {
    type Output = LFalse1<T>;
}
impl<T, F> Apply<F> for LFalse1<T> {
    type Output = F;
}

// =========================================================================
// If Expression
// =========================================================================

/// Pure If Alias: ((P T) E)
pub type LPureIf<P, T, E> = <<P as Apply<T>>::Output as Apply<E>>::Output;

/// Church If Struct
pub struct LIf<P, T, E>(PhantomData<(P, T, E)>);

impl<P, T, E> Lambda for LIf<P, T, E>
where
    P: Apply<T>,
    <P as Apply<T>>::Output: Apply<E>,
{
    type Output = LPureIf<P, T, E>;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;
    use crate::eval::Evaluate;

    #[test]
    fn test_church_bools_basic() {
        struct A;
        struct B;
        // Using Structs
        type TrueRes = Evaluate<LIf<LTrue, A, B>>;
        type FalseRes = Evaluate<LIf<LFalse, A, B>>;
        assert_type_eq_all!(TrueRes, A);
        assert_type_eq_all!(FalseRes, B);
    }
}
