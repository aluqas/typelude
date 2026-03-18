use typelude_std::{
    core::{Eval, Evaluate},
    std::col::array::{Array, IsList, Nil},
};

use crate::machine::{
    effects::{EmitTrace, RaiseTrap},
    instr::core::{OpAdd, OpAnd, OpEq, OpGt, OpLt, OpNeq, OpNot, OpOr, OpSub},
    result::Continue,
    step::{CoreMachine, StackUnderflow, Step, StepMachine},
};

pub trait ComputeBinary<Lhs, Rhs> {
    type Output;
}

impl<Lhs, Rhs> ComputeBinary<Lhs, Rhs> for OpAdd
where
    typelude_std::std::ops::EAdd<Lhs, Rhs>: Eval,
{
    type Output = Evaluate<typelude_std::std::ops::EAdd<Lhs, Rhs>>;
}

impl<Lhs, Rhs> ComputeBinary<Lhs, Rhs> for OpSub
where
    typelude_std::std::ops::ESub<Lhs, Rhs>: Eval,
{
    type Output = Evaluate<typelude_std::std::ops::ESub<Lhs, Rhs>>;
}

#[cfg(feature = "nightly")]
impl<Lhs, Rhs> ComputeBinary<Lhs, Rhs> for OpEq
where
    typelude_std::std::ops::EEq<Lhs, Rhs>: Eval,
{
    type Output = Evaluate<typelude_std::std::ops::EEq<Lhs, Rhs>>;
}

#[cfg(feature = "nightly")]
impl<Lhs, Rhs> ComputeBinary<Lhs, Rhs> for OpNeq
where
    typelude_std::std::ops::ENeq<Lhs, Rhs>: Eval,
{
    type Output = Evaluate<typelude_std::std::ops::ENeq<Lhs, Rhs>>;
}

impl<Lhs, Rhs> ComputeBinary<Lhs, Rhs> for OpLt
where
    typelude_std::std::ops::ELt<Lhs, Rhs>: Eval,
{
    type Output = Evaluate<typelude_std::std::ops::ELt<Lhs, Rhs>>;
}

impl<Lhs, Rhs> ComputeBinary<Lhs, Rhs> for OpGt
where
    typelude_std::std::ops::EGt<Lhs, Rhs>: Eval,
{
    type Output = Evaluate<typelude_std::std::ops::EGt<Lhs, Rhs>>;
}

impl<Lhs, Rhs> ComputeBinary<Lhs, Rhs> for OpAnd
where
    typelude_std::std::ops::EAnd<Lhs, Rhs>: Eval,
{
    type Output = Evaluate<typelude_std::std::ops::EAnd<Lhs, Rhs>>;
}

impl<Lhs, Rhs> ComputeBinary<Lhs, Rhs> for OpOr
where
    typelude_std::std::ops::EOr<Lhs, Rhs>: Eval,
{
    type Output = Evaluate<typelude_std::std::ops::EOr<Lhs, Rhs>>;
}

pub trait BinaryStep<Inst, Locals, Memory, Frames, Labels, Rest, Meta, Fx> {
    type Output;
}

impl<Inst, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    BinaryStep<Inst, Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Nil
where
    Rest: IsList,
    Fx: RaiseTrap<StackUnderflow, StepMachine<Nil, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx>>,
{
    type Output =
        <Fx as RaiseTrap<StackUnderflow, StepMachine<Nil, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx>>>::Output;
}

impl<Head, Inst, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    BinaryStep<Inst, Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Array<Head, Nil>
where
    Rest: IsList,
    Fx: RaiseTrap<
        StackUnderflow,
        StepMachine<Array<Head, Nil>, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx>,
    >,
{
    type Output = <Fx as RaiseTrap<
        StackUnderflow,
        StepMachine<Array<Head, Nil>, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx>,
    >>::Output;
}

impl<Inst, Lhs, Rhs, Tail, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    BinaryStep<Inst, Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Array<Lhs, Array<Rhs, Tail>>
where
    Rest: IsList,
    Inst: ComputeBinary<Lhs, Rhs>,
    Tail: IsList,
    Fx: EmitTrace<Inst, Meta>,
{
    type Output = Continue<
        CoreMachine<
            Array<<Inst as ComputeBinary<Lhs, Rhs>>::Output, Tail>,
            Locals,
            Memory,
            Frames,
            Labels,
            Rest,
            <Fx as EmitTrace<Inst, Meta>>::OutputMeta,
            Fx,
        >,
    >;
}

pub trait UnaryStep<Inst, Locals, Memory, Frames, Labels, Rest, Meta, Fx> {
    type Output;
}

impl<Inst, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    UnaryStep<Inst, Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Nil
where
    Rest: IsList,
    Fx: RaiseTrap<StackUnderflow, StepMachine<Nil, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx>>,
{
    type Output =
        <Fx as RaiseTrap<StackUnderflow, StepMachine<Nil, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx>>>::Output;
}

impl<Val, Tail, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    UnaryStep<OpNot, Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Array<Val, Tail>
where
    Rest: IsList,
    Tail: IsList,
    typelude_std::std::ops::ENot<Val>: Eval,
    Fx: EmitTrace<OpNot, Meta>,
{
    type Output = Continue<
        CoreMachine<
            Array<Evaluate<typelude_std::std::ops::ENot<Val>>, Tail>,
            Locals,
            Memory,
            Frames,
            Labels,
            Rest,
            <Fx as EmitTrace<OpNot, Meta>>::OutputMeta,
            Fx,
        >,
    >;
}

macro_rules! impl_binary_step {
    ($inst:ty) => {
        impl<Stack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
            Step<StepMachine<Stack, Locals, Memory, Frames, Labels, $inst, Rest, Meta, Fx>>
            for $inst
        where
            Rest: IsList,
            Stack: BinaryStep<$inst, Locals, Memory, Frames, Labels, Rest, Meta, Fx>,
        {
            type Output =
                <Stack as BinaryStep<$inst, Locals, Memory, Frames, Labels, Rest, Meta, Fx>>::Output;
        }
    };
}

impl_binary_step!(OpAdd);
impl_binary_step!(OpSub);
#[cfg(feature = "nightly")]
impl_binary_step!(OpEq);
#[cfg(feature = "nightly")]
impl_binary_step!(OpNeq);
impl_binary_step!(OpLt);
impl_binary_step!(OpGt);
impl_binary_step!(OpAnd);
impl_binary_step!(OpOr);

impl<Stack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    Step<StepMachine<Stack, Locals, Memory, Frames, Labels, OpNot, Rest, Meta, Fx>> for OpNot
where
    Rest: IsList,
    Stack: UnaryStep<OpNot, Locals, Memory, Frames, Labels, Rest, Meta, Fx>,
{
    type Output = <Stack as UnaryStep<OpNot, Locals, Memory, Frames, Labels, Rest, Meta, Fx>>::Output;
}
