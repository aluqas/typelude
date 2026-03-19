use typelude_std::std::col::array::Nil;

use crate::core::{
    either_t::EitherT,
    id::IdK,
    state_t::StateT,
    suspend_t::SuspendT,
    traits::Unit,
    writer_t::WriterT,
};
use crate::vm::algebra::effect::{io::VmRequest, trace::VmTrace, trap::VmTrap};

pub type VmFx<State, Trace = VmTrace, Trap = VmTrap, Req = VmRequest> =
    SuspendT<Req, EitherT<Trap, StateT<State, WriterT<Trace, IdK>>>>;

pub type EmptyTrace = Nil;
pub type VmUnit<F> = crate::core::traits::Pure<F, Unit>;
