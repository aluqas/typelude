//! **Type-Level Expression Lists (EList)**
//!
//! Provides `ECons` and `ENil` for passing variadic arguments to type-level
//! operators.

use std::marker::PhantomData;

use crate::Eval;

/// **Expression Cons List**: A type-level list of expressions.
///
/// Used to pass variable numbers of arguments to operators without using
/// Tuples.
///
/// # Examples
///
/// ```ignore
/// type Args = ECons<ELit<U1>, ECons<ELit<U2>, ENil>>;
/// ```
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
    type Output = (Head::Output, Tail::Output);
}

impl<Head, Tail> EList for ECons<Head, Tail>
where
    Head: Eval,
    Tail: EList,
{
}
