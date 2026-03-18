use core::marker::PhantomData;

#[derive(Debug)]
pub struct TraceEvent<Instr, Snapshot>(pub PhantomData<(Instr, Snapshot)>);
