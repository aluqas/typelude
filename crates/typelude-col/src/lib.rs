//! Type-level collection utilities.

use ::std::marker::PhantomData;

pub mod model {
    pub use typelude_std::model::col::{array::*, map::*, tree_array::*, tree_map::*};
}

pub mod std {
    pub use typelude_std::std::col::{array::*, map::*, tree_array::*, tree_map::*};
}

pub struct TArr<Val, A>{
    first: Val,
    rest: A,
}
pub struct TTerm;

#[macro_export]
macro_rules! tarr {
    () => ( $crate::ATerm );
    ($n:ty) => ( $crate::TArr<$n, $crate::ATerm> );
    ($n:ty,) => ( $crate::TArr<$n, $crate::ATerm> );
    ($n:ty, $($tail:ty),+) => ( $crate::TArr<$n, tarr![$($tail),+]> );
    ($n:ty, $($tail:ty),+,) => ( $crate::TArr<$n, tarr![$($tail),+]> );
    ($n:ty | $rest:ty) => ( $crate::TArr<$n, $rest> );
    ($n:ty, $($tail:ty),+ | $rest:ty) => ( $crate::TArr<$n, tarr![$($tail),+ | $rest]> );
}

/// **Marker Trait**
/// Represents that a type is a type-level array.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a type-level array",
    label = "not a TYPE-LEVEL array",
    note = "ensure `{Self}` is of the form `TArr<_, _>`"
)]
trait TypeArray {
    const LEN: usize;
}

impl TTerm {
    // 0-indexed
    const LEN: usize = 0;
}

impl<Val, A: TypeArray> TypeArray for TArr<Val, A> {
    const LEN: usize = A::LEN + 1;
}

#[cfg(test)]
mod tests {
    use super::*;

}
