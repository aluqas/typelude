//! Host instruction semantics.
//!
//! `OpHostCall` preserves the current stack as request arguments, commits the
//! advanced program state, and suspends. Resume pushes the host response onto
//! the stack before continuing from the remaining program.

use typelude_col::TArr;
use typelude_std::core::Op;

use crate::{
    opcode::host::OpHostCall,
    vm::{
        protocol::request::{HostRequest, HostSignature},
        semantics::{state::VmState, step::StepSuspend},
    },
};

impl<Sig, Stack, Locals, Memory, Frames, Rest>
    Op<VmState<Stack, Locals, Memory, Frames, TArr<OpHostCall<Sig>, Rest>>> for OpHostCall<Sig>
where
    Sig: HostSignature,
{
    type Output = StepSuspend<
        HostRequest<Sig, Stack, <Sig as HostSignature>::Response>,
        VmState<Stack, Locals, Memory, Frames, Rest>,
    >;
}
