//! Host instruction semantics.
//!
//! `OpHostCall` preserves the current stack as request arguments, commits the
//! advanced program state, and suspends. Resume pushes the host response onto
//! the stack before continuing from the remaining program.

use core::marker::PhantomData;

use typelude_std::{
    core::TyFn,
    std::col::array::{Array, IsList},
};

use crate::{
    opcode::host::OpHostCall,
    vm::{
        protocol::request::{HostRequest, HostSignature},
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
    Sig: HostSignature,
    Rest: IsList,
{
    type Output = StepSuspend<
        HostRequest<Sig, Stack, <Sig as HostSignature>::Response>,
        VmState<Stack, Locals, Memory, Frames, Rest>,
    >;
}

impl<Sig, State> StepInstr<State> for OpHostCall<Sig>
where
    LHostCall<Sig>: TyFn<State>,
{
    type Output = <LHostCall<Sig> as TyFn<State>>::Output;
}
