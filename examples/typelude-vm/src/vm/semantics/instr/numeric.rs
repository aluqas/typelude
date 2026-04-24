//! Numeric and boolean instruction semantics.
//!
//! Stack top is the array head. Binary operators consume `lhs` from the top of
//! the stack and `rhs` from the next slot. Boolean results are normalized to
//! `Lit<True>` or `Lit<False>`.

use typelude_col::{TArr, TTerm};
use typelude_std::core::Op;

use crate::{
    opcode::numeric::{OpAdd, OpAnd, OpEq, OpGt, OpLt, OpNeq, OpNot, OpOr, OpSub},
    vm::{
        protocol::trap::StackUnderflow,
        semantics::{
            helpers::value::{BinaryInstrResult, UnaryInstrResult},
            state::VmState,
            step::{StepContinue, StepTrap},
        },
    },
};

impl<Locals, Memory, Frames, Rest> Op<VmState<TTerm, Locals, Memory, Frames, TArr<OpNot, Rest>>>
    for OpNot
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Val, Tail, Locals, Memory, Frames, Rest>
    Op<VmState<TArr<Val, Tail>, Locals, Memory, Frames, TArr<OpNot, Rest>>> for OpNot
where
    Val: UnaryInstrResult<OpNot>,
{
    type Output = StepContinue<
        VmState<
            TArr<<Val as UnaryInstrResult<OpNot>>::Output, Tail>,
            Locals,
            Memory,
            Frames,
            Rest,
        >,
    >;
}

macro_rules! impl_binary_instr {
    ($inst:ty) => {
        impl<Locals, Memory, Frames, Rest>
            Op<VmState<TTerm, Locals, Memory, Frames, TArr<$inst, Rest>>> for $inst
        {
            type Output = StepTrap<StackUnderflow>;
        }

        impl<Head, Locals, Memory, Frames, Rest>
            Op<VmState<TArr<Head, TTerm>, Locals, Memory, Frames, TArr<$inst, Rest>>> for $inst
        {
            type Output = StepTrap<StackUnderflow>;
        }

        impl<Lhs, Rhs, Tail, Locals, Memory, Frames, Rest>
            Op<VmState<TArr<Lhs, TArr<Rhs, Tail>>, Locals, Memory, Frames, TArr<$inst, Rest>>>
            for $inst
        where
            Lhs: BinaryInstrResult<$inst, Rhs>,
        {
            type Output = StepContinue<
                VmState<
                    TArr<<Lhs as BinaryInstrResult<$inst, Rhs>>::Output, Tail>,
                    Locals,
                    Memory,
                    Frames,
                    Rest,
                >,
            >;
        }
    };
}

impl_binary_instr!(OpAdd);
impl_binary_instr!(OpSub);
impl_binary_instr!(OpEq);
impl_binary_instr!(OpNeq);
impl_binary_instr!(OpLt);
impl_binary_instr!(OpGt);
impl_binary_instr!(OpAnd);
impl_binary_instr!(OpOr);
