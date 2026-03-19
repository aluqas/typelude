pub use crate::vm::algebra::{
    outcome::{Done, Raised, Suspended},
    program::{EInterpProgram, InterpProgram},
    run::{ERunVm, ERunVmStack},
    state::VmState as ComposedVmState,
};

pub type DefaultVmFx<
    State,
    Trace = crate::vm::algebra::effect::trace::VmTrace,
    Trap = crate::vm::algebra::effect::trap::VmTrap,
    Req = crate::vm::algebra::effect::io::VmRequest,
> = crate::vm::algebra::effect::stack::VmFx<State, Trace, Trap, Req>;

pub type HostRequest<Sig> = crate::shared::request::HostRequest<Sig, ()>;
