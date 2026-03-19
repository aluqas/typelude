use typelude_std::std::col::array::{Array, IsList, Nil};

use crate::{
    opcode::{OpCall, OpReturn},
    vm::{
        effect::{EmitTrace, RaiseTrap},
        state::CallFrame,
        step::{Continue, CoreMachine, ReturnUnderflow, Step, StepMachine},
    },
};

impl<TargetProg, Stack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    Step<StepMachine<Stack, Locals, Memory, Frames, Labels, OpCall<TargetProg>, Rest, Meta, Fx>>
    for OpCall<TargetProg>
where
    Rest: IsList,
    Frames: IsList,
    Fx: EmitTrace<OpCall<TargetProg>, Meta>,
{
    type Output = Continue<
        CoreMachine<
            Stack,
            Nil,
            Memory,
            Array<CallFrame<Rest, Locals, Labels>, Frames>,
            Nil,
            TargetProg,
            <Fx as EmitTrace<OpCall<TargetProg>, Meta>>::OutputMeta,
            Fx,
        >,
    >;
}

pub trait ReturnStep<Stack, Locals, Memory, Labels, Rest, Meta, Fx> {
    type Output;
}

impl<Stack, Locals, Memory, Labels, Rest, Meta, Fx>
    ReturnStep<Stack, Locals, Memory, Labels, Rest, Meta, Fx> for Nil
where
    Rest: IsList,
    Fx: RaiseTrap<
            ReturnUnderflow,
            StepMachine<Stack, Locals, Memory, Nil, Labels, OpReturn, Rest, Meta, Fx>,
        >,
{
    type Output = <Fx as RaiseTrap<
        ReturnUnderflow,
        StepMachine<Stack, Locals, Memory, Nil, Labels, OpReturn, Rest, Meta, Fx>,
    >>::Output;
}

impl<
    Continuation,
    CallerLocals,
    CallerLabels,
    RestFrames,
    Stack,
    Locals,
    Memory,
    Labels,
    Rest,
    Meta,
    Fx,
> ReturnStep<Stack, Locals, Memory, Labels, Rest, Meta, Fx>
    for Array<CallFrame<Continuation, CallerLocals, CallerLabels>, RestFrames>
where
    Rest: IsList,
    RestFrames: IsList,
    Fx: EmitTrace<OpReturn, Meta>,
{
    type Output = Continue<
        CoreMachine<
            Stack,
            CallerLocals,
            Memory,
            RestFrames,
            CallerLabels,
            Continuation,
            <Fx as EmitTrace<OpReturn, Meta>>::OutputMeta,
            Fx,
        >,
    >;
}

impl<Stack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    Step<StepMachine<Stack, Locals, Memory, Frames, Labels, OpReturn, Rest, Meta, Fx>> for OpReturn
where
    Rest: IsList,
    Frames: ReturnStep<Stack, Locals, Memory, Labels, Rest, Meta, Fx>,
{
    type Output = <Frames as ReturnStep<Stack, Locals, Memory, Labels, Rest, Meta, Fx>>::Output;
}
