//! Trace event protocol shared by surface and lowered core traces.

use core::marker::PhantomData;

/// A single trace entry.
///
/// `Instr` identifies the executed instruction and `Snapshot` is reserved for
/// future enriched trace payloads. The current runtime uses `()`.
#[derive(Debug)]
pub struct TraceEvent<Instr, Snapshot>(pub PhantomData<(Instr, Snapshot)>);
