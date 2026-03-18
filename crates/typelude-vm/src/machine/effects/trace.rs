use core::marker::PhantomData;

use typelude_std::std::col::array::{Array, IsList};

use crate::machine::{effects::Effects, meta::VmMeta};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoTrace;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordTrace;

#[derive(Debug)]
pub struct TraceEvent<Instr, Snapshot>(pub PhantomData<(Instr, Snapshot)>);

pub trait EmitTrace<Event, Meta> {
    type OutputMeta;
}

impl<Event, Log, Fuel, World, FuelPolicy, TrapPolicy, IoPolicy>
    EmitTrace<Event, VmMeta<Log, Fuel, World>>
    for Effects<NoTrace, FuelPolicy, TrapPolicy, IoPolicy>
{
    type OutputMeta = VmMeta<Log, Fuel, World>;
}

impl<Event, Log, Fuel, World, FuelPolicy, TrapPolicy, IoPolicy>
    EmitTrace<Event, VmMeta<Log, Fuel, World>>
    for Effects<RecordTrace, FuelPolicy, TrapPolicy, IoPolicy>
where
    Log: IsList,
{
    type OutputMeta = VmMeta<Array<Event, Log>, Fuel, World>;
}
