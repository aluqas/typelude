use core::marker::PhantomData;

use crate::{core::traits::MonadWriter, vm::protocol::trace_event::TraceEvent};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VmTrace;

pub type VmTraceEvent<Instr> = TraceEvent<Instr, ()>;
pub type PushTrace<F, Instr, Trace = VmTrace> =
    <F as MonadWriter<Trace>>::Tell<VmTraceEvent<Instr>>;

#[derive(Debug)]
pub struct LogTrace<Event>(pub PhantomData<Event>);
