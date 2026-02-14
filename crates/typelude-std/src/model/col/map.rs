//! **Type-Level Map Data**
//!
//! Pure data structures for type-level maps.

use std::marker::PhantomData;

use typelude_std::core::Eval;

/// Marker Trait
pub trait IsMap {}

/// Empty type-level map
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Nil;

/// Non-empty type-level map: key-value pair with tail
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Map<Key, Value, Tail: IsMap>(pub PhantomData<(Key, Value, Tail)>);

impl IsMap for Nil {}
impl Eval for Nil {
    type Output = Self;
}

impl<K, V, T: IsMap> IsMap for Map<K, V, T> {}
impl<K, V, T: IsMap> Eval for Map<K, V, T> {
    type Output = Self;
}
