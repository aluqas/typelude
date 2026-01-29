//! **Type-Level TreeMap Data**
//!
//! Binary search tree for key-value pairs (asynchronous map).

use std::marker::PhantomData;

use typelude_std::core::Eval;

/// Marker Trait for TreeMap (key-value tree)
pub trait IsTreeMap {}

/// Empty node (Leaf)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Nil;

impl IsTreeMap for Nil {}
impl Eval for Nil {
    type Output = Self;
}

/// TreeMap: Key-Value binary search tree node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct TreeMap<Key, Value, Left: IsTreeMap, Right: IsTreeMap>(
    pub PhantomData<(Key, Value, Left, Right)>,
);

impl<K, V, L: IsTreeMap, R: IsTreeMap> IsTreeMap for TreeMap<K, V, L, R> {}
impl<K, V, L: IsTreeMap, R: IsTreeMap> Eval for TreeMap<K, V, L, R> {
    type Output = Self;
}
