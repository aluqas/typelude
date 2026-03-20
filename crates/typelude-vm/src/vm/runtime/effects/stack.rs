use typelude_std::std::col::array::Nil;

use crate::{
    core::{
        either_t::EitherT, id::IdK, state_t::StateT, suspend_t::SuspendT, traits::Unit,
        writer_t::WriterT,
    },
    vm::runtime::effects::{
        io::VmRequest,
        trace::{VmCoreTrace, VmSourceTrace},
        trap::VmTrap,
    },
};

pub type VmFx<
    State,
    SourceTrace = VmSourceTrace,
    CoreTrace = VmCoreTrace,
    Trap = VmTrap,
    Req = VmRequest,
> = SuspendT<Req, EitherT<Trap, StateT<State, WriterT<SourceTrace, WriterT<CoreTrace, IdK>>>>>;

pub type EmptyTrace = Nil;
pub type VmUnit<F> = crate::core::traits::Pure<F, Unit>;
