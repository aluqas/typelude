//! **Type-Level Array Data**
//!
//! Pure data structures for type-level arrays (Cons Lists).

use std::marker::PhantomData;

use typelude_core::Eval;

/// **Marker Trait**
///
/// Represents that a type is a Cons List (Collection).
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a Cons List",
    label = "not a list",
    note = "ensure `{Self}` is either `Nil` or `Array`"
)]
pub trait IsList {}

/// Termination of Array (Empty List).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Nil;

/// Type-level array (Cons Cell)
///
/// - `Head`: Any type
/// - `Tail`: Rest part (recursive `Array`, `Nil` is termination)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Array<Head, Tail: IsList>(pub PhantomData<(Head, Tail)>);

impl IsList for Nil {}
impl Eval for Nil {
    type Output = Self;
}

impl<Head, Tail: IsList> IsList for Array<Head, Tail> {}
impl<Head, Tail: IsList> Eval for Array<Head, Tail> {
    type Output = Self;
}

/// Macro for easily creating type lists
#[macro_export]
macro_rules! tyarray {
    // Empty list
    () => { $crate::model::col::array::Nil };
    // List with length 1 (with optional trailing comma)
    ($n:ty $(,)?) => { $crate::model::col::array::Array<$n, $crate::model::col::array::Nil> };
    // List with length 2 or more (with optional trailing comma)
    ($n:ty, $($tail:ty),+ $(,)?) => { $crate::model::col::array::Array<$n, $crate::tyarray![$($tail),+]> };
}
