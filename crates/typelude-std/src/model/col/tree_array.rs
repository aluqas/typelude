//! **Type-Level TreeArray Data**
//!
//! Binary search tree for values only (sorted set).

use std::marker::PhantomData;

use typelude_core::Eval;

/// Marker Trait for TreeArray (value-only tree)
pub trait IsTreeArray {}

/// Empty node (Leaf)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Nil;

impl IsTreeArray for Nil {}
impl Eval for Nil {
    type Output = Self;
}

/// TreeArray: Value-only binary search tree node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct TreeArray<Value, Left: IsTreeArray, Right: IsTreeArray>(
    pub PhantomData<(Value, Left, Right)>,
);

impl<V, L: IsTreeArray, R: IsTreeArray> IsTreeArray for TreeArray<V, L, R> {}
impl<V, L: IsTreeArray, R: IsTreeArray> Eval for TreeArray<V, L, R> {
    type Output = Self;
}
