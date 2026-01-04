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

use crate::{
    eval::Eval,
    lambda::{LApp, Lambda},
};

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
impl Lambda for LForce {
    type Output = LForce;
}

// Thunk<F, Arg> Force -> F Arg
impl<F, Arg> Lambda for LApp<LThunk<F, Arg>, LForce>
where
    F: Eval,
    Arg: Eval,
    LApp<F, Arg>: Lambda,
{
    type Output = <LApp<F, Arg> as Lambda>::Output;
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;

    use super::*;

    type App<F, A> = Evaluate<LApp<F, A>>;

    #[derive(Clone)]
    struct AddOne;
    impl Lambda for AddOne {
        type Output = AddOne;
    }

    #[derive(Clone)]
    struct Zero;
    impl Lambda for Zero {
        type Output = Zero;
    }

    #[derive(Clone)]
    struct One;
    impl Lambda for One {
        type Output = One;
    }

    impl Lambda for LApp<AddOne, Zero> {
        type Output = One;
    }

    #[test]
    fn test_thunk_force() {
        // Delayed computation: AddOne(Zero)
        type Delayed = LThunk<AddOne, Zero>;

        // Force it
        type Result = App<Delayed, LForce>;

        assert_type_eq_all!(Result, One);
    }
}
