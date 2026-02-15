//! **Type-Level TreeArray**
//!
//! Value-only binary search tree (sorted set).

use typenum::{B0, B1, Bit, IsEqual, IsLess};

/// Marker trait for type-level TreeArray
pub use crate::model::col::tree_array::IsTreeArray;
// Re-export kernel types
pub use crate::model::col::tree_array::{Nil, TreeArray};
use crate::{
    model::prim::bool::{False, True},
    std::prim::option::{None, Some},
};

// ============================================================================
// TreeArray Operations (Value-only)
// ============================================================================

/// Insert a value into TreeArray.
///
/// Uses typenum comparison: values that compare less go left, greater go right.
pub trait TreeArrayInsert<V> {
    type Output: IsTreeArray;
}

impl<V> TreeArrayInsert<V> for Nil {
    type Output = TreeArray<V, Nil, Nil>;
}

#[doc(hidden)]
pub trait TreeArrayInsertHelper<V, NodeValue, Left, Right, IsEq, IsLessResult> {
    type Output;
}

impl<V, NodeValue, Left, Right, IsLessResult>
    TreeArrayInsertHelper<V, NodeValue, Left, Right, B1, IsLessResult> for ()
where
    Left: IsTreeArray,
    Right: IsTreeArray,
{
    type Output = TreeArray<NodeValue, Left, Right>;
}

impl<V, NodeValue, Left, Right> TreeArrayInsertHelper<V, NodeValue, Left, Right, B0, B1> for ()
where
    Left: IsTreeArray + TreeArrayInsert<V>,
    Right: IsTreeArray,
    <Left as TreeArrayInsert<V>>::Output: IsTreeArray,
{
    type Output = TreeArray<NodeValue, <Left as TreeArrayInsert<V>>::Output, Right>;
}

impl<V, NodeValue, Left, Right> TreeArrayInsertHelper<V, NodeValue, Left, Right, B0, B0> for ()
where
    Left: IsTreeArray,
    Right: IsTreeArray + TreeArrayInsert<V>,
    <Right as TreeArrayInsert<V>>::Output: IsTreeArray,
{
    type Output = TreeArray<NodeValue, Left, <Right as TreeArrayInsert<V>>::Output>;
}

impl<V, NodeValue, Left: IsTreeArray, Right: IsTreeArray> TreeArrayInsert<V>
    for TreeArray<NodeValue, Left, Right>
where
    V: IsEqual<NodeValue> + IsLess<NodeValue>,
    <V as IsEqual<NodeValue>>::Output: Bit,
    <V as IsLess<NodeValue>>::Output: Bit,
    (): TreeArrayInsertHelper<
            V,
            NodeValue,
            Left,
            Right,
            <V as IsEqual<NodeValue>>::Output,
            <V as IsLess<NodeValue>>::Output,
        >,
    <() as TreeArrayInsertHelper<
            V,
            NodeValue,
            Left,
            Right,
            <V as IsEqual<NodeValue>>::Output,
            <V as IsLess<NodeValue>>::Output,
        >>::Output: IsTreeArray,
{
    type Output = <() as TreeArrayInsertHelper<
        V,
        NodeValue,
        Left,
        Right,
        <V as IsEqual<NodeValue>>::Output,
        <V as IsLess<NodeValue>>::Output,
    >>::Output;
}

/// Check if TreeArray contains a value.
pub trait TreeArrayContains<V> {
    type Output; // True or False
}

impl<V> TreeArrayContains<V> for Nil {
    type Output = False;
}

#[doc(hidden)]
pub trait TreeArrayContainsHelper<V, NodeValue, Left, Right, IsEq, IsLessResult> {
    type Output;
}

impl<V, NodeValue, Left, Right, IsLessResult>
    TreeArrayContainsHelper<V, NodeValue, Left, Right, B1, IsLessResult> for ()
{
    type Output = True;
}

impl<V, NodeValue, Left, Right>
    TreeArrayContainsHelper<V, NodeValue, Left, Right, B0, B1> for ()
where
    Left: IsTreeArray + TreeArrayContains<V>,
{
    type Output = <Left as TreeArrayContains<V>>::Output;
}

impl<V, NodeValue, Left, Right>
    TreeArrayContainsHelper<V, NodeValue, Left, Right, B0, B0> for ()
where
    Right: IsTreeArray + TreeArrayContains<V>,
{
    type Output = <Right as TreeArrayContains<V>>::Output;
}

impl<V, NodeValue, Left: IsTreeArray, Right: IsTreeArray> TreeArrayContains<V>
    for TreeArray<NodeValue, Left, Right>
where
    V: IsEqual<NodeValue> + IsLess<NodeValue>,
    <V as IsEqual<NodeValue>>::Output: Bit,
    <V as IsLess<NodeValue>>::Output: Bit,
    (): TreeArrayContainsHelper<
            V,
            NodeValue,
            Left,
            Right,
            <V as IsEqual<NodeValue>>::Output,
            <V as IsLess<NodeValue>>::Output,
        >,
{
    type Output = <() as TreeArrayContainsHelper<
        V,
        NodeValue,
        Left,
        Right,
        <V as IsEqual<NodeValue>>::Output,
        <V as IsLess<NodeValue>>::Output,
    >>::Output;
}

/// Get the minimum value from TreeArray (leftmost)
pub trait TreeArrayMin {
    type Output; // Some<V> or None
}

impl TreeArrayMin for Nil {
    type Output = None;
}

impl<V, R: IsTreeArray> TreeArrayMin for TreeArray<V, Nil, R> {
    type Output = Some<V>;
}

impl<V, LV, LL: IsTreeArray, LR: IsTreeArray, R: IsTreeArray> TreeArrayMin
    for TreeArray<V, TreeArray<LV, LL, LR>, R>
where
    TreeArray<LV, LL, LR>: TreeArrayMin,
{
    type Output = <TreeArray<LV, LL, LR> as TreeArrayMin>::Output;
}

/// Get the maximum value from TreeArray (rightmost)
pub trait TreeArrayMax {
    type Output; // Some<V> or None
}

impl TreeArrayMax for Nil {
    type Output = None;
}

impl<V, L: IsTreeArray> TreeArrayMax for TreeArray<V, L, Nil> {
    type Output = Some<V>;
}

impl<V, L: IsTreeArray, RV, RL: IsTreeArray, RR: IsTreeArray> TreeArrayMax
    for TreeArray<V, L, TreeArray<RV, RL, RR>>
where
    TreeArray<RV, RL, RR>: TreeArrayMax,
{
    type Output = <TreeArray<RV, RL, RR> as TreeArrayMax>::Output;
}

/// Convert TreeArray to sorted list (in-order traversal)
pub trait TreeArrayToList {
    type Output: crate::model::col::array::IsList;
}

impl TreeArrayToList for Nil {
    type Output = crate::model::col::array::Nil;
}

impl<V, L: IsTreeArray + TreeArrayToList, R: IsTreeArray + TreeArrayToList> TreeArrayToList
    for TreeArray<V, L, R>
where
    <L as TreeArrayToList>::Output: crate::model::col::array::IsList
        + crate::std::col::array::Concat<
            crate::model::col::array::Array<V, <R as TreeArrayToList>::Output>,
        >,
    <R as TreeArrayToList>::Output: crate::model::col::array::IsList,
{
    type Output = <<L as TreeArrayToList>::Output as crate::std::col::array::Concat<
        crate::model::col::array::Array<V, <R as TreeArrayToList>::Output>,
    >>::Output;
}

crate::def_expr_via_trait!(
    /// Insert value into TreeArray
    pub ETreeArrayInsert<Tree, Value>
    args [Tree, Value]
    where [~Tree: TreeArrayInsert<~Value>]
    => <~Tree as TreeArrayInsert<~Value>>::Output
);

crate::def_expr_via_trait!(
    /// Check if TreeArray contains value
    pub ETreeArrayContains<Tree, Value>
    args [Tree, Value]
    where [~Tree: TreeArrayContains<~Value>]
    => <~Tree as TreeArrayContains<~Value>>::Output
);

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typenum::{U1, U2, U3, U5};

    use super::*;
    use crate::model::col::array::{Array as ArrayData, Nil as NilArray};

    #[test]
    fn test_insert() {
        type T1 = <Nil as TreeArrayInsert<U2>>::Output;
        assert_type_eq_all!(T1, TreeArray<U2, Nil, Nil>);
    }

    #[test]
    fn test_contains() {
        type Tree1 = TreeArray<U3, TreeArray<U1, Nil, Nil>, TreeArray<U5, Nil, Nil>>;

        type Has1 = <Tree1 as TreeArrayContains<U1>>::Output;
        assert_type_eq_all!(Has1, True);

        type Has3 = <Tree1 as TreeArrayContains<U3>>::Output;
        assert_type_eq_all!(Has3, True);

        type Has2 = <Tree1 as TreeArrayContains<U2>>::Output;
        assert_type_eq_all!(Has2, False);
    }

    #[test]
    fn test_min_max() {
        type Tree1 = TreeArray<U3, Nil, Nil>;
        assert_type_eq_all!(<Tree1 as TreeArrayMin>::Output, Some<U3>);
        assert_type_eq_all!(<Tree1 as TreeArrayMax>::Output, Some<U3>);

        type Tree2 = TreeArray<U3, TreeArray<U1, Nil, Nil>, Nil>;
        assert_type_eq_all!(<Tree2 as TreeArrayMin>::Output, Some<U1>);
        assert_type_eq_all!(<Tree2 as TreeArrayMax>::Output, Some<U3>);

        type Tree3 = TreeArray<U3, Nil, TreeArray<U5, Nil, Nil>>;
        assert_type_eq_all!(<Tree3 as TreeArrayMin>::Output, Some<U3>);
        assert_type_eq_all!(<Tree3 as TreeArrayMax>::Output, Some<U5>);
    }

    #[test]
    fn test_to_list() {
        type Tree1 = TreeArray<U3, Nil, Nil>;
        type List1 = <Tree1 as TreeArrayToList>::Output;
        assert_type_eq_all!(List1, ArrayData<U3, NilArray>);

        type Tree2 = TreeArray<U2, TreeArray<U1, Nil, Nil>, Nil>;
        type List2 = <Tree2 as TreeArrayToList>::Output;
        assert_type_eq_all!(List2, ArrayData<U1, ArrayData<U2, NilArray>>);
    }

    #[test]
    fn test_empty() {
        type Has = <Nil as TreeArrayContains<U1>>::Output;
        assert_type_eq_all!(Has, False);

        type Min = <Nil as TreeArrayMin>::Output;
        assert_type_eq_all!(Min, None);

        type Max = <Nil as TreeArrayMax>::Output;
        assert_type_eq_all!(Max, None);
    }
}
