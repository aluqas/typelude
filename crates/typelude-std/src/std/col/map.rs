//! **Type-Level Map**
//!
//! Type-level associative map (key-value store) and its operations.

use typelude_core::Evaluate;
use typelude_macros::def_op;

/// Marker trait for type-level maps
pub use crate::data::collections::map::IsMap as TypeMap;
// Re-export kernel types for convenience/compatibility if mostly used from here
pub use crate::data::collections::map::{Map, Nil};
use crate::{
    data::primitives::bool::{False, True},
    std::prim::option::{None, Some},
};

/// Get a value by key, returns Some<V> or None
/// Uses type identity (same type = found)
pub trait MapGet<Key> {
    type Output; // Some<V> or None
}

impl<K> MapGet<K> for Nil {
    type Output = None;
}

// Key matches exactly (same type)
impl<K, V, T: TypeMap> MapGet<K> for Map<K, V, T> {
    type Output = Some<V>;
}

// Key doesn't match, continue search in tail
// This uses nightly specialization or manual impl for specific types
// For stable Rust, we rely on the fact that only exact type matches work

/// Insert a key-value pair (prepend, allows shadowing)
pub trait MapInsert<Key, Value> {
    type Output: TypeMap;
}

impl<K, V, M: TypeMap> MapInsert<K, V> for M {
    type Output = Map<K, V, M>;
}

/// Check if map contains a key
pub trait MapContains<Key> {
    type Output; // True or False
}

impl<K> MapContains<K> for Nil {
    type Output = False;
}

// Key matches exactly
impl<K, V, T: TypeMap> MapContains<K> for Map<K, V, T> {
    type Output = True;
}

/// Extract all keys as an Array
pub trait MapKeys {
    type Output: crate::data::collections::array::IsList;
}

impl MapKeys for Nil {
    type Output = crate::data::collections::array::Nil;
}

impl<K, V, T: TypeMap + MapKeys> MapKeys for Map<K, V, T> {
    type Output = crate::data::collections::array::Array<K, <T as MapKeys>::Output>;
}

/// Extract all values as an Array
pub trait MapValues {
    type Output: crate::data::collections::array::IsList;
}

impl MapValues for Nil {
    type Output = crate::data::collections::array::Nil;
}

impl<K, V, T: TypeMap + MapValues> MapValues for Map<K, V, T> {
    type Output = crate::data::collections::array::Array<V, <T as MapValues>::Output>;
}

/// Get the size of the map
pub trait MapLen {
    type Output;
}

impl MapLen for Nil {
    type Output = typenum::U0;
}

impl<K, V, T: TypeMap + MapLen> MapLen for Map<K, V, T>
where
    <T as MapLen>::Output: std::ops::Add<typenum::B1>,
{
    type Output = <<T as MapLen>::Output as std::ops::Add<typenum::B1>>::Output;
}

def_op! {
    /// Get value from map by key
    name: OpMapGet,
    args: (Map, Key),
    ast: EMapGet {
        where: [
            Evaluate<Map>: MapGet<Evaluate<Key>>
        ],
        type Output = <Evaluate<Map> as MapGet<Evaluate<Key>>>::Output
    }
}

def_op! {
    /// Insert key-value pair into map
    name: OpMapInsert,
    args: (Map, Key, Value),
    ast: EMapInsert {
        where: [
            Evaluate<Map>: MapInsert<Evaluate<Key>, Evaluate<Value>>
        ],
        type Output = <Evaluate<Map> as MapInsert<Evaluate<Key>, Evaluate<Value>>>::Output
    }
}

def_op! {
    /// Check if map contains key
    name: OpMapContains,
    args: (Map, Key),
    ast: EMapContains {
        where: [
            Evaluate<Map>: MapContains<Evaluate<Key>>
        ],
        type Output = <Evaluate<Map> as MapContains<Evaluate<Key>>>::Output
    }
}

def_op! {
    /// Get all keys from map
    name: OpMapKeys,
    args: (Map),
    ast: EMapKeys {
        where: [
            Evaluate<Map>: MapKeys
        ],
        type Output = <Evaluate<Map> as MapKeys>::Output
    }
}

def_op! {
    /// Get all values from map
    name: OpMapValues,
    args: (Map),
    ast: EMapValues {
        where: [
            Evaluate<Map>: MapValues
        ],
        type Output = <Evaluate<Map> as MapValues>::Output
    }
}

/// Create a type-level map from key-value pairs
/// Usage: `tymap![(K1, V1), (K2, V2)]`
#[macro_export]
macro_rules! tymap {
    [] => { $crate::std::col::map::Nil };
    [($k:ty, $v:ty)] => {
        $crate::std::col::map::Map<$k, $v, $crate::std::col::map::Nil>
    };
    [($k:ty, $v:ty), $(($ks:ty, $vs:ty)),+ $(,)?] => {
        $crate::std::col::map::Map<$k, $v, $crate::tymap![$(($ks, $vs)),+]>
    };
}

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typenum::{U1, U2};

    use super::*;
    use crate::data::collections::array::{Array, Nil as NilArray};

    #[test]
    fn test_map_insert_keys_values() {
        // Create map: {U1 => i32, U2 => f64}
        type Map = super::Map<U1, i32, super::Map<U2, f64, Nil>>;

        // Keys
        type Keys = <Map as MapKeys>::Output;
        assert_type_eq_all!(Keys, Array<U1, Array<U2, NilArray>>);

        // Values
        type Values = <Map as MapValues>::Output;
        assert_type_eq_all!(Values, Array<i32, Array<f64, NilArray>>);
    }

    #[test]
    fn test_map_macro() {
        type TestMap = tymap![(U1, i32), (U2, f64)];
        type Keys = <TestMap as MapKeys>::Output;
        assert_type_eq_all!(Keys, Array<U1, Array<U2, NilArray>>);
    }

    #[test]
    fn test_map_insert() {
        type Map1 = Nil;
        type Map2 = <Map1 as MapInsert<U1, i32>>::Output;
        type Map3 = <Map2 as MapInsert<U2, f64>>::Output;

        type Keys = <Map3 as MapKeys>::Output;
        // Insert prepends, so order is U2, U1
        assert_type_eq_all!(Keys, Array<U2, Array<U1, NilArray>>);
    }
}
