//! **Type-Level Map**
//!
//! Type-level associative map (key-value store) and its operations.

use std::marker::PhantomData;

use typelude_core::Evaluate;
use typelude_macros::def_op;

use crate::{
    data::primitives::bool::{TyFalse, TyTrue},
    std::primitives::{
        array::{Cons, TyArray, TyNil},
        option::{TyNone, TySome},
    },
};

// =============================================================================
// Data Structure
// =============================================================================

/// Empty type-level map
pub struct TyMapNil;

/// Non-empty type-level map: key-value pair with tail
pub struct TyMapCons<Key, Value, Tail>(PhantomData<(Key, Value, Tail)>);

/// Marker trait for type-level maps
pub trait TypeMap {}

impl TypeMap for TyMapNil {}
impl<K, V, T: TypeMap> TypeMap for TyMapCons<K, V, T> {}

// =============================================================================
// Core Operations
// =============================================================================

/// Get a value by key, returns TySome<V> or TyNone
/// Uses type identity (same type = found)
pub trait MapGet<Key> {
    type Output; // TySome<V> or TyNone
}

impl<K> MapGet<K> for TyMapNil {
    type Output = TyNone;
}

// Key matches exactly (same type)
impl<K, V, T: TypeMap> MapGet<K> for TyMapCons<K, V, T> {
    type Output = TySome<V>;
}

// Key doesn't match, continue search in tail
// This uses nightly specialization or manual impl for specific types
// For stable Rust, we rely on the fact that only exact type matches work

/// Insert a key-value pair (prepend, allows shadowing)
pub trait MapInsert<Key, Value> {
    type Output: TypeMap;
}

impl<K, V, M: TypeMap> MapInsert<K, V> for M {
    type Output = TyMapCons<K, V, M>;
}

/// Check if map contains a key
pub trait MapContains<Key> {
    type Output; // TyTrue or TyFalse
}

impl<K> MapContains<K> for TyMapNil {
    type Output = TyFalse;
}

// Key matches exactly
impl<K, V, T: TypeMap> MapContains<K> for TyMapCons<K, V, T> {
    type Output = TyTrue;
}

/// Extract all keys as a TyArray
pub trait MapKeys {
    type Output: Cons;
}

impl MapKeys for TyMapNil {
    type Output = TyNil;
}

impl<K, V, T: TypeMap + MapKeys> MapKeys for TyMapCons<K, V, T>
where
    <T as MapKeys>::Output: Cons,
{
    type Output = TyArray<K, <T as MapKeys>::Output>;
}

/// Extract all values as a TyArray
pub trait MapValues {
    type Output: Cons;
}

impl MapValues for TyMapNil {
    type Output = TyNil;
}

impl<K, V, T: TypeMap + MapValues> MapValues for TyMapCons<K, V, T>
where
    <T as MapValues>::Output: Cons,
{
    type Output = TyArray<V, <T as MapValues>::Output>;
}

/// Get the size of the map
pub trait MapLen {
    type Output;
}

impl MapLen for TyMapNil {
    type Output = typenum::U0;
}

impl<K, V, T: TypeMap + MapLen> MapLen for TyMapCons<K, V, T>
where
    <T as MapLen>::Output: std::ops::Add<typenum::B1>,
{
    type Output = <<T as MapLen>::Output as std::ops::Add<typenum::B1>>::Output;
}

// =============================================================================
// Expression Wrappers via def_op!
// =============================================================================

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

// =============================================================================
// Convenience Macros
// =============================================================================

/// Create a type-level map from key-value pairs
/// Usage: `tymap![(K1, V1), (K2, V2)]`
#[macro_export]
macro_rules! tymap {
    [] => { $crate::std::primitives::map::TyMapNil };
    [($k:ty, $v:ty)] => {
        $crate::std::primitives::map::TyMapCons<$k, $v, $crate::std::primitives::map::TyMapNil>
    };
    [($k:ty, $v:ty), $(($ks:ty, $vs:ty)),+ $(,)?] => {
        $crate::std::primitives::map::TyMapCons<$k, $v, tymap![$(($ks, $vs)),+]>
    };
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typenum::{U1, U2, U3};

    use super::*;

    #[test]
    fn test_map_insert_keys_values() {
        // Create map: {U1 => i32, U2 => f64}
        type Map = TyMapCons<U1, i32, TyMapCons<U2, f64, TyMapNil>>;

        // Keys
        type Keys = <Map as MapKeys>::Output;
        assert_type_eq_all!(Keys, TyArray<U1, TyArray<U2, TyNil>>);

        // Values
        type Values = <Map as MapValues>::Output;
        assert_type_eq_all!(Values, TyArray<i32, TyArray<f64, TyNil>>);
    }

    #[test]
    fn test_map_macro() {
        type Map = tymap![(U1, i32), (U2, f64)];
        type Keys = <Map as MapKeys>::Output;
        assert_type_eq_all!(Keys, TyArray<U1, TyArray<U2, TyNil>>);
    }

    #[test]
    fn test_map_insert() {
        type Map1 = TyMapNil;
        type Map2 = <Map1 as MapInsert<U1, i32>>::Output;
        type Map3 = <Map2 as MapInsert<U2, f64>>::Output;

        type Keys = <Map3 as MapKeys>::Output;
        // Insert prepends, so order is U2, U1
        assert_type_eq_all!(Keys, TyArray<U2, TyArray<U1, TyNil>>);
    }
}
