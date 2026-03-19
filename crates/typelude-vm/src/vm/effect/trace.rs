use typelude_std::std::col::array::{Array, IsList};

use crate::vm::state::VmMeta;
pub use crate::{
    shared::trace_event::TraceEvent,
    vm::algebra::effect::trace::{VmTrace, VmTraceEvent},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoTrace;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordTrace;

pub trait EmitTrace<Event, Meta> {
    type OutputMeta;
}

impl<Event, Log, Fuel, World, FuelPolicy, TrapPolicy, IoPolicy>
    EmitTrace<Event, VmMeta<Log, Fuel, World>>
    for super::Effects<NoTrace, FuelPolicy, TrapPolicy, IoPolicy>
{
    type OutputMeta = VmMeta<Log, Fuel, World>;
}

impl<Event, Log, Fuel, World, FuelPolicy, TrapPolicy, IoPolicy>
    EmitTrace<Event, VmMeta<Log, Fuel, World>>
    for super::Effects<RecordTrace, FuelPolicy, TrapPolicy, IoPolicy>
where
    Log: IsList,
{
    type OutputMeta = VmMeta<Array<Event, Log>, Fuel, World>;
}
