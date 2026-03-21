//! **Type-Level TreeMap**
//!
//! Key-Value binary search tree (sorted map).

use typenum::{B0, B1, Bit, IsEqual, IsLess};

/// Marker trait for type-level TreeMap
pub use crate::model::col::tree_map::IsTreeMap;
// Re-export kernel types
pub use crate::model::col::tree_map::{Nil, TreeMap};
pub use crate::std::traits::{Foldable, ToArray};
use crate::{
    model::prim::bool::{False, True},
    std::prim::option::{None, Some},
};

// ============================================================================
// TreeMap Operations (Key-Value)
// ============================================================================

/// Insert a key-value pair into TreeMap.
///
/// Uses typenum comparison on Keys. If key exists, value is replaced.
pub trait TreeMapInsert<K, V> {
    type Output: IsTreeMap;
}

impl<K, V> TreeMapInsert<K, V> for Nil {
    type Output = TreeMap<K, V, Nil, Nil>;
}

#[doc(hidden)]
pub trait TreeMapInsertHelper<K, V, NodeKey, NodeValue, Left, Right, IsEq, IsLessResult> {
    type Output;
}

impl<K, V, NodeKey, NodeValue, Left, Right, IsLessResult>
    TreeMapInsertHelper<K, V, NodeKey, NodeValue, Left, Right, B1, IsLessResult> for ()
where
    Left: IsTreeMap,
    Right: IsTreeMap,
{
    type Output = TreeMap<NodeKey, V, Left, Right>;
}

impl<K, V, NodeKey, NodeValue, Left, Right>
    TreeMapInsertHelper<K, V, NodeKey, NodeValue, Left, Right, B0, B1> for ()
where
    Left: IsTreeMap + TreeMapInsert<K, V>,
    Right: IsTreeMap,
    <Left as TreeMapInsert<K, V>>::Output: IsTreeMap,
{
    type Output = TreeMap<NodeKey, NodeValue, <Left as TreeMapInsert<K, V>>::Output, Right>;
}

impl<K, V, NodeKey, NodeValue, Left, Right>
    TreeMapInsertHelper<K, V, NodeKey, NodeValue, Left, Right, B0, B0> for ()
where
    Left: IsTreeMap,
    Right: IsTreeMap + TreeMapInsert<K, V>,
    <Right as TreeMapInsert<K, V>>::Output: IsTreeMap,
{
    type Output = TreeMap<NodeKey, NodeValue, Left, <Right as TreeMapInsert<K, V>>::Output>;
}

impl<K, V, NodeKey, NodeValue, Left: IsTreeMap, Right: IsTreeMap> TreeMapInsert<K, V>
    for TreeMap<NodeKey, NodeValue, Left, Right>
where
    K: IsEqual<NodeKey> + IsLess<NodeKey>,
    <K as IsEqual<NodeKey>>::Output: Bit,
    <K as IsLess<NodeKey>>::Output: Bit,
    (): TreeMapInsertHelper<
            K,
            V,
            NodeKey,
            NodeValue,
            Left,
            Right,
            <K as IsEqual<NodeKey>>::Output,
            <K as IsLess<NodeKey>>::Output,
        >,
    <() as TreeMapInsertHelper<
        K,
        V,
        NodeKey,
        NodeValue,
        Left,
        Right,
        <K as IsEqual<NodeKey>>::Output,
        <K as IsLess<NodeKey>>::Output,
    >>::Output: IsTreeMap,
{
    type Output = <() as TreeMapInsertHelper<
        K,
        V,
        NodeKey,
        NodeValue,
        Left,
        Right,
        <K as IsEqual<NodeKey>>::Output,
        <K as IsLess<NodeKey>>::Output,
    >>::Output;
}

/// Get a value by key from TreeMap.
///
/// Returns Some<V> if found, None otherwise.
pub trait TreeMapGet<K> {
    type Output; // Some<V> or None
}

impl<K> TreeMapGet<K> for Nil {
    type Output = None;
}

#[doc(hidden)]
pub trait TreeMapGetHelper<K, NodeKey, NodeValue, Left, Right, IsEq, IsLessResult> {
    type Output;
}

impl<K, NodeKey, NodeValue, Left, Right, IsLessResult>
    TreeMapGetHelper<K, NodeKey, NodeValue, Left, Right, B1, IsLessResult> for ()
{
    type Output = Some<NodeValue>;
}

impl<K, NodeKey, NodeValue, Left, Right>
    TreeMapGetHelper<K, NodeKey, NodeValue, Left, Right, B0, B1> for ()
where
    Left: IsTreeMap + TreeMapGet<K>,
{
    type Output = <Left as TreeMapGet<K>>::Output;
}

impl<K, NodeKey, NodeValue, Left, Right>
    TreeMapGetHelper<K, NodeKey, NodeValue, Left, Right, B0, B0> for ()
where
    Right: IsTreeMap + TreeMapGet<K>,
{
    type Output = <Right as TreeMapGet<K>>::Output;
}

impl<K, NodeKey, NodeValue, Left: IsTreeMap, Right: IsTreeMap> TreeMapGet<K>
    for TreeMap<NodeKey, NodeValue, Left, Right>
where
    K: IsEqual<NodeKey> + IsLess<NodeKey>,
    <K as IsEqual<NodeKey>>::Output: Bit,
    <K as IsLess<NodeKey>>::Output: Bit,
    (): TreeMapGetHelper<
            K,
            NodeKey,
            NodeValue,
            Left,
            Right,
            <K as IsEqual<NodeKey>>::Output,
            <K as IsLess<NodeKey>>::Output,
        >,
{
    type Output = <() as TreeMapGetHelper<
        K,
        NodeKey,
        NodeValue,
        Left,
        Right,
        <K as IsEqual<NodeKey>>::Output,
        <K as IsLess<NodeKey>>::Output,
    >>::Output;
}

/// Check if TreeMap contains a key.
pub trait TreeMapContains<K> {
    type Output; // True or False
}

impl<K> TreeMapContains<K> for Nil {
    type Output = False;
}

#[doc(hidden)]
pub trait TreeMapContainsHelper<K, NodeKey, Left, Right, IsEq, IsLessResult> {
    type Output;
}

impl<K, NodeKey, Left, Right, IsLessResult>
    TreeMapContainsHelper<K, NodeKey, Left, Right, B1, IsLessResult> for ()
{
    type Output = True;
}

impl<K, NodeKey, Left, Right> TreeMapContainsHelper<K, NodeKey, Left, Right, B0, B1> for ()
where
    Left: IsTreeMap + TreeMapContains<K>,
{
    type Output = <Left as TreeMapContains<K>>::Output;
}

impl<K, NodeKey, Left, Right> TreeMapContainsHelper<K, NodeKey, Left, Right, B0, B0> for ()
where
    Right: IsTreeMap + TreeMapContains<K>,
{
    type Output = <Right as TreeMapContains<K>>::Output;
}

impl<K, NodeKey, NodeValue, Left: IsTreeMap, Right: IsTreeMap> TreeMapContains<K>
    for TreeMap<NodeKey, NodeValue, Left, Right>
where
    K: IsEqual<NodeKey> + IsLess<NodeKey>,
    <K as IsEqual<NodeKey>>::Output: Bit,
    <K as IsLess<NodeKey>>::Output: Bit,
    (): TreeMapContainsHelper<
            K,
            NodeKey,
            Left,
            Right,
            <K as IsEqual<NodeKey>>::Output,
            <K as IsLess<NodeKey>>::Output,
        >,
{
    type Output = <() as TreeMapContainsHelper<
        K,
        NodeKey,
        Left,
        Right,
        <K as IsEqual<NodeKey>>::Output,
        <K as IsLess<NodeKey>>::Output,
    >>::Output;
}

/// Get the minimum key-value pair from TreeMap (leftmost)
pub trait TreeMapMin {
    type Output; // Some<(K, V)> or None
}

impl TreeMapMin for Nil {
    type Output = None;
}

impl<K, V, R: IsTreeMap> TreeMapMin for TreeMap<K, V, Nil, R> {
    type Output = Some<(K, V)>;
}

impl<K, V, LK, LV, LL: IsTreeMap, LR: IsTreeMap, R: IsTreeMap> TreeMapMin
    for TreeMap<K, V, TreeMap<LK, LV, LL, LR>, R>
where
    TreeMap<LK, LV, LL, LR>: TreeMapMin,
{
    type Output = <TreeMap<LK, LV, LL, LR> as TreeMapMin>::Output;
}

/// Get the maximum key-value pair from TreeMap (rightmost)
pub trait TreeMapMax {
    type Output; // Some<(K, V)> or None
}

impl TreeMapMax for Nil {
    type Output = None;
}

impl<K, V, L: IsTreeMap> TreeMapMax for TreeMap<K, V, L, Nil> {
    type Output = Some<(K, V)>;
}

impl<K, V, L: IsTreeMap, RK, RV, RL: IsTreeMap, RR: IsTreeMap> TreeMapMax
    for TreeMap<K, V, L, TreeMap<RK, RV, RL, RR>>
where
    TreeMap<RK, RV, RL, RR>: TreeMapMax,
{
    type Output = <TreeMap<RK, RV, RL, RR> as TreeMapMax>::Output;
}

/// Convert TreeMap to sorted list of key-value pairs (in-order traversal)
pub trait TreeMapToList {
    type Output: crate::model::col::array::IsList;
}

impl TreeMapToList for Nil {
    type Output = crate::model::col::array::Nil;
}

impl<K, V, L: IsTreeMap + TreeMapToList, R: IsTreeMap + TreeMapToList> TreeMapToList
    for TreeMap<K, V, L, R>
where
    <L as TreeMapToList>::Output: crate::model::col::array::IsList
        + crate::std::col::array::Concat<
            crate::model::col::array::Array<(K, V), <R as TreeMapToList>::Output>,
        >,
    <R as TreeMapToList>::Output: crate::model::col::array::IsList,
{
    type Output = <<L as TreeMapToList>::Output as crate::std::col::array::Concat<
        crate::model::col::array::Array<(K, V), <R as TreeMapToList>::Output>,
    >>::Output;
}

impl ToArray for Nil {
    type Output = crate::model::col::array::Nil;
}

impl<K, V, L: IsTreeMap + TreeMapToList, R: IsTreeMap + TreeMapToList> ToArray
    for TreeMap<K, V, L, R>
where
    TreeMap<K, V, L, R>: TreeMapToList,
{
    type Output = <TreeMap<K, V, L, R> as TreeMapToList>::Output;
}

impl<Op, Init> Foldable<Op, Init> for Nil
where
    crate::model::col::array::Nil: crate::std::col::array::Foldable<Op, Init>,
{
    type Output =
        <crate::model::col::array::Nil as crate::std::col::array::Foldable<Op, Init>>::Output;
}

impl<K, V, L, R, Op, Init> Foldable<Op, Init> for TreeMap<K, V, L, R>
where
    TreeMap<K, V, L, R>: ToArray,
    <TreeMap<K, V, L, R> as ToArray>::Output: crate::std::col::array::Foldable<Op, Init>,
{
    type Output = <<TreeMap<K, V, L, R> as ToArray>::Output as crate::std::col::array::Foldable<
        Op,
        Init,
    >>::Output;
}

crate::typelude_macros::ty_fn! {
    /// Insert a key-value pair into a TreeMap.
    pub struct FTreeMapInsert<Tree>
    {
        type Output = FTreeMapInsertCaptured1<Tree>;
    }
}

crate::typelude_macros::ty_fn! {
    /// Insert a key-value pair into a TreeMap.
    pub struct FTreeMapInsertCaptured1<Tree, Key>
    {
        type Output = FTreeMapInsertCaptured2<Tree, Key>;
    }
}

crate::typelude_macros::ty_fn! {
    /// Insert a key-value pair into a TreeMap.
    pub struct FTreeMapInsertCaptured2<Tree, Key, Value>
    where [Tree: TreeMapInsert<Key, Value>]
    {
        type Output = <Tree as TreeMapInsert<Key, Value>>::Output;
    }
}

crate::typelude_macros::ty_fn! {
    /// Get a value by key from a TreeMap.
    pub struct FTreeMapGet<Tree>
    {
        type Output = FTreeMapGetCaptured<Tree>;
    }
}

crate::typelude_macros::ty_fn! {
    /// Get a value by key from a TreeMap.
    pub struct FTreeMapGetCaptured<Tree, Key>
    where [Tree: TreeMapGet<Key>]
    {
        type Output = <Tree as TreeMapGet<Key>>::Output;
    }
}

crate::typelude_macros::ty_fn! {
    /// Check whether a TreeMap contains a key.
    pub struct FTreeMapContains<Tree>
    {
        type Output = FTreeMapContainsCaptured<Tree>;
    }
}

crate::typelude_macros::ty_fn! {
    /// Check whether a TreeMap contains a key.
    pub struct FTreeMapContainsCaptured<Tree, Key>
    where [Tree: TreeMapContains<Key>]
    {
        type Output = <Tree as TreeMapContains<Key>>::Output;
    }
}

crate::typelude_macros::ty_fn! {
    /// Get the minimum key-value pair of a TreeMap.
    pub struct FTreeMapMin<Tree>
    where [Tree: TreeMapMin]
    {
        type Output = <Tree as TreeMapMin>::Output;
    }
}

crate::typelude_macros::ty_fn! {
    /// Get the maximum key-value pair of a TreeMap.
    pub struct FTreeMapMax<Tree>
    where [Tree: TreeMapMax]
    {
        type Output = <Tree as TreeMapMax>::Output;
    }
}

crate::typelude_macros::ty_fn! {
    /// Convert a TreeMap into a sorted array of pairs.
    pub struct FTreeMapToList<Tree>
    where [Tree: TreeMapToList]
    {
        type Output = <Tree as TreeMapToList>::Output;
    }
}

/// Insert key-value pair into TreeMap.
pub type ETreeMapInsert<Tree, Key, Value> = crate::core::ECall3<FTreeMapInsert, Tree, Key, Value>;
/// Get value by key from TreeMap.
pub type ETreeMapGet<Tree, Key> = crate::core::ECall2<FTreeMapGet, Tree, Key>;
/// Check if TreeMap contains key.
pub type ETreeMapContains<Tree, Key> = crate::core::ECall2<FTreeMapContains, Tree, Key>;
/// Get the minimum key-value pair in a TreeMap.
pub type ETreeMapMin<Tree> = crate::core::ECall<FTreeMapMin, Tree>;
/// Get the maximum key-value pair in a TreeMap.
pub type ETreeMapMax<Tree> = crate::core::ECall<FTreeMapMax, Tree>;
/// Convert a TreeMap into a sorted array of pairs.
pub type ETreeMapToList<Tree> = crate::core::ECall<FTreeMapToList, Tree>;

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typenum::{U0, U1, U2, U3, U5};

    use super::*;
    use crate::model::col::array::{Array as ArrayData, Nil as NilArray};

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
    fn test_insert() {
        type T1 = <Nil as TreeMapInsert<U2, i32>>::Output;
        assert_type_eq_all!(T1, TreeMap<U2, i32, Nil, Nil>);
    }

    #[test]
    fn test_get() {
        type Tree1 = TreeMap<U2, i32, Nil, Nil>;
        type Res1 = <Tree1 as TreeMapGet<U2>>::Output;
        assert_type_eq_all!(Res1, Some<i32>);

        type Res2 = <Tree1 as TreeMapGet<U3>>::Output;
        assert_type_eq_all!(Res2, None);

        type Tree2 = TreeMap<U3, char, TreeMap<U1, i32, Nil, Nil>, TreeMap<U5, f64, Nil, Nil>>;
        type ResRoot = <Tree2 as TreeMapGet<U3>>::Output;
        assert_type_eq_all!(ResRoot, Some<char>);

        type ResLeft = <Tree2 as TreeMapGet<U1>>::Output;
        assert_type_eq_all!(ResLeft, Some<i32>);

        type ResRight = <Tree2 as TreeMapGet<U5>>::Output;
        assert_type_eq_all!(ResRight, Some<f64>);
    }

    #[test]
    fn test_contains() {
        type Tree1 = TreeMap<U3, char, TreeMap<U1, i32, Nil, Nil>, TreeMap<U5, f64, Nil, Nil>>;

        type Has1 = <Tree1 as TreeMapContains<U1>>::Output;
        assert_type_eq_all!(Has1, True);

        type Has2 = <Tree1 as TreeMapContains<U2>>::Output;
        assert_type_eq_all!(Has2, False);
    }

    #[test]
    fn test_min_max() {
        type Tree1 = TreeMap<U3, i32, Nil, Nil>;
        assert_type_eq_all!(<Tree1 as TreeMapMin>::Output, Some<(U3, i32)>);
        assert_type_eq_all!(<Tree1 as TreeMapMax>::Output, Some<(U3, i32)>);

        type Tree2 = TreeMap<U3, char, TreeMap<U1, i32, Nil, Nil>, Nil>;
        assert_type_eq_all!(<Tree2 as TreeMapMin>::Output, Some<(U1, i32)>);
        assert_type_eq_all!(<Tree2 as TreeMapMax>::Output, Some<(U3, char)>);
    }

    #[test]
    fn test_to_list() {
        type Tree1 = TreeMap<U3, i32, Nil, Nil>;
        type List1 = <Tree1 as TreeMapToList>::Output;
        assert_type_eq_all!(List1, ArrayData<(U3, i32), NilArray>);

        type Tree2 = TreeMap<U2, char, TreeMap<U1, i32, Nil, Nil>, Nil>;
        type List2 = <Tree2 as TreeMapToList>::Output;
        assert_type_eq_all!(List2, ArrayData<(U1, i32), ArrayData<(U2, char), NilArray>>);

        type AsArray = <Tree2 as ToArray>::Output;
        assert_type_eq_all!(AsArray, ArrayData<(U1, i32), ArrayData<(U2, char), NilArray>>);
        assert_type_eq_all!(<Tree2 as Foldable<CountEntries, U0>>::Output, U2);
    }

    #[test]
    fn test_insert_update() {
        type T1 = <Nil as TreeMapInsert<U2, i32>>::Output;
        type T2 = <T1 as TreeMapInsert<U2, f64>>::Output;

        type Res = <T2 as TreeMapGet<U2>>::Output;
        assert_type_eq_all!(Res, Some<f64>);
    }

    #[test]
    fn test_empty() {
        type Res = <Nil as TreeMapGet<U1>>::Output;
        assert_type_eq_all!(Res, None);

        type Has = <Nil as TreeMapContains<U1>>::Output;
        assert_type_eq_all!(Has, False);

        type Min = <Nil as TreeMapMin>::Output;
        assert_type_eq_all!(Min, None);

        type Max = <Nil as TreeMapMax>::Output;
        assert_type_eq_all!(Max, None);
    }
}
