use core::marker::PhantomData;

use typelude_std::{
    core::TyFn,
    std::col::array::{Array, IsList},
};

use crate::{
    opcode::host::OpHostCall,
    vm::{
        protocol::request::HostRequest,
        semantics::{
            state::VmState,
            step::{StepInstr, StepSuspend},
        },
    },
};

pub struct LHostCall<Sig>(pub PhantomData<Sig>);

impl<Sig, Stack, Locals, Memory, Frames, Rest>
    TyFn<VmState<Stack, Locals, Memory, Frames, Array<OpHostCall<Sig>, Rest>>> for LHostCall<Sig>
where
    Rest: IsList,
{
    type Output =
        StepSuspend<HostRequest<Sig, Stack>, VmState<Stack, Locals, Memory, Frames, Rest>>;
}

impl<Sig, State> StepInstr<State> for OpHostCall<Sig>
where
    LHostCall<Sig>: TyFn<State>,
{
    type Output = <LHostCall<Sig> as TyFn<State>>::Output;
}
