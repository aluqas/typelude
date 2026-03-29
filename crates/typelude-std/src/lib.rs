// Unstable features required for type-level programming

// Use for NegativeEquality
#![cfg_attr(feature = "nightly", feature(specialization))]
// Use for Integration to value
#![cfg_attr(feature = "nightly", feature(generic_const_exprs))]
#![allow(incomplete_features)]
#![recursion_limit = "65536"]

use std::marker::PhantomData;

#[macro_use]
pub mod macros;
mod r#while;
mod core;


struct Same<T>(PhantomData<T>);
struct None;

trait Option<T> {
    type IsSome;
    type IsNone;
}

impl<T> Option<T> for Same<T> {
    type IsSome = True;
    type IsNone = False;
}

impl Option<None> for None {
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

/// ```
/// where
///   Add<U11, U11>: Validate<Check = U22>,
/// ```
#[diagnostic::on_unimplemented(
    message = "Unexpected types",
    label = "validation failed",
    note = "expected: {Check} (the result of the addition)"
)]
trait Assert<Check> {}
impl<T> Assert<T> for T {}

trait Reify<Into> {}
trait From {}
trait Into {}

pub trait Eval {
    type Output;
}

impl<T> Eval for T {
    type Output = T::Output;
}

pub trait OpAdd<Lhs, Rhs> {
    type Output;
}
type Add<Lhs, Rhs> = <Lhs as OpAdd<Rhs>>::Output;

pub trait OpSub<Lhs, Rhs> {
    type Output;
}
type Sub<Lhs, Rhs> = <Lhs as OpSub<Rhs>>::Output;

pub trait OpMul<Lhs, Rhs> {
    type Output;
}
type Mul<Lhs, Rhs> = <Lhs as OpMul<Rhs>>::Output;
