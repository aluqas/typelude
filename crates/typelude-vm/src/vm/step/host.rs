use typelude_std::std::col::array::IsList;

use crate::{
    opcode::OpHostCall,
    vm::{
        effect::{EmitTrace, SuspendIo},
        step::{CoreMachine, Step, StepMachine},
    },
};

impl<Sig, Stack, Locals, Memory, Frames, Labels, Rest, Meta, Fx>
    Step<StepMachine<Stack, Locals, Memory, Frames, Labels, OpHostCall<Sig>, Rest, Meta, Fx>>
    for OpHostCall<Sig>
where
    Rest: IsList,
    Fx: EmitTrace<OpHostCall<Sig>, Meta>
        + SuspendIo<
            crate::shared::request::HostRequest<Sig, Stack>,
            CoreMachine<
                Stack,
                Locals,
                Memory,
                Frames,
                Labels,
                Rest,
                <Fx as EmitTrace<OpHostCall<Sig>, Meta>>::OutputMeta,
                Fx,
            >,
        >,
{
    type Output = <Fx as SuspendIo<
        crate::shared::request::HostRequest<Sig, Stack>,
        CoreMachine<
            Stack,
            Locals,
            Memory,
            Frames,
            Labels,
            Rest,
            <Fx as EmitTrace<OpHostCall<Sig>, Meta>>::OutputMeta,
            Fx,
        >,
    >>::Output;
}
