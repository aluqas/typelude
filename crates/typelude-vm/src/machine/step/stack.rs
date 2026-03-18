use typelude_std::std::col::array::{Array, IsList, Nil};

use crate::machine::{
    effects::{EmitTrace, RaiseTrap},
    instr::core::{OpDrop, OpDup, OpPop, OpPush, OpSwap},
    result::Continue,
    step::{CoreMachine, StackUnderflow, Step, StepMachine},
};

pub trait PopFrame<Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx> {
    type Output;
}

impl<Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx>
    PopFrame<Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx> for Nil
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

impl<Head, Tail, Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx>
    PopFrame<Locals, Memory, Frames, Labels, Inst, Rest, Meta, Fx> for Array<Head, Tail>
where
    Rest: IsList,
    Tail: IsList,
    Fx: EmitTrace<Inst, Meta>,
{
    type Output = Continue<
        CoreMachine<
            Tail,
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

pub trait DupFrame<Locals, Memory, Frames, Labels, Rest, Meta, Fx> {
    type Output;
}

impl<Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    DupFrame<Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Nil
where
    Rest: IsList,
    Fx: RaiseTrap<
            StackUnderflow,
            StepMachine<Nil, Locals, Memory, Frames, Labels, OpDup, Rest, Meta, Fx>,
        >,
{
    type Output = <Fx as RaiseTrap<
        StackUnderflow,
        StepMachine<Nil, Locals, Memory, Frames, Labels, OpDup, Rest, Meta, Fx>,
    >>::Output;
}

impl<Head, Tail, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    DupFrame<Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Array<Head, Tail>
where
    Rest: IsList,
    Tail: IsList,
    Fx: EmitTrace<OpDup, Meta>,
{
    type Output = Continue<
        CoreMachine<
            Array<Head, Array<Head, Tail>>,
            Locals,
            Memory,
            Frames,
            Labels,
            Rest,
            <Fx as EmitTrace<OpDup, Meta>>::OutputMeta,
            Fx,
        >,
    >;
}

pub trait SwapFrame<Locals, Memory, Frames, Labels, Rest, Meta, Fx> {
    type Output;
}

impl<Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    SwapFrame<Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Nil
where
    Rest: IsList,
    Fx: RaiseTrap<
            StackUnderflow,
            StepMachine<Nil, Locals, Memory, Frames, Labels, OpSwap, Rest, Meta, Fx>,
        >,
{
    type Output = <Fx as RaiseTrap<
        StackUnderflow,
        StepMachine<Nil, Locals, Memory, Frames, Labels, OpSwap, Rest, Meta, Fx>,
    >>::Output;
}

impl<Head, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    SwapFrame<Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Array<Head, Nil>
where
    Rest: IsList,
    Fx: RaiseTrap<
            StackUnderflow,
            StepMachine<Array<Head, Nil>, Locals, Memory, Frames, Labels, OpSwap, Rest, Meta, Fx>,
        >,
{
    type Output = <Fx as RaiseTrap<
        StackUnderflow,
        StepMachine<Array<Head, Nil>, Locals, Memory, Frames, Labels, OpSwap, Rest, Meta, Fx>,
    >>::Output;
}

impl<A, B, Tail, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    SwapFrame<Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Array<A, Array<B, Tail>>
where
    Rest: IsList,
    Tail: IsList,
    Fx: EmitTrace<OpSwap, Meta>,
{
    type Output = Continue<
        CoreMachine<
            Array<B, Array<A, Tail>>,
            Locals,
            Memory,
            Frames,
            Labels,
            Rest,
            <Fx as EmitTrace<OpSwap, Meta>>::OutputMeta,
            Fx,
        >,
    >;
}

impl<V, Stack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    Step<StepMachine<Stack, Locals, Memory, Frames, Labels, OpPush<V>, Rest, Meta, Fx>>
    for OpPush<V>
where
    Rest: IsList,
    Stack: IsList,
    Fx: EmitTrace<OpPush<V>, Meta>,
{
    type Output = Continue<
        CoreMachine<
            Array<V, Stack>,
            Locals,
            Memory,
            Frames,
            Labels,
            Rest,
            <Fx as EmitTrace<OpPush<V>, Meta>>::OutputMeta,
            Fx,
        >,
    >;
}

impl<Stack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    Step<StepMachine<Stack, Locals, Memory, Frames, Labels, OpPop, Rest, Meta, Fx>> for OpPop
where
    Rest: IsList,
    Stack: PopFrame<Locals, Memory, Frames, Labels, OpPop, Rest, Meta, Fx>,
{
    type Output =
        <Stack as PopFrame<Locals, Memory, Frames, Labels, OpPop, Rest, Meta, Fx>>::Output;
}

impl<Stack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    Step<StepMachine<Stack, Locals, Memory, Frames, Labels, OpDrop, Rest, Meta, Fx>> for OpDrop
where
    Rest: IsList,
    Stack: PopFrame<Locals, Memory, Frames, Labels, OpDrop, Rest, Meta, Fx>,
{
    type Output =
        <Stack as PopFrame<Locals, Memory, Frames, Labels, OpDrop, Rest, Meta, Fx>>::Output;
}

impl<Stack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    Step<StepMachine<Stack, Locals, Memory, Frames, Labels, OpDup, Rest, Meta, Fx>> for OpDup
where
    Rest: IsList,
    Stack: DupFrame<Locals, Memory, Frames, Labels, Rest, Meta, Fx>,
{
    type Output = <Stack as DupFrame<Locals, Memory, Frames, Labels, Rest, Meta, Fx>>::Output;
}

impl<Stack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    Step<StepMachine<Stack, Locals, Memory, Frames, Labels, OpSwap, Rest, Meta, Fx>> for OpSwap
where
    Rest: IsList,
    Stack: SwapFrame<Locals, Memory, Frames, Labels, Rest, Meta, Fx>,
{
    type Output = <Stack as SwapFrame<Locals, Memory, Frames, Labels, Rest, Meta, Fx>>::Output;
}
