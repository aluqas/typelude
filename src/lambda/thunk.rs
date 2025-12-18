//! Thunk: Lazy Evaluation Primitive
//!
//! A `Thunk` represents a suspended computation.
//!
//! - `Thunk<F, Arg>`: A pending application of `F` to `Arg`.
//! - `Force`: A token that triggers the evaluation of a `Thunk`.
//!
//! Usage:
//! `Thunk<F, Arg> Force` -> `F Arg`

use std::marker::PhantomData;

use crate::{kernel::traits::Apply, lambda::Lambda};

// =========================================================================
// Thunk
// =========================================================================

/// Thunk: Represents a delayed application of `F` to `Arg`.
///
/// It is a passive value until `Force` is applied.
pub struct LThunk<F, Arg>(PhantomData<(F, Arg)>);

impl<F, Arg> Lambda for LThunk<F, Arg> {
    type Output = LThunk<F, Arg>;
}

// =========================================================================
// Force
// =========================================================================

/// Force: Trigger evaluation of a Thunk.
pub struct LForce;

// Thunk<F, Arg> Force -> F Arg
impl<F, Arg> Apply<LForce> for LThunk<F, Arg>
where
    F: Apply<Arg>,
{
    type Output = <F as Apply<Arg>>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;

    struct AddOne;
    struct Zero;
    struct One;

    impl Apply<Zero> for AddOne {
        type Output = One;
    }

    #[test]
    fn test_thunk_force() {
        // Delayed computation: AddOne(Zero)
        type Delayed = LThunk<AddOne, Zero>;

        // Force it
        type Result = <Delayed as Apply<LForce>>::Output;

        assert_type_eq_all!(Result, One);
    }
}
