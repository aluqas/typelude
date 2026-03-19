use core::marker::PhantomData;

use crate::core::traits::MonadWriter;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VmTrace;

pub type VmTraceEvent<Instr> = crate::shared::trace_event::TraceEvent<Instr, ()>;
pub type PushTrace<F, Instr> = <F as MonadWriter<VmTrace>>::Tell<VmTraceEvent<Instr>>;

#[derive(Debug)]
pub struct LogTrace<Event>(pub PhantomData<Event>);
