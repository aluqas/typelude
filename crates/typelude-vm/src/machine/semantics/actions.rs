//! **Primitive Monadic Actions**
//!
//! Atomic operations defined as State Monad types `S -> (A, S')`.
//! Using `Apply` trait for reductions.

use std::marker::PhantomData;

use typelude_std::core::{Eval, Evaluate};
use typelude_std::{
    lambda::church::LPair2,
    std::col::array::Array,
};
use typelude_std::core::Apply;

use crate::machine::semantics::{monad::Unit, state::VmState};

// =============================================================================
// Stack Actions
// =============================================================================

// --- Push ---
// Push(V): State -> ((), State')
// Stack: S -> Cons<V, S>
pub struct ActPush<Val>(PhantomData<Val>);

impl<Val> Eval for ActPush<Val> {
    type Output = ActPush<Val>;
}

impl<Val, S, L, M, C> Apply<VmState<S, L, M, C>> for ActPush<Val>
where
    Val: Eval,
    S: Eval,
    Evaluate<S>: typelude_std::std::col::array::IsList,
{
    // Return (Unit, NewState)
    type Output = LPair2<Unit, VmState<Array<Evaluate<Val>, Evaluate<S>>, L, M, C>>;
}

// --- Pop ---
// Pop: State -> (Val, State')
// Stack: Cons<Val, Rest> -> Rest
pub struct ActPop;
impl Eval for ActPop {
    type Output = ActPop;
}

impl<Head, Tail, L, M, C> Apply<VmState<Array<Head, Tail>, L, M, C>> for ActPop
where
    Head: Eval,
    Tail: Eval + typelude_std::std::col::array::IsList,
{
    // Return (Head, NewState)
    type Output = LPair2<Evaluate<Head>, VmState<Evaluate<Tail>, L, M, C>>;
}

// =============================================================================
// Arithmetic / Logic Actions
// =============================================================================

// --- BinOp ---
// BinOp<Op>: State -> ((), State')
// Stack: Cons<Rhs, Cons<Lhs, Rest>> -> Cons<Op<Lhs, Rhs>, Rest>

pub struct ActBinOp<Op>(PhantomData<Op>);

impl<Op> Eval for ActBinOp<Op> {
    type Output = ActBinOp<Op>;
}

impl<Op, Rhs, Lhs, Rest, L, M, C> Apply<VmState<Array<Rhs, Array<Lhs, Rest>>, L, M, C>>
    for ActBinOp<Op>
where
    Rest: Eval + typelude_std::std::col::array::IsList,
    // Construct Args Tuple: (Lhs, Rhs)
    // Apply Op to Args
    Op: Apply<(Lhs, Rhs)>,
    <Op as Apply<(Lhs, Rhs)>>::Output: Eval,
    Evaluate<Rest>: typelude_std::std::col::array::IsList,
{
    type Output = LPair2<
        Unit,
        VmState<
            Array<
                Evaluate<
                    <Op as Apply<(Lhs, Rhs)>>::Output,
                >,
                Evaluate<Rest>,
            >,
            L,
            M,
            C,
        >,
    >;
}
