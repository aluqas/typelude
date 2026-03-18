use typelude_std::std::col::array::Nil;

use super::{VmRequest, VmTrace, VmTrap};
use crate::composed::core::{EitherT, IdK, StateT, SuspendT, Unit, WriterT};

pub type VmFx<State, Trace = VmTrace, Trap = VmTrap, Req = VmRequest> =
    SuspendT<Req, EitherT<Trap, StateT<State, WriterT<Trace, IdK>>>>;

pub type EmptyTrace = Nil;
pub type VmUnit<F> = crate::composed::core::Pure<F, Unit>;
