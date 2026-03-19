use core::ops::Sub;

use typelude_std::{
    core::{Eval, Evaluate},
    std::col::array::{Array, IsList, Nil},
};
use typenum::{B1, Sub1, U0, UInt, Unsigned};

use crate::{
    opcode::{OpDropLocal, OpGetLocal, OpLet, OpSetLocal},
    vm::{
        effect::{EmitTrace, RaiseTrap},
        step::{
            BadLocalIndex, Continue, CoreMachine, LocalUnderflow, StackUnderflow, Step,
            StepMachine,
        },
    },
};

pub trait GetLocalStep<Idx, Stack, Memory, Frames, Labels, Rest, Meta, Fx> {
    type Output;
}

impl<Idx, Stack, Memory, Frames, Labels, Rest, Meta, Fx>
    GetLocalStep<Idx, Stack, Memory, Frames, Labels, Rest, Meta, Fx> for Nil
where
    Rest: IsList,
    Fx: RaiseTrap<
            BadLocalIndex<Idx>,
            StepMachine<Stack, Nil, Memory, Frames, Labels, OpGetLocal<Idx>, Rest, Meta, Fx>,
        >,
{
    type Output = <Fx as RaiseTrap<
        BadLocalIndex<Idx>,
        StepMachine<Stack, Nil, Memory, Frames, Labels, OpGetLocal<Idx>, Rest, Meta, Fx>,
    >>::Output;
}

impl<Head, Tail, Stack, Memory, Frames, Labels, Rest, Meta, Fx>
    GetLocalStep<U0, Stack, Memory, Frames, Labels, Rest, Meta, Fx> for Array<Head, Tail>
where
    Rest: IsList,
    Stack: IsList,
    Tail: IsList,
    Fx: EmitTrace<OpGetLocal<U0>, Meta>,
{
    type Output = Continue<
        CoreMachine<
            Array<Head, Stack>,
            Array<Head, Tail>,
            Memory,
            Frames,
            Labels,
            Rest,
            <Fx as EmitTrace<OpGetLocal<U0>, Meta>>::OutputMeta,
            Fx,
        >,
    >;
}

impl<Head, Tail, N, B, Stack, Memory, Frames, Labels, Rest, Meta, Fx>
    GetLocalStep<UInt<N, B>, Stack, Memory, Frames, Labels, Rest, Meta, Fx> for Array<Head, Tail>
where
    Rest: IsList,
    UInt<N, B>: Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    Tail: GetLocalStep<Sub1<UInt<N, B>>, Stack, Memory, Frames, Labels, Rest, Meta, Fx> + IsList,
{
    type Output = <Tail as GetLocalStep<
        Sub1<UInt<N, B>>,
        Stack,
        Memory,
        Frames,
        Labels,
        Rest,
        Meta,
        Fx,
    >>::Output;
}

pub trait SetLocalStep<Idx, Value, Memory, Frames, Labels, Rest, Meta, Fx> {
    type Output;
}

impl<Idx, Value, Memory, Frames, Labels, Rest, Meta, Fx>
    SetLocalStep<Idx, Value, Memory, Frames, Labels, Rest, Meta, Fx> for Nil
where
    Rest: IsList,
    Fx: RaiseTrap<
            BadLocalIndex<Idx>,
            StepMachine<
                Array<Value, Nil>,
                Nil,
                Memory,
                Frames,
                Labels,
                OpSetLocal<Idx>,
                Rest,
                Meta,
                Fx,
            >,
        >,
{
    type Output = <Fx as RaiseTrap<
        BadLocalIndex<Idx>,
        StepMachine<
            Array<Value, Nil>,
            Nil,
            Memory,
            Frames,
            Labels,
            OpSetLocal<Idx>,
            Rest,
            Meta,
            Fx,
        >,
    >>::Output;
}

impl<Head, Tail, Value, Memory, Frames, Labels, Rest, Meta, Fx>
    SetLocalStep<U0, Value, Memory, Frames, Labels, Rest, Meta, Fx> for Array<Head, Tail>
where
    Rest: IsList,
    Tail: IsList,
    Fx: EmitTrace<OpSetLocal<U0>, Meta>,
{
    type Output = Continue<
        CoreMachine<
            Tail,
            Array<Value, Tail>,
            Memory,
            Frames,
            Labels,
            Rest,
            <Fx as EmitTrace<OpSetLocal<U0>, Meta>>::OutputMeta,
            Fx,
        >,
    >;
}

impl<Head, Tail, N, B, Value, Memory, Frames, Labels, Rest, Meta, Fx>
    SetLocalStep<UInt<N, B>, Value, Memory, Frames, Labels, Rest, Meta, Fx> for Array<Head, Tail>
where
    Rest: IsList,
    UInt<N, B>: Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    Tail: SetLocalStep<Sub1<UInt<N, B>>, Value, Memory, Frames, Labels, Rest, Meta, Fx> + IsList,
    <Tail as SetLocalStep<Sub1<UInt<N, B>>, Value, Memory, Frames, Labels, Rest, Meta, Fx>>::Output:
        LiftSetLocal<Head, Memory, Frames, Labels, Rest, Meta, Fx>,
{
    type Output = <<Tail as SetLocalStep<Sub1<UInt<N, B>>, Value, Memory, Frames, Labels, Rest, Meta, Fx>>::Output
        as LiftSetLocal<Head, Memory, Frames, Labels, Rest, Meta, Fx>>::Output;
}

pub trait LiftSetLocal<Head, Memory, Frames, Labels, Rest, Meta, Fx> {
    type Output;
}

impl<Head, Stack, Tail, Memory, Frames, Labels, Rest, Meta, Fx>
    LiftSetLocal<Head, Memory, Frames, Labels, Rest, Meta, Fx>
    for Continue<CoreMachine<Stack, Tail, Memory, Frames, Labels, Rest, Meta, Fx>>
where
    Tail: IsList,
{
    type Output =
        Continue<CoreMachine<Stack, Array<Head, Tail>, Memory, Frames, Labels, Rest, Meta, Fx>>;
}

impl<Reason, M, Head, Memory, Frames, Labels, Rest, Meta, Fx>
    LiftSetLocal<Head, Memory, Frames, Labels, Rest, Meta, Fx>
    for crate::vm::step::result::Trap<Reason, M>
{
    type Output = crate::vm::step::result::Trap<Reason, M>;
}

impl<Stack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    Step<StepMachine<Stack, Locals, Memory, Frames, Labels, OpLet, Rest, Meta, Fx>> for OpLet
where
    Rest: IsList,
    Stack: LetStep<Locals, Memory, Frames, Labels, Rest, Meta, Fx>,
{
    type Output = <Stack as LetStep<Locals, Memory, Frames, Labels, Rest, Meta, Fx>>::Output;
}

pub trait LetStep<Locals, Memory, Frames, Labels, Rest, Meta, Fx> {
    type Output;
}

impl<Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    LetStep<Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Nil
where
    Rest: IsList,
    Fx: RaiseTrap<
            StackUnderflow,
            StepMachine<Nil, Locals, Memory, Frames, Labels, OpLet, Rest, Meta, Fx>,
        >,
{
    type Output = <Fx as RaiseTrap<
        StackUnderflow,
        StepMachine<Nil, Locals, Memory, Frames, Labels, OpLet, Rest, Meta, Fx>,
    >>::Output;
}

impl<Value, Tail, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    LetStep<Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Array<Value, Tail>
where
    Rest: IsList,
    Tail: IsList,
    Locals: IsList,
    Fx: EmitTrace<OpLet, Meta>,
{
    type Output = Continue<
        CoreMachine<
            Tail,
            Array<Value, Locals>,
            Memory,
            Frames,
            Labels,
            Rest,
            <Fx as EmitTrace<OpLet, Meta>>::OutputMeta,
            Fx,
        >,
    >;
}

impl<Stack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    Step<StepMachine<Stack, Locals, Memory, Frames, Labels, OpDropLocal, Rest, Meta, Fx>>
    for OpDropLocal
where
    Rest: IsList,
    Locals: DropLocalStep<Stack, Memory, Frames, Labels, Rest, Meta, Fx>,
{
    type Output = <Locals as DropLocalStep<Stack, Memory, Frames, Labels, Rest, Meta, Fx>>::Output;
}

pub trait DropLocalStep<Stack, Memory, Frames, Labels, Rest, Meta, Fx> {
    type Output;
}

impl<Stack, Memory, Frames, Labels, Rest, Meta, Fx>
    DropLocalStep<Stack, Memory, Frames, Labels, Rest, Meta, Fx> for Nil
where
    Rest: IsList,
    Fx: RaiseTrap<
            LocalUnderflow,
            StepMachine<Stack, Nil, Memory, Frames, Labels, OpDropLocal, Rest, Meta, Fx>,
        >,
{
    type Output = <Fx as RaiseTrap<
        LocalUnderflow,
        StepMachine<Stack, Nil, Memory, Frames, Labels, OpDropLocal, Rest, Meta, Fx>,
    >>::Output;
}

impl<Head, Tail, Stack, Memory, Frames, Labels, Rest, Meta, Fx>
    DropLocalStep<Stack, Memory, Frames, Labels, Rest, Meta, Fx> for Array<Head, Tail>
where
    Rest: IsList,
    Tail: IsList,
    Fx: EmitTrace<OpDropLocal, Meta>,
{
    type Output = Continue<
        CoreMachine<
            Stack,
            Tail,
            Memory,
            Frames,
            Labels,
            Rest,
            <Fx as EmitTrace<OpDropLocal, Meta>>::OutputMeta,
            Fx,
        >,
    >;
}

impl<Index, Stack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    Step<StepMachine<Stack, Locals, Memory, Frames, Labels, OpGetLocal<Index>, Rest, Meta, Fx>>
    for OpGetLocal<Index>
where
    Rest: IsList,
    Index: Eval,
    Evaluate<Index>: Unsigned,
    Locals: GetLocalStep<Evaluate<Index>, Stack, Memory, Frames, Labels, Rest, Meta, Fx>,
{
    type Output = <Locals as GetLocalStep<
        Evaluate<Index>,
        Stack,
        Memory,
        Frames,
        Labels,
        Rest,
        Meta,
        Fx,
    >>::Output;
}

impl<Index, Value, RestStack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    Step<
        StepMachine<
            Array<Value, RestStack>,
            Locals,
            Memory,
            Frames,
            Labels,
            OpSetLocal<Index>,
            Rest,
            Meta,
            Fx,
        >,
    > for OpSetLocal<Index>
where
    Rest: IsList,
    Index: Eval,
    Evaluate<Index>: Unsigned,
    RestStack: IsList,
    Locals: SetLocalStep<Evaluate<Index>, Value, Memory, Frames, Labels, Rest, Meta, Fx>,
{
    type Output = <Locals as SetLocalStep<
        Evaluate<Index>,
        Value,
        Memory,
        Frames,
        Labels,
        Rest,
        Meta,
        Fx,
    >>::Output;
}
