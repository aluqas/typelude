//! **Type-Level Tree Data**
//!
//! Pure data structures for type-level binary search trees.

use std::marker::PhantomData;

use typelude_core::Eval;

/// Marker Trait
pub trait IsTree {}

/// Empty node (Leaf)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Nil;

/// Non-empty node (Internal Node)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Tree<Val, Left: IsTree, Right: IsTree>(pub PhantomData<(Val, Left, Right)>);

impl IsTree for Nil {}
impl Eval for Nil {
    type Output = Self;
}

impl<V, L: IsTree, R: IsTree> IsTree for Tree<V, L, R> {}
impl<V, L: IsTree, R: IsTree> Eval for Tree<V, L, R> {
    type Output = Self;
}
