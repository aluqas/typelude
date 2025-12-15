//! **Instruction Set**
//!
//! Defines the instruction types for the stack machine.

use core::marker::PhantomData;
use crate::std::trace::Trace;

// Marker trait for instructions could be added here if needed in future.

/// Pushes a value N onto the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpPush<N>(PhantomData<N>);

impl<N: Trace> Trace for OpPush<N> {
    fn fmt() -> String {
        format!("Push({})", N::fmt())
    }
}

/// Adds the top two values on the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpAdd;

impl Trace for OpAdd { fn fmt() -> String { "Add".to_string() } }

/// Subtracts the top value from the second top value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpSub;

impl Trace for OpSub { fn fmt() -> String { "Sub".to_string() } }

/// Duplicates the top value of the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpDup;

impl Trace for OpDup { fn fmt() -> String { "Dup".to_string() } }

/// Swaps the top two values of the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpSwap;

impl Trace for OpSwap { fn fmt() -> String { "Swap".to_string() } }

/// Drops the top value of the stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpDrop;

impl Trace for OpDrop { fn fmt() -> String { "Drop".to_string() } }

/// Conditional execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpIf<ThenProg, ElseProg>(PhantomData<(ThenProg, ElseProg)>);

impl<T: Trace, E: Trace> Trace for OpIf<T, E> {
    fn fmt() -> String {
        // Just show basic info to avoid massive output
        "If(...)".to_string()
    }
}

/// Loads a value from global memory at address specified on stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpLoad;

impl Trace for OpLoad { fn fmt() -> String { "Load".to_string() } }

/// Stores a value to global memory at address specified on stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpStore;

impl Trace for OpStore { fn fmt() -> String { "Store".to_string() } }

/// Calls a subroutine (Program type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpCall<TargetProg>(PhantomData<TargetProg>);

impl<T> Trace for OpCall<T> { fn fmt() -> String { "Call".to_string() } }

/// Returns from a subroutine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpReturn;

impl Trace for OpReturn { fn fmt() -> String { "Return".to_string() } }

/// Checks equality of top two values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpEq;

impl Trace for OpEq { fn fmt() -> String { "Eq".to_string() } }

/// Checks inequality of top two values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpNeq;

impl Trace for OpNeq { fn fmt() -> String { "Neq".to_string() } }

/// Checks if second top value is less than top value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpLt;

impl Trace for OpLt { fn fmt() -> String { "Lt".to_string() } }

/// Checks if second top value is greater than top value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpGt;

impl Trace for OpGt { fn fmt() -> String { "Gt".to_string() } }

/// Logical NOT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpNot;

impl Trace for OpNot { fn fmt() -> String { "Not".to_string() } }

/// Logical AND.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpAnd;

impl Trace for OpAnd { fn fmt() -> String { "And".to_string() } }

/// Logical OR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpOr;

impl Trace for OpOr { fn fmt() -> String { "Or".to_string() } }

/// While loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpWhile<CondProg, BodyProg>(PhantomData<(CondProg, BodyProg)>);

impl<C, B> Trace for OpWhile<C, B> { fn fmt() -> String { "While(...)".to_string() } }

/// Gets a local variable by De Bruijn index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpGetLocal<Index>(PhantomData<Index>);

impl<I: Trace> Trace for OpGetLocal<I> {
    fn fmt() -> String {
        format!("GetLocal({})", I::fmt())
    }
}

/// Sets a local variable by De Bruijn index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpSetLocal<Index>(PhantomData<Index>);

impl<I: Trace> Trace for OpSetLocal<I> {
    fn fmt() -> String {
        format!("SetLocal({})", I::fmt())
    }
}

/// Defines a new local variable (moves top of stack to locals).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpLet;

impl Trace for OpLet { fn fmt() -> String { "Let".to_string() } }

/// Drops the most recent local variable (end of scope).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpDropLocal;

impl Trace for OpDropLocal { fn fmt() -> String { "DropLocal".to_string() } }
