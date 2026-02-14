//! **Compiler / Interpreter**
//!
//! Maps Syntax (Instructions) to Semantics (Actions).

use typelude_std::{
    lambda::monads::state::LReturn,
    std::col::array::{Array, Nil},
};

use crate::machine::{
    instruction::{OpAdd, OpIf, OpPop, OpPush, OpWhile},
    semantics::{
        actions::{ActBinOp, ActPop, ActPush},
        control::{ActBranch, ActSeq, ActWhile},
        monad::Unit,
    },
};

// =============================================================================
// ToAction Trait
// =============================================================================

/// Convert a single Instruction to an Action.
pub trait ToAction {
    type Output;
}

// --- Push ---
impl<Val> ToAction for OpPush<Val> {
    type Output = ActPush<Val>;
}

// --- Pop ---
impl ToAction for OpPop {
    type Output = ActPop;
}

// --- Add ---
// Add is composed? Or primitive?
// OpAdd -> ActBinOp<EAdd> ?
// We defined ActBinOp<Op>.
// We need the Op to be the Lambda that performs addition.
// typelude_std::std::ops::EAdd ?
// EAdd is an internal struct for AST.
// We need a Lambda.
// Use `typelude_std::lambda::church::numeral::LAdd` for Church Numerals?
// Or `typelude_std::std::ops::OpAdd`?
// Let's assume `typelude_std::std::ops::OpAdd` works as an Apply-able struct.
// If not, we might need a wrapper.
// For now, let's use a placeholder `OpAddLambda`.
use typelude_std::std::ops::arith::OpAdd as OpAddLambda;

impl ToAction for OpAdd {
    type Output = ActBinOp<OpAddLambda>;
}

// --- Swap / Dup / Drop (Stack Ops) ---
// We need actions for these.
// ActDup = GetStack >>= \s. let x = head(s) in Push(x)
// ActSwap = Pop y, Pop x, Push y, Push x
// ActDrop = Pop, strict ignored.

// We need to define these Actions in `actions.rs` or here.
// For now let's skip them or define stub.

// =============================================================================
// Compile Trait (Instruction List -> Action)
// =============================================================================

pub trait Compile {
    type Output;
}

// Compile Nil -> Return Unit
impl Compile for Nil {
    type Output = LReturn<Unit>;
}

// Compile Cons<Op, Rest> -> Op >> Compile<Rest>
impl<Op, Rest> Compile for Array<Op, Rest>
where
    Op: ToAction,
    Rest: Compile + typelude_std::std::col::array::IsList,
{
    type Output = ActSeq<<Op as ToAction>::Output, <Rest as Compile>::Output>;
}

// =============================================================================
// Control Flow Compilation
// =============================================================================

// --- If ---
// OpIf<Then, Else> -> ActIf<Compile<Then>, Compile<Else>>
impl<Then, Else> ToAction for OpIf<Then, Else>
where
    Then: Compile,
    Else: Compile,
{
    type Output = ActBranch<<Then as Compile>::Output, <Else as Compile>::Output>;
}

impl<Cond, Body> ToAction for OpWhile<Cond, Body>
where
    Cond: Compile,
    Body: Compile,
{
    type Output = ActWhile<<Cond as Compile>::Output, <Body as Compile>::Output>;
}
