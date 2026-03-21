//! **Type-Level Map**
//!
//! Type-level associative map (key-value store) and its operations.

use typenum::{B0, B1, Bit, IsEqual};

/// Marker trait for type-level maps
pub use crate::model::col::map::IsMap as TypeMap;
// Re-export kernel types for convenience/compatibility if mostly used from here
pub use crate::model::col::map::{Map, Nil};
pub use crate::std::traits::{Foldable, ToArray};
use crate::{
    model::{
        col::array::{Array, Nil as NilArray},
        prim::bool::{False, True},
    },
    std::prim::option::{None, Some},
};

/// Get a value by key, returns Some<V> or None.
///
/// Uses type equality comparison to traverse the map.
///
/// # Examples
///
/// ```ignore
/// type Res = <Map<U1, i32, Nil> as MapGet<U1>>::Output; // Some<i32>
/// ```
pub trait MapGet<Key> {
    type Output; // Some<V> or None
}

impl<K> MapGet<K> for Nil {
    type Output = None;
}

// Helper for MapGet dispatch based on key equality.
#[doc(hidden)]
pub trait MapGetHelper<Key, NodeKey, NodeValue, Tail, IsEq> {
    type Output;
}

impl<Key, NodeKey, NodeValue, Tail> MapGetHelper<Key, NodeKey, NodeValue, Tail, B1> for () {
    type Output = Some<NodeValue>;
}

impl<Key, NodeKey, NodeValue, Tail> MapGetHelper<Key, NodeKey, NodeValue, Tail, B0> for ()
where
    Tail: TypeMap + MapGet<Key>,
{
    type Output = <Tail as MapGet<Key>>::Output;
}

impl<K, NK, V, T: TypeMap> MapGet<K> for Map<NK, V, T>
where
    K: IsEqual<NK>,
    <K as IsEqual<NK>>::Output: Bit,
    (): MapGetHelper<K, NK, V, T, <K as IsEqual<NK>>::Output>,
{
    type Output = <() as MapGetHelper<K, NK, V, T, <K as IsEqual<NK>>::Output>>::Output;
}

/// Insert a key-value pair (prepend, allows shadowing).
///
/// # Examples
///
/// ```ignore
/// type NewMap = <Nil as MapInsert<U1, i32>>::Output;
/// ```
pub trait MapInsert<Key, Value> {
    type Output: TypeMap;
}

impl<K, V, M: TypeMap> MapInsert<K, V> for M {
    type Output = Map<K, V, M>;
}

/// Check if map contains a key.
///
/// # Examples
///
/// ```ignore
/// type HasKey = <Map<U1, i32, Nil> as MapContains<U1>>::Output; // True
/// ```
pub trait MapContains<Key> {
    type Output; // True or False
}

impl<K> MapContains<K> for Nil {
    type Output = False;
}

// Helper for MapContains dispatch based on key equality.
#[doc(hidden)]
pub trait MapContainsHelper<Key, NodeKey, Tail, IsEq> {
    type Output;
}

impl<Key, NodeKey, Tail> MapContainsHelper<Key, NodeKey, Tail, B1> for () {
    type Output = True;
}

impl<Key, NodeKey, Tail> MapContainsHelper<Key, NodeKey, Tail, B0> for ()
where
    Tail: TypeMap + MapContains<Key>,
{
    type Output = <Tail as MapContains<Key>>::Output;
}

impl<K, NK, V, T: TypeMap> MapContains<K> for Map<NK, V, T>
where
    K: IsEqual<NK>,
    <K as IsEqual<NK>>::Output: Bit,
    (): MapContainsHelper<K, NK, T, <K as IsEqual<NK>>::Output>,
{
    type Output = <() as MapContainsHelper<K, NK, T, <K as IsEqual<NK>>::Output>>::Output;
}

/// Extract all keys as an Array
pub trait MapKeys {
    type Output: crate::model::col::array::IsList;
}

impl MapKeys for Nil {
    type Output = crate::model::col::array::Nil;
}

impl<K, V, T: TypeMap + MapKeys> MapKeys for Map<K, V, T> {
    type Output = crate::model::col::array::Array<K, <T as MapKeys>::Output>;
}

/// Extract all values as an Array
pub trait MapValues {
    type Output: crate::model::col::array::IsList;
}

impl MapValues for Nil {
    type Output = crate::model::col::array::Nil;
}

impl<K, V, T: TypeMap + MapValues> MapValues for Map<K, V, T> {
    type Output = crate::model::col::array::Array<V, <T as MapValues>::Output>;
}

impl ToArray for Nil {
    type Output = NilArray;
}

impl<K, V, T> ToArray for Map<K, V, T>
where
    T: TypeMap + ToArray,
{
    type Output = Array<(K, V), <T as ToArray>::Output>;
}

impl<Op, Init> Foldable<Op, Init> for Nil
where
    NilArray: crate::std::col::array::Foldable<Op, Init>,
{
    type Output = <NilArray as crate::std::col::array::Foldable<Op, Init>>::Output;
}

impl<K, V, T, Op, Init> Foldable<Op, Init> for Map<K, V, T>
where
    Map<K, V, T>: ToArray,
    <Map<K, V, T> as ToArray>::Output: crate::std::col::array::Foldable<Op, Init>,
{
    type Output =
        <<Map<K, V, T> as ToArray>::Output as crate::std::col::array::Foldable<Op, Init>>::Output;
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

crate::typelude_macros::ty_fn! {
    /// Get a value from a map by key.
    pub struct FMapGet<MapTy>
    {
        type Output = FMapGetCaptured<MapTy>;
    }
}

crate::typelude_macros::ty_fn! {
    /// Get a value from a map by key.
    pub struct FMapGetCaptured<MapTy, Key>
    where [MapTy: MapGet<Key>]
    {
        type Output = <MapTy as MapGet<Key>>::Output;
    }
}

crate::typelude_macros::ty_fn! {
    /// Insert a key-value pair into a map.
    pub struct FMapInsert<MapTy>
    {
        type Output = FMapInsertCaptured1<MapTy>;
    }
}

crate::typelude_macros::ty_fn! {
    /// Insert a key-value pair into a map.
    pub struct FMapInsertCaptured1<MapTy, Key>
    {
        type Output = FMapInsertCaptured2<MapTy, Key>;
    }
}

crate::typelude_macros::ty_fn! {
    /// Insert a key-value pair into a map.
    pub struct FMapInsertCaptured2<MapTy, Key, Value>
    where [MapTy: MapInsert<Key, Value>]
    {
        type Output = <MapTy as MapInsert<Key, Value>>::Output;
    }
}

crate::typelude_macros::ty_fn! {
    /// Check whether a map contains a key.
    pub struct FMapContains<MapTy>
    {
        type Output = FMapContainsCaptured<MapTy>;
    }
}

crate::typelude_macros::ty_fn! {
    /// Check whether a map contains a key.
    pub struct FMapContainsCaptured<MapTy, Key>
    where [MapTy: MapContains<Key>]
    {
        type Output = <MapTy as MapContains<Key>>::Output;
    }
}

crate::typelude_macros::ty_fn! {
    /// Extract the keys of a map.
    pub struct FMapKeys<MapTy>
    where [MapTy: MapKeys]
    {
        type Output = <MapTy as MapKeys>::Output;
    }
}

crate::typelude_macros::ty_fn! {
    /// Extract the values of a map.
    pub struct FMapValues<MapTy>
    where [MapTy: MapValues]
    {
        type Output = <MapTy as MapValues>::Output;
    }
}

crate::typelude_macros::ty_fn! {
    /// Get the length of a map.
    pub struct FMapLen<MapTy>
    where [MapTy: MapLen]
    {
        type Output = <MapTy as MapLen>::Output;
    }
}

/// Get value from map by key.
pub type EMapGet<MapTy, Key> = crate::core::ECall2<FMapGet, MapTy, Key>;
/// Insert key-value pair into map.
pub type EMapInsert<MapTy, Key, Value> = crate::core::ECall3<FMapInsert, MapTy, Key, Value>;
/// Check if map contains key.
pub type EMapContains<MapTy, Key> = crate::core::ECall2<FMapContains, MapTy, Key>;
/// Get all keys from map.
pub type EMapKeys<MapTy> = crate::core::ECall<FMapKeys, MapTy>;
/// Get all values from map.
pub type EMapValues<MapTy> = crate::core::ECall<FMapValues, MapTy>;
/// Get the size of a map.
pub type EMapLen<MapTy> = crate::core::ECall<FMapLen, MapTy>;

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
    use typenum::{U1, U2, U3};

    use super::*;
    use crate::{
        model::col::array::{Array, Nil as NilArray},
        std::prim::option::{None, Some},
    };

    crate::typelude_macros::ty_fn! {
        struct CountEntries<Acc> {
            type Output = CountEntriesCaptured<Acc>;
        }
    }

    crate::typelude_macros::ty_fn! {
        struct CountEntriesCaptured<Acc, Entry>
        where [Acc: std::ops::Add<typenum::B1>]
        {
            type Output = typenum::Add1<Acc>;
        }
    }

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

        type AsArray = <Map as ToArray>::Output;
        assert_type_eq_all!(AsArray, Array<(U1, i32), Array<(U2, f64), NilArray>>);
        assert_type_eq_all!(<Map as Foldable<CountEntries, typenum::U0>>::Output, typenum::U2);
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

    #[test]
    fn test_map_get_recursion() {
        // Map layout: U2 -> f64, U1 -> i32 (U2 is head, U1 is in tail)
        type TestMap = super::Map<U2, f64, super::Map<U1, i32, Nil>>;

        // Get head key (U2)
        type Res1 = <TestMap as MapGet<U2>>::Output;
        assert_type_eq_all!(Res1, Some<f64>);

        // Get tail key (U1) - THIS is what was broken before
        type Res2 = <TestMap as MapGet<U1>>::Output;
        assert_type_eq_all!(Res2, Some<i32>);

        // Get non-existent key (U3)
        type Res3 = <TestMap as MapGet<U3>>::Output;
        assert_type_eq_all!(Res3, None);
    }

    #[test]
    fn test_map_contains_recursion() {
        // Map layout: U2 -> f64, U1 -> i32
        type TestMap = super::Map<U2, f64, super::Map<U1, i32, Nil>>;

        // Contains head key (U2)
        type Has2 = <TestMap as MapContains<U2>>::Output;
        assert_type_eq_all!(Has2, True);

        // Contains tail key (U1) - THIS is what was broken before
        type Has1 = <TestMap as MapContains<U1>>::Output;
        assert_type_eq_all!(Has1, True);

        // Contains non-existent key (U3)
        type Has3 = <TestMap as MapContains<U3>>::Output;
        assert_type_eq_all!(Has3, False);
    }

    #[test]
    fn test_map_get_empty() {
        type Res = <Nil as MapGet<U1>>::Output;
        assert_type_eq_all!(Res, None);
    }

    #[test]
    fn test_map_contains_empty() {
        type Res = <Nil as MapContains<U1>>::Output;
        assert_type_eq_all!(Res, False);
    }
}
