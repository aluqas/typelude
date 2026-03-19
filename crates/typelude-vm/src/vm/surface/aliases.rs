use typelude_std::std::col::array::Nil;

use crate::{
    vm::{
        run::direct::Machine,
        effect::{
            Effects, MeteredFuel, NoTrace, PureEffects, SuspendIoPolicy, TraceEffects,
            TrapAsResult, world::NoWorld,
        },
        state::{DefaultMeta, NoFuel, VmMeta, VmState},
    },
};

pub type PureVm<Stack, Locals, Memory, Frames, Labels = Nil, Program = Nil> =
    Machine<VmState<Stack, Locals, Memory, Frames, Labels, Program>, DefaultMeta, PureEffects>;

pub type ProgramVm<Program> = PureVm<Nil, Nil, Nil, Nil, Nil, Program>;

pub type TracedVm<Stack, Locals, Memory, Frames, Labels = Nil, Program = Nil, History = Nil> =
    Machine<
        VmState<Stack, Locals, Memory, Frames, Labels, Program>,
        VmMeta<History, NoFuel, NoWorld>,
        TraceEffects,
    >;

pub type MeteredVm<
    Stack,
    Locals,
    Memory,
    Frames,
    Labels = Nil,
    Program = Nil,
    Log = Nil,
    Fuel = typenum::U0,
    World = NoWorld,
> = Machine<
    VmState<Stack, Locals, Memory, Frames, Labels, Program>,
    VmMeta<Log, Fuel, World>,
    Effects<NoTrace, MeteredFuel, TrapAsResult, SuspendIoPolicy>,
>;
