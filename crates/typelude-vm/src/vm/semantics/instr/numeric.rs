//! Numeric and boolean instruction semantics.
//!
//! Stack top is the array head. Binary operators consume `lhs` from the top of the stack and
//! `rhs` from the next slot. Boolean results are normalized to `ELit<True>` or `ELit<False>`.

use core::marker::PhantomData;

use typelude_std::{
    core::{Eval, TyFn},
    std::col::array::{Array, IsList, Nil},
};

use crate::{
    opcode::numeric::{OpAdd, OpAnd, OpEq, OpGt, OpLt, OpNeq, OpNot, OpOr, OpSub},
    vm::{
        protocol::trap::StackUnderflow,
        semantics::{
            helpers::value::{AsValueExpr, BinaryResult, BoolResult},
            state::VmState,
            step::{StepContinue, StepInstr, StepTrap},
        },
    },
};

pub struct LUnaryNot;
pub struct LBinaryStep<Inst>(pub PhantomData<Inst>);

impl<Locals, Memory, Frames, Rest> TyFn<VmState<Nil, Locals, Memory, Frames, Array<OpNot, Rest>>>
    for LUnaryNot
where
    Rest: IsList,
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Val, Tail, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Val, Tail>, Locals, Memory, Frames, Array<OpNot, Rest>>> for LUnaryNot
where
    Tail: IsList,
    Rest: IsList,
    Val: AsValueExpr,
    typelude_std::std::ops::ENot<<Val as AsValueExpr>::Output>: Eval,
    typelude_std::std::ops::ENot<<Val as AsValueExpr>::Output>:
        BoolResult<typelude_std::std::ops::ENot<<Val as AsValueExpr>::Output>>,
{
    type Output = StepContinue<
        VmState<
            Array<
                <
                    typelude_std::std::ops::ENot<
                        <Val as AsValueExpr>::Output,
                    > as BoolResult<
                        typelude_std::std::ops::ENot<<Val as AsValueExpr>::Output>,
                    >
                >::Output,
                Tail,
            >,
            Locals,
            Memory,
            Frames,
            Rest,
        >,
    >;
}

impl<Inst, Locals, Memory, Frames, Rest>
    TyFn<VmState<Nil, Locals, Memory, Frames, Array<Inst, Rest>>> for LBinaryStep<Inst>
where
    Rest: IsList,
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Inst, Head, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Head, Nil>, Locals, Memory, Frames, Array<Inst, Rest>>>
    for LBinaryStep<Inst>
where
    Rest: IsList,
{
    type Output = StepTrap<StackUnderflow>;
}

impl<Inst, Lhs, Rhs, Tail, Locals, Memory, Frames, Rest>
    TyFn<VmState<Array<Lhs, Array<Rhs, Tail>>, Locals, Memory, Frames, Array<Inst, Rest>>>
    for LBinaryStep<Inst>
where
    Inst: BinaryResult<Lhs, Rhs>,
    Tail: IsList,
    Rest: IsList,
{
    type Output = StepContinue<
        VmState<
            Array<<Inst as BinaryResult<Lhs, Rhs>>::Output, Tail>,
            Locals,
            Memory,
            Frames,
            Rest,
        >,
    >;
}

macro_rules! impl_binary_step_instr {
    ($inst:ty) => {
        impl<State> StepInstr<State> for $inst
        where
            LBinaryStep<$inst>: TyFn<State>,
        {
            type Output = <LBinaryStep<$inst> as TyFn<State>>::Output;
        }
    };
}

impl_binary_step_instr!(OpAdd);
impl_binary_step_instr!(OpSub);
#[cfg(feature = "nightly")]
impl_binary_step_instr!(OpEq);
#[cfg(feature = "nightly")]
impl_binary_step_instr!(OpNeq);
impl_binary_step_instr!(OpLt);
impl_binary_step_instr!(OpGt);
impl_binary_step_instr!(OpAnd);
impl_binary_step_instr!(OpOr);

impl<State> StepInstr<State> for OpNot
where
    LUnaryNot: TyFn<State>,
{
    type Output = <LUnaryNot as TyFn<State>>::Output;
}
