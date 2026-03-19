use typelude_std::{
    core::{ELit, Eval, Evaluate},
    std::col::array::{Array, IsList, Nil},
};

use crate::{
    opcode::{OpAdd, OpAnd, OpEq, OpGt, OpLt, OpNeq, OpNot, OpOr, OpSub},
    vm::{
        effect::{EmitTrace, RaiseTrap},
        step::{Continue, CoreMachine, StackUnderflow, Step, StepMachine},
    },
};

pub trait AsValueExpr {
    type Output;
}

impl<T> AsValueExpr for ELit<T> {
    type Output = ELit<T>;
}

impl AsValueExpr for typenum::UTerm {
    type Output = ELit<typenum::UTerm>;
}

impl<N, B> AsValueExpr for typenum::UInt<N, B> {
    type Output = ELit<typenum::UInt<N, B>>;
}

impl<U> AsValueExpr for typenum::PInt<U>
where
    U: typenum::Unsigned + typenum::NonZero,
{
    type Output = ELit<typenum::PInt<U>>;
}

impl<U> AsValueExpr for typenum::NInt<U>
where
    U: typenum::Unsigned + typenum::NonZero,
{
    type Output = ELit<typenum::NInt<U>>;
}

impl AsValueExpr for typenum::Z0 {
    type Output = ELit<typenum::Z0>;
}

impl AsValueExpr for typenum::B0 {
    type Output = ELit<typenum::B0>;
}

impl AsValueExpr for typenum::B1 {
    type Output = ELit<typenum::B1>;
}

impl AsValueExpr for typelude_std::std::prim::bool::True {
    type Output = ELit<typelude_std::std::prim::bool::True>;
}

impl AsValueExpr for typelude_std::std::prim::bool::False {
    type Output = ELit<typelude_std::std::prim::bool::False>;
}

pub trait ComputeBinary<Lhs, Rhs> {
    type Output;
}

impl<Lhs, Rhs> ComputeBinary<Lhs, Rhs> for OpAdd
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::EAdd<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::EAdd<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

impl<Lhs, Rhs> ComputeBinary<Lhs, Rhs> for OpSub
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::ESub<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::ESub<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

#[cfg(feature = "nightly")]
impl<Lhs, Rhs> ComputeBinary<Lhs, Rhs> for OpEq
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::EEq<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::EEq<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

#[cfg(feature = "nightly")]
impl<Lhs, Rhs> ComputeBinary<Lhs, Rhs> for OpNeq
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::ENeq<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::ENeq<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

impl<Lhs, Rhs> ComputeBinary<Lhs, Rhs> for OpLt
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::ELt<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::ELt<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

impl<Lhs, Rhs> ComputeBinary<Lhs, Rhs> for OpGt
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::EGt<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::EGt<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

impl<Lhs, Rhs> ComputeBinary<Lhs, Rhs> for OpAnd
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::EAnd<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::EAnd<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

impl<Lhs, Rhs> ComputeBinary<Lhs, Rhs> for OpOr
where
    Lhs: AsValueExpr,
    Rhs: AsValueExpr,
    typelude_std::std::ops::EOr<<Lhs as AsValueExpr>::Output, <Rhs as AsValueExpr>::Output>: Eval,
{
    type Output = ELit<
        Evaluate<
            typelude_std::std::ops::EOr<
                <Lhs as AsValueExpr>::Output,
                <Rhs as AsValueExpr>::Output,
            >,
        >,
    >;
}

pub trait BinaryStep<Inst, Locals, Memory, Frames, Labels, Rest, Meta, Fx> {
    type Output;
}

impl<Inst, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    BinaryStep<Inst, Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Nil
where
    Rest: IsList,
    Fx: RaiseTrap<
            StackUnderflow,
            StepMachine<Nil, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx>,
        >,
{
    type Output = <Fx as RaiseTrap<
        StackUnderflow,
        StepMachine<Nil, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx>,
    >>::Output;
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
    BinaryStep<Inst, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    for Array<Lhs, Array<Rhs, Tail>>
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
    Fx: RaiseTrap<
            StackUnderflow,
            StepMachine<Nil, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx>,
        >,
{
    type Output = <Fx as RaiseTrap<
        StackUnderflow,
        StepMachine<Nil, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx>,
    >>::Output;
}

impl<Val, Tail, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    UnaryStep<OpNot, Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Array<Val, Tail>
where
    Rest: IsList,
    Tail: IsList,
    Val: AsValueExpr,
    typelude_std::std::ops::ENot<<Val as AsValueExpr>::Output>: Eval,
    Fx: EmitTrace<OpNot, Meta>,
{
    type Output = Continue<
        CoreMachine<
            Array<
                ELit<Evaluate<typelude_std::std::ops::ENot<<Val as AsValueExpr>::Output>>>,
                Tail,
            >,
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
            type Output = <Stack as BinaryStep<
                $inst,
                Locals,
                Memory,
                Frames,
                Labels,
                Rest,
                Meta,
                Fx,
            >>::Output;
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
    type Output =
        <Stack as UnaryStep<OpNot, Locals, Memory, Frames, Labels, Rest, Meta, Fx>>::Output;
}
