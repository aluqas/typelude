use std::marker::PhantomData;

use crate::Eval;

/// **Expression Cons List**: A type-level list of expressions.
/// Used to pass variable numbers of arguments to operators without using Tuples.
pub trait EList: Eval {}

/// **Nil for Expression List**
pub struct ENil;

impl Eval for ENil {
    type Output = (); // Or a specific Nil type if needed, but () is standard unit.
}
impl EList for ENil {}

/// **Cons for Expression List**
pub struct ECons<Head, Tail>(PhantomData<(Head, Tail)>);

impl<Head, Tail> Eval for ECons<Head, Tail>
where
    Head: Eval,
    Tail: Eval,
{
    // Evaluate to a tuple-list (H, T) or just a value list?
    // For now, let's make it evaluate to a nested tuple (H, T::Output) to maintain compatibility with some structures,
    // or arguably it should evaluate to a Value List.
    // Given the refactor goal is "reduction", maybe we want the output to be a standard Cons List Value?
    // Let's assume (H::Output, T::Output) for now as a generic container.
    type Output = (Head::Output, Tail::Output);
}

impl<Head, Tail> EList for ECons<Head, Tail>
where
    Head: Eval,
    Tail: EList,
{
}
