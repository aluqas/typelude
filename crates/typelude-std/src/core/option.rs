//! Type-level `Option` primitives.

use core::marker::PhantomData;

use typelude_bool::{False, True};

use super::{Unwrap, UnwrapOr, Value};

/// Type-level `Some<T>`.
pub struct Some<T>(pub PhantomData<T>);

/// Type-level `None`.
pub struct None;

/// Capability trait for type-level option values.
pub trait IsOption {
    type IsSome;
    type IsNone;
}

impl<T> Value for Some<T> {}
impl Value for None {}

impl<T> IsOption for Some<T> {
    type IsSome = True;
    type IsNone = False;
}

impl IsOption for None {
    type IsSome = False;
    type IsNone = True;
}

impl<T> Unwrap for Some<T> {
    type Output = T;
}

impl<T, Default> UnwrapOr<Default> for Some<T> {
    type Output = T;
}

impl<Default> UnwrapOr<Default> for None {
    type Output = Default;
}
