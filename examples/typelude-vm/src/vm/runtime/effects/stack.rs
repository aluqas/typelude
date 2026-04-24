use typelude_col::TTerm;
use typelude_std::effect::{EitherT, IdK, Pure, StateT, SuspendT, Unit, WriterT};

use crate::vm::runtime::effects::{
    io::VmRequest,
    trace::{VmCoreTrace, VmSourceTrace},
    trap::VmTrap,
};

pub type VmFx<
    State,
    SourceTrace = VmSourceTrace,
    CoreTrace = VmCoreTrace,
    Trap = VmTrap,
    Req = VmRequest,
> = SuspendT<Req, EitherT<Trap, StateT<State, WriterT<SourceTrace, WriterT<CoreTrace, IdK>>>>>;

pub type EmptyTrace = TTerm;
pub type VmUnit<F> = Pure<F, Unit>;
