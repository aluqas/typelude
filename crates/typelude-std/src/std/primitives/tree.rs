//! **Type-Level Binary Search Tree**
//!
//! Type-level binary search tree (BST) with insert, contains, min, max, and traversal operations.
//! This is an unbalanced BST; for truly balanced trees, AVL rotations would be needed.

use std::marker::PhantomData;

use typelude_core::Evaluate;
use typelude_macros::def_op;
use typenum::{B0, B1, Bit, IsEqual, IsLess};

use crate::{
    data::primitives::bool::{TyFalse, TyTrue},
    std::primitives::{
        array::{Concat, Cons, TyArray, TyNil},
        option::{TyNone, TySome},
    },
};

// =============================================================================
// Data Structure
// =============================================================================

/// Empty tree (leaf)
pub struct TyLeaf;

/// Non-empty tree node with value, left subtree, and right subtree
pub struct TyNode<Value, Left, Right>(PhantomData<(Value, Left, Right)>);

/// Marker trait for type-level trees
pub trait TypeTree {}

impl TypeTree for TyLeaf {}
impl<V, L: TypeTree, R: TypeTree> TypeTree for TyNode<V, L, R> {}

// =============================================================================
// Core Operations
// =============================================================================

/// Insert a value into the tree
/// Uses typenum comparison: values that compare less go left, greater go right
pub trait TreeInsert<V> {
    type Output: TypeTree;
}

// Insert into empty tree
impl<V> TreeInsert<V> for TyLeaf {
    type Output = TyNode<V, TyLeaf, TyLeaf>;
}

/// Helper for insert dispatch based on comparison
#[doc(hidden)]
pub trait TreeInsertHelper<V, Value, Left, Right, CmpResult> {
    type Output: TypeTree;
}

// V < Value: go left
impl<V, Value, Left: TypeTree + TreeInsert<V>, Right: TypeTree>
    TreeInsertHelper<V, Value, Left, Right, B1> for ()
where
    <Left as TreeInsert<V>>::Output: TypeTree,
{
    type Output = TyNode<Value, <Left as TreeInsert<V>>::Output, Right>;
}

// V >= Value: go right
impl<V, Value, Left: TypeTree, Right: TypeTree + TreeInsert<V>>
    TreeInsertHelper<V, Value, Left, Right, B0> for ()
where
    <Right as TreeInsert<V>>::Output: TypeTree,
{
    type Output = TyNode<Value, Left, <Right as TreeInsert<V>>::Output>;
}

impl<V, Value, Left: TypeTree, Right: TypeTree> TreeInsert<V> for TyNode<Value, Left, Right>
where
    V: IsLess<Value>,
    <V as IsLess<Value>>::Output: Bit,
    (): TreeInsertHelper<V, Value, Left, Right, <V as IsLess<Value>>::Output>,
{
    type Output =
        <() as TreeInsertHelper<V, Value, Left, Right, <V as IsLess<Value>>::Output>>::Output;
}

/// Check if the tree contains a value
pub trait TreeContains<V> {
    type Output; // TyTrue or TyFalse
}

impl<V> TreeContains<V> for TyLeaf {
    type Output = TyFalse;
}

/// Helper for contains dispatch
#[doc(hidden)]
pub trait TreeContainsHelper<V, Value, Left, Right, IsEq, IsLessResult> {
    type Output;
}

// V == Value: found
impl<V, Value, Left: TypeTree, Right: TypeTree, IsLessResult>
    TreeContainsHelper<V, Value, Left, Right, B1, IsLessResult> for ()
{
    type Output = TyTrue;
}

// V < Value: search left
impl<V, Value, Left: TypeTree + TreeContains<V>, Right: TypeTree>
    TreeContainsHelper<V, Value, Left, Right, B0, B1> for ()
{
    type Output = <Left as TreeContains<V>>::Output;
}

// V > Value: search right
impl<V, Value, Left: TypeTree, Right: TypeTree + TreeContains<V>>
    TreeContainsHelper<V, Value, Left, Right, B0, B0> for ()
{
    type Output = <Right as TreeContains<V>>::Output;
}

impl<V, Value, Left: TypeTree, Right: TypeTree> TreeContains<V> for TyNode<Value, Left, Right>
where
    V: IsEqual<Value> + IsLess<Value>,
    <V as IsEqual<Value>>::Output: Bit,
    <V as IsLess<Value>>::Output: Bit,
    (): TreeContainsHelper<
            V,
            Value,
            Left,
            Right,
            <V as IsEqual<Value>>::Output,
            <V as IsLess<Value>>::Output,
        >,
{
    type Output = <() as TreeContainsHelper<
        V,
        Value,
        Left,
        Right,
        <V as IsEqual<Value>>::Output,
        <V as IsLess<Value>>::Output,
    >>::Output;
}

/// Get the minimum value (leftmost)
pub trait TreeMin {
    type Output; // TySome<V> or TyNone
}

impl TreeMin for TyLeaf {
    type Output = TyNone;
}

impl<V, R: TypeTree> TreeMin for TyNode<V, TyLeaf, R> {
    type Output = TySome<V>;
}

impl<V, LV, LL: TypeTree, LR: TypeTree, R: TypeTree> TreeMin for TyNode<V, TyNode<LV, LL, LR>, R>
where
    TyNode<LV, LL, LR>: TreeMin,
{
    type Output = <TyNode<LV, LL, LR> as TreeMin>::Output;
}

/// Get the maximum value (rightmost)
pub trait TreeMax {
    type Output; // TySome<V> or TyNone
}

impl TreeMax for TyLeaf {
    type Output = TyNone;
}

impl<V, L: TypeTree> TreeMax for TyNode<V, L, TyLeaf> {
    type Output = TySome<V>;
}

impl<V, L: TypeTree, RV, RL: TypeTree, RR: TypeTree> TreeMax for TyNode<V, L, TyNode<RV, RL, RR>>
where
    TyNode<RV, RL, RR>: TreeMax,
{
    type Output = <TyNode<RV, RL, RR> as TreeMax>::Output;
}

/// Convert tree to sorted list (in-order traversal)
pub trait TreeToList {
    type Output: Cons;
}

impl TreeToList for TyLeaf {
    type Output = TyNil;
}

impl<V, L: TypeTree + TreeToList, R: TypeTree + TreeToList> TreeToList for TyNode<V, L, R>
where
    <L as TreeToList>::Output: Cons + Concat<TyArray<V, <R as TreeToList>::Output>>,
    <R as TreeToList>::Output: Cons,
{
    // In-order: left ++ [value] ++ right
    type Output =
        <<L as TreeToList>::Output as Concat<TyArray<V, <R as TreeToList>::Output>>>::Output;
}

/// Get the height/depth of the tree
pub trait TreeHeight {
    type Output;
}

impl TreeHeight for TyLeaf {
    type Output = typenum::U0;
}

impl<V, L: TypeTree + TreeHeight, R: TypeTree + TreeHeight> TreeHeight for TyNode<V, L, R>
where
    <L as TreeHeight>::Output: typenum::Max<<R as TreeHeight>::Output>,
    <<L as TreeHeight>::Output as typenum::Max<<R as TreeHeight>::Output>>::Output:
        std::ops::Add<typenum::B1>,
{
    type Output = <<<L as TreeHeight>::Output as typenum::Max<<R as TreeHeight>::Output>>::Output as std::ops::Add<typenum::B1>>::Output;
}

// =============================================================================
// Expression Wrappers via def_op!
// =============================================================================

def_op! {
    /// Insert value into tree
    name: OpTreeInsert,
    args: (Tree, Value),
    ast: ETreeInsert {
        where: [
            Evaluate<Tree>: TreeInsert<Evaluate<Value>>
        ],
        type Output = <Evaluate<Tree> as TreeInsert<Evaluate<Value>>>::Output
    }
}

def_op! {
    /// Get minimum value from tree
    name: OpTreeMin,
    args: (Tree),
    ast: ETreeMin {
        where: [
            Evaluate<Tree>: TreeMin
        ],
        type Output = <Evaluate<Tree> as TreeMin>::Output
    }
}

def_op! {
    /// Get maximum value from tree
    name: OpTreeMax,
    args: (Tree),
    ast: ETreeMax {
        where: [
            Evaluate<Tree>: TreeMax
        ],
        type Output = <Evaluate<Tree> as TreeMax>::Output
    }
}

def_op! {
    /// Convert tree to sorted list
    name: OpTreeToList,
    args: (Tree),
    ast: ETreeToList {
        where: [
            Evaluate<Tree>: TreeToList
        ],
        type Output = <Evaluate<Tree> as TreeToList>::Output
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use static_assertions::assert_type_eq_all;
    use typenum::{U1, U2, U3, U4, U5};

    use super::*;

    #[test]
    fn test_tree_insert() {
        // Insert into empty tree
        type T1 = <TyLeaf as TreeInsert<U2>>::Output;
        assert_type_eq_all!(T1, TyNode<U2, TyLeaf, TyLeaf>);
    }

    #[test]
    fn test_tree_min_max() {
        // Single node tree
        type Tree1 = TyNode<U3, TyLeaf, TyLeaf>;
        assert_type_eq_all!(<Tree1 as TreeMin>::Output, TySome<U3>);
        assert_type_eq_all!(<Tree1 as TreeMax>::Output, TySome<U3>);

        // Tree with left child (min is in left)
        type Tree2 = TyNode<U3, TyNode<U1, TyLeaf, TyLeaf>, TyLeaf>;
        assert_type_eq_all!(<Tree2 as TreeMin>::Output, TySome<U1>);
        assert_type_eq_all!(<Tree2 as TreeMax>::Output, TySome<U3>);

        // Tree with right child (max is in right)
        type Tree3 = TyNode<U3, TyLeaf, TyNode<U5, TyLeaf, TyLeaf>>;
        assert_type_eq_all!(<Tree3 as TreeMin>::Output, TySome<U3>);
        assert_type_eq_all!(<Tree3 as TreeMax>::Output, TySome<U5>);
    }

    #[test]
    fn test_tree_to_list() {
        // Single node
        type Tree1 = TyNode<U3, TyLeaf, TyLeaf>;
        type List1 = <Tree1 as TreeToList>::Output;
        assert_type_eq_all!(List1, TyArray<U3, TyNil>);

        // Tree: 2 with left=1
        type Tree2 = TyNode<U2, TyNode<U1, TyLeaf, TyLeaf>, TyLeaf>;
        type List2 = <Tree2 as TreeToList>::Output;
        assert_type_eq_all!(List2, TyArray<U1, TyArray<U2, TyNil>>);
    }
}
