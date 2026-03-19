use typelude_std::{
    core::ELit,
    std::col::array::{Array, Concat, IsList, Nil},
};

use crate::{
    opcode::{OpIf, OpWhile},
    vm::{
        effect::{EmitTrace, RaiseTrap},
        kernel::LowerInstr,
        step::{Continue, CoreMachine, StackUnderflow, Step, StepMachine},
    },
};

pub trait ToBranchBool {
    type Output;
}

impl ToBranchBool for typelude_std::std::prim::bool::True {
    type Output = typelude_std::std::prim::bool::True;
}

impl ToBranchBool for typelude_std::std::prim::bool::False {
    type Output = typelude_std::std::prim::bool::False;
}

impl<T> ToBranchBool for ELit<T>
where
    T: ToBranchBool,
{
    type Output = <T as ToBranchBool>::Output;
}

pub trait SelectBranch<Then, Else> {
    type Output;
}

impl<Then, Else> SelectBranch<Then, Else> for typelude_std::std::prim::bool::True {
    type Output = Then;
}

impl<Then, Else> SelectBranch<Then, Else> for typelude_std::std::prim::bool::False {
    type Output = Else;
}

pub trait IfStep<ThenProg, ElseProg, Locals, Memory, Frames, Labels, Rest, Meta, Fx> {
    type Output;
}

impl<ThenProg, ElseProg, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    IfStep<ThenProg, ElseProg, Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Nil
where
    Rest: IsList,
    Fx: RaiseTrap<
            StackUnderflow,
            StepMachine<
                Nil,
                Locals,
                Memory,
                Frames,
                Labels,
                OpIf<ThenProg, ElseProg>,
                Rest,
                Meta,
                Fx,
            >,
        >,
{
    type Output = <Fx as RaiseTrap<
        StackUnderflow,
        StepMachine<Nil, Locals, Memory, Frames, Labels, OpIf<ThenProg, ElseProg>, Rest, Meta, Fx>,
    >>::Output;
}

impl<Cond, RestStack, ThenProg, ElseProg, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    IfStep<ThenProg, ElseProg, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    for Array<Cond, RestStack>
where
    Rest: IsList,
    Cond: ToBranchBool,
    RestStack: IsList,
    ThenProg: Concat<Rest>,
    ElseProg: Concat<Rest>,
    Fx: EmitTrace<OpIf<ThenProg, ElseProg>, Meta>,
    <Cond as ToBranchBool>::Output: SelectBranch<
            Continue<
                CoreMachine<
                    RestStack,
                    Locals,
                    Memory,
                    Frames,
                    Labels,
                    <ThenProg as Concat<Rest>>::Output,
                    <Fx as EmitTrace<OpIf<ThenProg, ElseProg>, Meta>>::OutputMeta,
                    Fx,
                >,
            >,
            Continue<
                CoreMachine<
                    RestStack,
                    Locals,
                    Memory,
                    Frames,
                    Labels,
                    <ElseProg as Concat<Rest>>::Output,
                    <Fx as EmitTrace<OpIf<ThenProg, ElseProg>, Meta>>::OutputMeta,
                    Fx,
                >,
            >,
        >,
{
    type Output = <<Cond as ToBranchBool>::Output as SelectBranch<
        Continue<
            CoreMachine<
                RestStack,
                Locals,
                Memory,
                Frames,
                Labels,
                <ThenProg as Concat<Rest>>::Output,
                <Fx as EmitTrace<OpIf<ThenProg, ElseProg>, Meta>>::OutputMeta,
                Fx,
            >,
        >,
        Continue<
            CoreMachine<
                RestStack,
                Locals,
                Memory,
                Frames,
                Labels,
                <ElseProg as Concat<Rest>>::Output,
                <Fx as EmitTrace<OpIf<ThenProg, ElseProg>, Meta>>::OutputMeta,
                Fx,
            >,
        >,
    >>::Output;
}

impl<ThenProg, ElseProg, Stack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    Step<
        StepMachine<
            Stack,
            Locals,
            Memory,
            Frames,
            Labels,
            OpIf<ThenProg, ElseProg>,
            Rest,
            Meta,
            Fx,
        >,
    > for OpIf<ThenProg, ElseProg>
where
    Rest: IsList,
    Stack: IfStep<ThenProg, ElseProg, Locals, Memory, Frames, Labels, Rest, Meta, Fx>,
{
    type Output = <Stack as IfStep<
        ThenProg,
        ElseProg,
        Locals,
        Memory,
        Frames,
        Labels,
        Rest,
        Meta,
        Fx,
    >>::Output;
}

impl<CondProg, BodyProg, Stack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    Step<
        StepMachine<
            Stack,
            Locals,
            Memory,
            Frames,
            Labels,
            OpWhile<CondProg, BodyProg>,
            Rest,
            Meta,
            Fx,
        >,
    > for OpWhile<CondProg, BodyProg>
where
    Rest: IsList,
    OpWhile<CondProg, BodyProg>: LowerInstr<Rest>,
    Fx: EmitTrace<OpWhile<CondProg, BodyProg>, Meta>,
{
    type Output = Continue<
        CoreMachine<
            Stack,
            Locals,
            Memory,
            Frames,
            Labels,
            <OpWhile<CondProg, BodyProg> as LowerInstr<Rest>>::Output,
            <Fx as EmitTrace<OpWhile<CondProg, BodyProg>, Meta>>::OutputMeta,
            Fx,
        >,
    >;
}
