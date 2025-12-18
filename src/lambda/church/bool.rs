//! Church Booleans
//!
//! Church encoding of boolean values:
//! - `True`: λt f. t
//! - `False`: λt f. f

use std::marker::PhantomData;

use crate::kernel::traits::Apply;
use crate::lambda::Lambda;

// =========================================================================
// Church Booleans
// =========================================================================

/// Church True: λt f. t
pub struct True;
/// Church False: λt f. f
pub struct False;

impl Lambda for True {
    type Output = True;
}
impl Lambda for False {
    type Output = False;
}

// Partial Application States
pub struct True1<T>(PhantomData<T>);
pub struct False1<T>(PhantomData<T>);

// --- True Implementation ---
// λt f. t
impl<T> Apply<T> for True {
    type Output = True1<T>;
}
impl<T, F> Apply<F> for True1<T> {
    type Output = T;
}

// --- False Implementation ---
// λt f. f
impl<T> Apply<T> for False {
    type Output = False1<T>;
}
impl<T, F> Apply<F> for False1<T> {
    type Output = F;
}

// =========================================================================
// If Expression
// =========================================================================

/// Pure If Alias: ((P T) E)
pub type PureIf<P, T, E> = <<P as Apply<T>>::Output as Apply<E>>::Output;

/// Church If Struct
pub struct If<P, T, E>(PhantomData<(P, T, E)>);

impl<P, T, E> Lambda for If<P, T, E>
where
    P: Apply<T>,
    <P as Apply<T>>::Output: Apply<E>,
{
    type Output = PureIf<P, T, E>;
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
        type TrueRes = Evaluate<If<True, A, B>>;
        type FalseRes = Evaluate<If<False, A, B>>;
        assert_type_eq_all!(TrueRes, A);
        assert_type_eq_all!(FalseRes, B);
    }
}
