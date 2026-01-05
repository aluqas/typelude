//! **Type-Level Option**
//!
//! Type-level representation of `Option<T>` for type-level computations.

use std::marker::PhantomData;

use typelude_core::{Eval, Evaluate};

use crate::data::primitives::bool::{TyFalse, TyTrue};

// =============================================================================
// Data Structures
// =============================================================================

/// Type-level `Some<T>` — wraps a value
pub struct TySome<T>(PhantomData<T>);

/// Type-level `None` — absence of value
pub struct TyNone;

// =============================================================================
// TyOption Trait
// =============================================================================

/// Trait for type-level Option operations
pub trait TyOption {
    /// Is this a Some variant?
    type IsSome;
    /// Is this a None variant?
    type IsNone;
}

impl<T> TyOption for TySome<T> {
    type IsSome = TyTrue;
    type IsNone = TyFalse;
}

impl TyOption for TyNone {
    type IsSome = TyFalse;
    type IsNone = TyTrue;
}

// =============================================================================
// Unwrap Operations
// =============================================================================

/// Unwrap a TySome, compile error on TyNone
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be unwrapped (it is TyNone)",
    label = "Unwrap failed",
    note = "ensure `{Self}` is `TySome<T>`, not `TyNone`"
)]
pub trait Unwrap {
    type Output;
}

impl<T> Unwrap for TySome<T> {
    type Output = T;
}

// Note: TyNone does NOT implement Unwrap — attempting to unwrap TyNone is a compile error

/// Unwrap with a default value
pub trait UnwrapOr<Default> {
    type Output;
}

impl<T, D> UnwrapOr<D> for TySome<T> {
    type Output = T;
}

impl<D> UnwrapOr<D> for TyNone {
    type Output = D;
}

// =============================================================================
// Map Operation
// =============================================================================

/// Map a function over an Option
pub trait OptionMap<Op> {
    type Output;
}

impl<T, Op> OptionMap<Op> for TySome<T>
where
    Op: Eval,
    Evaluate<Op>: crate::traits::Apply<T>,
{
    type Output = TySome<<Evaluate<Op> as crate::traits::Apply<T>>::Output>;
}

impl<Op> OptionMap<Op> for TyNone {
    type Output = TyNone;
}

// =============================================================================
// Expression Wrappers
// =============================================================================

/// Expression: Unwrap an option
pub struct EUnwrap<Opt>(PhantomData<Opt>);

impl<Opt> Eval for EUnwrap<Opt>
where
    Opt: Eval,
    Evaluate<Opt>: Unwrap,
{
    type Output = <Evaluate<Opt> as Unwrap>::Output;
}

/// Expression: Unwrap with default
pub struct EUnwrapOr<Opt, Default>(PhantomData<(Opt, Default)>);

impl<Opt, Default> Eval for EUnwrapOr<Opt, Default>
where
    Opt: Eval,
    Default: Eval,
    Evaluate<Opt>: UnwrapOr<Evaluate<Default>>,
{
    type Output = <Evaluate<Opt> as UnwrapOr<Evaluate<Default>>>::Output;
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typenum::U42;

    use super::*;

    #[test]
    fn test_some_unwrap() {
        type Result = <TySome<U42> as Unwrap>::Output;
        assert_type_eq_all!(Result, U42);
    }

    #[test]
    fn test_unwrap_or_some() {
        type Result = <TySome<U42> as UnwrapOr<typenum::U0>>::Output;
        assert_type_eq_all!(Result, U42);
    }

    #[test]
    fn test_unwrap_or_none() {
        type Result = <TyNone as UnwrapOr<U42>>::Output;
        assert_type_eq_all!(Result, U42);
    }

    #[test]
    fn test_is_some() {
        assert_type_eq_all!(<TySome<U42> as TyOption>::IsSome, TyTrue);
        assert_type_eq_all!(<TyNone as TyOption>::IsSome, TyFalse);
    }
}
