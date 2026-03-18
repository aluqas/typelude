use core::ops::Sub;

use typelude_std::{
    core::{Eval, Evaluate},
    std::col::array::{Array, IsList, Nil},
};
use typenum::{B1, Sub1, U0, UInt, Unsigned};

use crate::machine::{
    effects::{EmitTrace, RaiseTrap},
    instr::core::{OpLoad, OpStore},
    result::{Continue, Trap},
    step::{BadMemoryIndex, CoreMachine, StackUnderflow, Step, StepMachine},
};

pub trait LoadFromMemory<Idx, RestStack, Locals, Frames, Labels, Rest, Meta, Fx> {
    type Output;
}

impl<Idx, RestStack, Locals, Frames, Labels, Rest, Meta, Fx>
    LoadFromMemory<Idx, RestStack, Locals, Frames, Labels, Rest, Meta, Fx> for Nil
where
    Rest: IsList,
    RestStack: IsList,
    Fx: RaiseTrap<
            BadMemoryIndex<Idx>,
            StepMachine<
                Array<Idx, RestStack>,
                Locals,
                Nil,
                Frames,
                Labels,
                OpLoad,
                Rest,
                Meta,
                Fx,
            >,
        >,
{
    type Output = <Fx as RaiseTrap<
        BadMemoryIndex<Idx>,
        StepMachine<Array<Idx, RestStack>, Locals, Nil, Frames, Labels, OpLoad, Rest, Meta, Fx>,
    >>::Output;
}

impl<Head, Tail, RestStack, Locals, Frames, Labels, Rest, Meta, Fx>
    LoadFromMemory<U0, RestStack, Locals, Frames, Labels, Rest, Meta, Fx> for Array<Head, Tail>
where
    Rest: IsList,
    RestStack: IsList,
    Tail: IsList,
    Fx: EmitTrace<OpLoad, Meta>,
{
    type Output = Continue<
        CoreMachine<
            Array<Head, RestStack>,
            Locals,
            Array<Head, Tail>,
            Frames,
            Labels,
            Rest,
            <Fx as EmitTrace<OpLoad, Meta>>::OutputMeta,
            Fx,
        >,
    >;
}

impl<Head, Tail, N, B, RestStack, Locals, Frames, Labels, Rest, Meta, Fx>
    LoadFromMemory<UInt<N, B>, RestStack, Locals, Frames, Labels, Rest, Meta, Fx>
    for Array<Head, Tail>
where
    Rest: IsList,
    UInt<N, B>: Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    Tail: LoadFromMemory<Sub1<UInt<N, B>>, RestStack, Locals, Frames, Labels, Rest, Meta, Fx>
        + IsList,
{
    type Output = <Tail as LoadFromMemory<
        Sub1<UInt<N, B>>,
        RestStack,
        Locals,
        Frames,
        Labels,
        Rest,
        Meta,
        Fx,
    >>::Output;
}

pub trait StoreInMemory<Idx, Value, RestStack, Locals, Frames, Labels, Rest, Meta, Fx> {
    type Output;
}

impl<Idx, Value, RestStack, Locals, Frames, Labels, Rest, Meta, Fx>
    StoreInMemory<Idx, Value, RestStack, Locals, Frames, Labels, Rest, Meta, Fx> for Nil
where
    Rest: IsList,
    RestStack: IsList,
    Fx: RaiseTrap<
            BadMemoryIndex<Idx>,
            StepMachine<
                Array<Value, Array<Idx, RestStack>>,
                Locals,
                Nil,
                Frames,
                Labels,
                OpStore,
                Rest,
                Meta,
                Fx,
            >,
        >,
{
    type Output = <Fx as RaiseTrap<
        BadMemoryIndex<Idx>,
        StepMachine<
            Array<Value, Array<Idx, RestStack>>,
            Locals,
            Nil,
            Frames,
            Labels,
            OpStore,
            Rest,
            Meta,
            Fx,
        >,
    >>::Output;
}

impl<Head, Tail, Value, RestStack, Locals, Frames, Labels, Rest, Meta, Fx>
    StoreInMemory<U0, Value, RestStack, Locals, Frames, Labels, Rest, Meta, Fx>
    for Array<Head, Tail>
where
    Rest: IsList,
    Tail: IsList,
    RestStack: IsList,
    Fx: EmitTrace<OpStore, Meta>,
{
    type Output = Continue<
        CoreMachine<
            RestStack,
            Locals,
            Array<Value, Tail>,
            Frames,
            Labels,
            Rest,
            <Fx as EmitTrace<OpStore, Meta>>::OutputMeta,
            Fx,
        >,
    >;
}

impl<Head, Tail, N, B, Value, RestStack, Locals, Frames, Labels, Rest, Meta, Fx>
    StoreInMemory<UInt<N, B>, Value, RestStack, Locals, Frames, Labels, Rest, Meta, Fx>
    for Array<Head, Tail>
where
    Rest: IsList,
    UInt<N, B>: Sub<B1>,
    Sub1<UInt<N, B>>: Unsigned,
    Tail: StoreInMemory<Sub1<UInt<N, B>>, Value, RestStack, Locals, Frames, Labels, Rest, Meta, Fx>
        + IsList,
    <Tail as StoreInMemory<
        Sub1<UInt<N, B>>,
        Value,
        RestStack,
        Locals,
        Frames,
        Labels,
        Rest,
        Meta,
        Fx,
    >>::Output: LiftStoredMemory<Head>,
{
    type Output = <<Tail as StoreInMemory<
        Sub1<UInt<N, B>>,
        Value,
        RestStack,
        Locals,
        Frames,
        Labels,
        Rest,
        Meta,
        Fx,
    >>::Output as LiftStoredMemory<Head>>::Output;
}

pub trait LiftStoredMemory<Head> {
    type Output;
}

impl<Head, Stack, Locals, Tail, Frames, Labels, Rest, Meta, Fx> LiftStoredMemory<Head>
    for Continue<CoreMachine<Stack, Locals, Tail, Frames, Labels, Rest, Meta, Fx>>
where
    Tail: IsList,
{
    type Output =
        Continue<CoreMachine<Stack, Locals, Array<Head, Tail>, Frames, Labels, Rest, Meta, Fx>>;
}

impl<Reason, M, Head> LiftStoredMemory<Head> for Trap<Reason, M> {
    type Output = Trap<Reason, M>;
}

pub trait LoadStep<Locals, Memory, Frames, Labels, Rest, Meta, Fx> {
    type Output;
}

impl<Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    LoadStep<Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Nil
where
    Rest: IsList,
    Fx: RaiseTrap<
            StackUnderflow,
            StepMachine<Nil, Locals, Memory, Frames, Labels, OpLoad, Rest, Meta, Fx>,
        >,
{
    type Output = <Fx as RaiseTrap<
        StackUnderflow,
        StepMachine<Nil, Locals, Memory, Frames, Labels, OpLoad, Rest, Meta, Fx>,
    >>::Output;
}

impl<Addr, RestStack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    LoadStep<Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Array<Addr, RestStack>
where
    Rest: IsList,
    Addr: Eval,
    Evaluate<Addr>: Unsigned,
    RestStack: IsList,
    Memory: LoadFromMemory<Evaluate<Addr>, RestStack, Locals, Frames, Labels, Rest, Meta, Fx>,
{
    type Output = <Memory as LoadFromMemory<
        Evaluate<Addr>,
        RestStack,
        Locals,
        Frames,
        Labels,
        Rest,
        Meta,
        Fx,
    >>::Output;
}

pub trait StoreStep<Locals, Memory, Frames, Labels, Rest, Meta, Fx> {
    type Output;
}

impl<Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    StoreStep<Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Nil
where
    Rest: IsList,
    Fx: RaiseTrap<
            StackUnderflow,
            StepMachine<Nil, Locals, Memory, Frames, Labels, OpStore, Rest, Meta, Fx>,
        >,
{
    type Output = <Fx as RaiseTrap<
        StackUnderflow,
        StepMachine<Nil, Locals, Memory, Frames, Labels, OpStore, Rest, Meta, Fx>,
    >>::Output;
}

impl<Value, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    StoreStep<Locals, Memory, Frames, Labels, Rest, Meta, Fx> for Array<Value, Nil>
where
    Rest: IsList,
    Fx: RaiseTrap<
            StackUnderflow,
            StepMachine<
                Array<Value, Nil>,
                Locals,
                Memory,
                Frames,
                Labels,
                OpStore,
                Rest,
                Meta,
                Fx,
            >,
        >,
{
    type Output = <Fx as RaiseTrap<
        StackUnderflow,
        StepMachine<Array<Value, Nil>, Locals, Memory, Frames, Labels, OpStore, Rest, Meta, Fx>,
    >>::Output;
}

impl<Value, Addr, RestStack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    StoreStep<Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    for Array<Value, Array<Addr, RestStack>>
where
    Rest: IsList,
    Addr: Eval,
    Evaluate<Addr>: Unsigned,
    RestStack: IsList,
    Memory:
        StoreInMemory<Evaluate<Addr>, Value, RestStack, Locals, Frames, Labels, Rest, Meta, Fx>,
{
    type Output = <Memory as StoreInMemory<
        Evaluate<Addr>,
        Value,
        RestStack,
        Locals,
        Frames,
        Labels,
        Rest,
        Meta,
        Fx,
    >>::Output;
}

impl<Stack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    Step<StepMachine<Stack, Locals, Memory, Frames, Labels, OpLoad, Rest, Meta, Fx>> for OpLoad
where
    Rest: IsList,
    Stack: LoadStep<Locals, Memory, Frames, Labels, Rest, Meta, Fx>,
{
    type Output = <Stack as LoadStep<Locals, Memory, Frames, Labels, Rest, Meta, Fx>>::Output;
}

impl<Stack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    Step<StepMachine<Stack, Locals, Memory, Frames, Labels, OpStore, Rest, Meta, Fx>> for OpStore
where
    Rest: IsList,
    Stack: StoreStep<Locals, Memory, Frames, Labels, Rest, Meta, Fx>,
{
    type Output = <Stack as StoreStep<Locals, Memory, Frames, Labels, Rest, Meta, Fx>>::Output;
}
