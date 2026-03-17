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
    type Output = Some<crate::eval_twice!(EApp<Op, ELit<T>>)>;
}

impl<Op> OptionMap<Op> for None {
    type Output = None;
}

crate::def_expr_via_trait!(
    /// Expression: Unwrap an option
    pub EUnwrap<Opt>
    args [Opt]
    where [~Opt: Unwrap]
    => <~Opt as Unwrap>::Output
);

crate::def_expr_via_trait!(
    /// Expression: Unwrap with default
    pub EUnwrapOr<Opt, Default>
    args [Opt, Default]
    where [~Opt: UnwrapOr<~Default>]
    => <~Opt as UnwrapOr<~Default>>::Output
);

crate::def_expr_via_trait!(
    /// Expression: Check if Option is Some
    pub EIsSome<Opt>
    args [Opt]
    where [~Opt: Option]
    => <~Opt as Option>::IsSome
);

crate::def_expr_via_trait!(
    /// Expression: Check if Option is None
    pub EIsNone<Opt>
    args [Opt]
    where [~Opt: Option]
    => <~Opt as Option>::IsNone
);

crate::def_expr_via_trait!(
    /// Expression: Map a function over an Option
    pub EMap<Op, Opt>
    args [Opt]
    where [~Opt: OptionMap<Op>]
    => <~Opt as OptionMap<Op>>::Output
);

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

    #[test]
    fn test_exprs() {
        use crate::core::ELit;
        type S = ELit<Some<U42>>;
        type N = ELit<None>;

        assert_type_eq_all!(Evaluate<EUnwrap<S>>, U42);
        assert_type_eq_all!(Evaluate<EUnwrapOr<N, ELit<typenum::U0>>>, typenum::U0);
        assert_type_eq_all!(Evaluate<EIsSome<S>>, True);
        assert_type_eq_all!(Evaluate<EIsNone<N>>, True);
    }
}
