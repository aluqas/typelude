use typenum::U0;

use crate::machine::{
    effects::{trap::RaiseTrap, Effects},
    machine::Machine,
    meta::VmMeta,
    result::Continue,
};

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
    type Output = Continue<M>;
}

impl<TracePolicy, TrapPolicy, IoPolicy, Core, Log, World>
    BeforeStep<Machine<Core, VmMeta<Log, U0, World>, Effects<TracePolicy, MeteredFuel, TrapPolicy, IoPolicy>>>
    for Effects<TracePolicy, MeteredFuel, TrapPolicy, IoPolicy>
where
    Effects<TracePolicy, MeteredFuel, TrapPolicy, IoPolicy>:
        RaiseTrap<
            OutOfFuel,
            Machine<Core, VmMeta<Log, U0, World>, Effects<TracePolicy, MeteredFuel, TrapPolicy, IoPolicy>>,
        >,
{
    type Output = <Effects<TracePolicy, MeteredFuel, TrapPolicy, IoPolicy> as RaiseTrap<
        OutOfFuel,
        Machine<Core, VmMeta<Log, U0, World>, Effects<TracePolicy, MeteredFuel, TrapPolicy, IoPolicy>>,
    >>::Output;
}
