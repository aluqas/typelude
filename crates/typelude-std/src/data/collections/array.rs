//! **Type-Level Array Data**
//!
//! Pure data structures for type-level arrays (Cons Lists).

use std::marker::PhantomData;

use typelude_core::{Eval, Sealed};

/// **Marker Trait**
///
/// Represents that a type is a Cons List (Collection).
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a Cons List",
    label = "not a list",
    note = "ensure `{Self}` is either `TyNil` or `TyArray`"
)]
pub trait Cons: Sealed {}

/// Termination of TyArray (Empty List).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct TyNil;

/// Type-level array (Cons Cell)
///
/// - `Head`: Any type
/// - `Tail`: Rest part (recursive `TyArray`, `TyNil` is termination)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct TyArray<Head, Tail: Cons>(pub PhantomData<(Head, Tail)>);

impl Sealed for TyNil {}
impl Cons for TyNil {}
impl Eval for TyNil {
    type Output = Self;
}

impl<Head, Tail: Cons> Sealed for TyArray<Head, Tail> {}
impl<Head, Tail: Cons> Cons for TyArray<Head, Tail> {}
impl<Head, Tail: Cons> Eval for TyArray<Head, Tail> {
    type Output = Self;
}

/// Macro for easily creating type lists
#[macro_export]
macro_rules! tyarray {
    // Empty list
    () => { $crate::data::collections::array::TyNil };
    // List with length 1 (with optional trailing comma)
    ($n:ty $(,)?) => { $crate::data::collections::array::TyArray<$n, $crate::data::collections::array::TyNil> };
    // List with length 2 or more (with optional trailing comma)
    ($n:ty, $($tail:ty),+ $(,)?) => { $crate::data::collections::array::TyArray<$n, $crate::tyarray![$($tail),+]> };
}
