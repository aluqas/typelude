pub mod capability;
pub mod fuel;
pub mod log;
pub mod trace;
pub mod world;

use core::marker::PhantomData;

pub use io::{HostRequest, SuspendIo, SuspendIoPolicy};
pub use log::VmLog;
pub use trace::{EmitTrace, NoTrace, RecordTrace, VmTraceEvent};
pub use trap::{RaiseTrap, TrapAsResult};
use typelude_std::core::Eval;

mod io {
    pub use crate::shared::request::HostRequest;

    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SuspendIoPolicy;

    pub trait SuspendIo<Request, M> {
        type Output;
    }

    impl<Request, M, TracePolicy, FuelPolicy, TrapPolicy> SuspendIo<Request, M>
        for super::Effects<TracePolicy, FuelPolicy, TrapPolicy, SuspendIoPolicy>
    {
        type Output = crate::vm::step::Suspend<Request, M>;
    }
}

mod trap {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct TrapAsResult;

    pub trait RaiseTrap<Reason, M> {
        type Output;
    }

    impl<Reason, M, TracePolicy, FuelPolicy, IoPolicy> RaiseTrap<Reason, M>
        for super::Effects<TracePolicy, FuelPolicy, TrapAsResult, IoPolicy>
    {
        type Output = crate::vm::step::Trap<Reason, M>;
    }
}

#[derive(Debug)]
pub struct Effects<TracePolicy, FuelPolicy, TrapPolicy, IoPolicy>(
    pub PhantomData<(TracePolicy, FuelPolicy, TrapPolicy, IoPolicy)>,
);

impl<T, F, R, I> Eval for Effects<T, F, R, I> {
    type Output = Self;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IgnoreFuel;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MeteredFuel;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutOfFuel;

pub trait BeforeStep<M> {
    type Output;
}

impl<TracePolicy, TrapPolicy, IoPolicy, M> BeforeStep<M>
    for Effects<TracePolicy, IgnoreFuel, TrapPolicy, IoPolicy>
{
    type Output = crate::vm::step::Continue<M>;
}

impl<TracePolicy, TrapPolicy, IoPolicy, Core, Log, World>
    BeforeStep<
        crate::vm::run::direct::Machine<
            Core,
            crate::vm::state::VmMeta<Log, typenum::U0, World>,
            Effects<TracePolicy, MeteredFuel, TrapPolicy, IoPolicy>,
        >,
    > for Effects<TracePolicy, MeteredFuel, TrapPolicy, IoPolicy>
where
    Effects<TracePolicy, MeteredFuel, TrapPolicy, IoPolicy>: RaiseTrap<
            OutOfFuel,
            crate::vm::run::direct::Machine<
                Core,
                crate::vm::state::VmMeta<Log, typenum::U0, World>,
                Effects<TracePolicy, MeteredFuel, TrapPolicy, IoPolicy>,
            >,
        >,
{
    type Output = <Effects<TracePolicy, MeteredFuel, TrapPolicy, IoPolicy> as RaiseTrap<
        OutOfFuel,
        crate::vm::run::direct::Machine<
            Core,
            crate::vm::state::VmMeta<Log, typenum::U0, World>,
            Effects<TracePolicy, MeteredFuel, TrapPolicy, IoPolicy>,
        >,
    >>::Output;
}

pub type PureEffects = Effects<NoTrace, IgnoreFuel, TrapAsResult, SuspendIoPolicy>;
pub type TraceEffects = Effects<RecordTrace, IgnoreFuel, TrapAsResult, SuspendIoPolicy>;
