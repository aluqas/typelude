//! **Type-Level Option**
//!
//! Type-level representation of `Option<T>` for type-level computations.

use std::marker::PhantomData;

use typelude_std::core::{EApp, ELit, Eval, Evaluate};

use crate::std::prim::bool::{False, True};
/// Type-level `Some<T>` — wraps a value
pub struct Some<T>(PhantomData<T>);

/// Type-level `None` — absence of value
pub struct None;
/// Trait for type-level Option operations
pub trait Option {
    /// Is this a Some variant?
    type IsSome;
    /// Is this a None variant?
    type IsNone;
}

impl<T> Option for Some<T> {
    type IsSome = True;
    type IsNone = False;
}

impl Option for None {
    type IsSome = False;
    type IsNone = True;
}
/// Unwrap a Some, compile error on None
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot be unwrapped (it is None)",
    label = "Unwrap failed",
    note = "ensure `{Self}` is `Some<T>`, not `None`"
)]
pub trait Unwrap {
    type Output;
}

impl<T> Unwrap for Some<T> {
    type Output = T;
}

// Note: None does NOT implement Unwrap — attempting to unwrap None is a compile
// error

/// Unwrap with a default value
pub trait UnwrapOr<Default> {
    type Output;
}

impl<T, D> UnwrapOr<D> for Some<T> {
    type Output = T;
}

impl<D> UnwrapOr<D> for None {
    type Output = D;
}
/// Map a function over an Option (via `EApp` + `ELit`)
pub trait OptionMap<Op> {
    type Output;
}

impl<T, Op> OptionMap<Op> for Some<T>
where
    Op: Eval,
    EApp<Op, ELit<T>>: Eval,
    Evaluate<EApp<Op, ELit<T>>>: Eval,
{
    type Output = Some<Evaluate<Evaluate<EApp<Op, ELit<T>>>>>;
}

impl<Op> OptionMap<Op> for None {
    type Output = None;
}
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
#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typenum::U42;

    use super::*;

    #[test]
    fn test_some_unwrap() {
        type Result = <Some<U42> as Unwrap>::Output;
        assert_type_eq_all!(Result, U42);
    }

    #[test]
    fn test_unwrap_or_some() {
        type Result = <Some<U42> as UnwrapOr<typenum::U0>>::Output;
        assert_type_eq_all!(Result, U42);
    }

    #[test]
    fn test_unwrap_or_none() {
        type Result = <None as UnwrapOr<U42>>::Output;
        assert_type_eq_all!(Result, U42);
    }

    #[test]
    fn test_is_some() {
        assert_type_eq_all!(<Some<U42> as Option>::IsSome, True);
        assert_type_eq_all!(<None as Option>::IsSome, False);
    }
}
