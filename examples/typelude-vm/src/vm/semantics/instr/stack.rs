//! Stack instruction semantics.
//!
//! Stack top is the array head. All stack instructions commit their resulting
//! state on success and trap with `StackUnderflow` without committing on
//! insufficient input.

use typelude_col::{TArr, TTerm};
use typelude_std::core::Op;

use crate::{
    opcode::stack::{OpDrop, OpDup, OpPop, OpPush, OpSwap},
    vm::{
        protocol::trap::StackUnderflow,
        semantics::{
            state::VmState,
            step::{StepContinue, StepTrap},
        },
    },
};

impl<V, Stack, Locals, Memory, Frames, Rest>
    Op<VmState<Stack, Locals, Memory, Frames, TArr<OpPush<V>, Rest>>> for OpPush<V>
{
    type Output = StepContinue<VmState<TArr<V, Stack>, Locals, Memory, Frames, Rest>>;
}

impl<Locals, Memory, Frames, Rest> Op<VmState<TTerm, Locals, Memory, Frames, TArr<OpDrop, Rest>>>
    for OpDrop
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Head, Tail, Locals, Memory, Frames, Rest>
    Op<VmState<TArr<Head, Tail>, Locals, Memory, Frames, TArr<OpDrop, Rest>>> for OpDrop
{
    type Output = StepContinue<VmState<Tail, Locals, Memory, Frames, Rest>>;
}

impl<Locals, Memory, Frames, Rest> Op<VmState<TTerm, Locals, Memory, Frames, TArr<OpPop, Rest>>>
    for OpPop
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Head, Tail, Locals, Memory, Frames, Rest>
    Op<VmState<TArr<Head, Tail>, Locals, Memory, Frames, TArr<OpPop, Rest>>> for OpPop
{
    type Output = StepContinue<VmState<Tail, Locals, Memory, Frames, Rest>>;
}

impl<Locals, Memory, Frames, Rest> Op<VmState<TTerm, Locals, Memory, Frames, TArr<OpDup, Rest>>>
    for OpDup
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Head, Tail, Locals, Memory, Frames, Rest>
    Op<VmState<TArr<Head, Tail>, Locals, Memory, Frames, TArr<OpDup, Rest>>> for OpDup
{
    type Output =
        StepContinue<VmState<TArr<Head, TArr<Head, Tail>>, Locals, Memory, Frames, Rest>>;
}

impl<Locals, Memory, Frames, Rest> Op<VmState<TTerm, Locals, Memory, Frames, TArr<OpSwap, Rest>>>
    for OpSwap
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Head, Locals, Memory, Frames, Rest>
    Op<VmState<TArr<Head, TTerm>, Locals, Memory, Frames, TArr<OpSwap, Rest>>> for OpSwap
{
    type Output = StepTrap<StackUnderflow>;
}

impl<A, B, Tail, Locals, Memory, Frames, Rest>
    Op<VmState<TArr<A, TArr<B, Tail>>, Locals, Memory, Frames, TArr<OpSwap, Rest>>> for OpSwap
{
    type Output = StepContinue<VmState<TArr<B, TArr<A, Tail>>, Locals, Memory, Frames, Rest>>;
}
