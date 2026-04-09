//! Runtime trace channels.

use core::marker::PhantomData;

use typelude_col::{TArr, TTerm};
use typelude_std::{
    core::Value,
    effect::{
        Append, EitherLift, EitherT, Empty, IdK, MonadWriter, StateLift, StateT, SuspendLift,
        WriterLift, WriterT,
    },
};

use crate::vm::{
    protocol::trace_event::TraceEvent,
    runtime::effects::{io::VmRequest, trap::VmTrap},
};

/// Surface opcode trace channel.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VmSourceTrace;

/// Lowered/core execution trace channel.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VmCoreTrace;

/// Bundles the two trace streams carried by a completed runtime outcome.
#[derive(Debug)]
pub struct TraceBundle<Source, Core>(pub PhantomData<(Source, Core)>);

pub type SourceTraceEvent<Instr> = TraceEvent<Instr, ()>;
pub type CoreTraceEvent<Instr> = TraceEvent<Instr, ()>;
pub type SourceTraceChunk<Instr> = TArr<SourceTraceEvent<Instr>, TTerm>;
pub type CoreTraceChunk<Instr> = TArr<CoreTraceEvent<Instr>, TTerm>;

pub type PushSourceTrace<F, Instr, Trace = VmSourceTrace> =
    <F as MonadWriter<Trace>>::Tell<SourceTraceChunk<Instr>>;

pub type PushCoreTrace<
    State,
    Instr,
    SourceTrace = VmSourceTrace,
    CoreTrace = VmCoreTrace,
    Trap = VmTrap,
    Req = VmRequest,
> = SuspendLift<
    Req,
    EitherT<Trap, StateT<State, WriterT<SourceTrace, WriterT<CoreTrace, IdK>>>>,
    EitherLift<
        Trap,
        StateT<State, WriterT<SourceTrace, WriterT<CoreTrace, IdK>>>,
        StateLift<
            State,
            WriterT<SourceTrace, WriterT<CoreTrace, IdK>>,
            WriterLift<
                SourceTrace,
                WriterT<CoreTrace, IdK>,
                <WriterT<CoreTrace, IdK> as MonadWriter<CoreTrace>>::Tell<CoreTraceChunk<Instr>>,
            >,
        >,
    >,
>;

impl Empty for VmSourceTrace {
    type Output = TTerm;
}

impl Empty for VmCoreTrace {
    type Output = TTerm;
}

/// Concatenates a newer trace bundle onto an existing trace bundle during
/// resume.
pub trait AppendTraceBundle<Other> {
    type Output;
}

impl<SourceA, CoreA, SourceB, CoreB> AppendTraceBundle<TraceBundle<SourceB, CoreB>>
    for TraceBundle<SourceA, CoreA>
where
    SourceA: Append<SourceB>,
    CoreA: Append<CoreB>,
{
    type Output =
        TraceBundle<<SourceA as Append<SourceB>>::Output, <CoreA as Append<CoreB>>::Output>;
}

impl<Source, Core> Value for TraceBundle<Source, Core> {}

#[derive(Debug)]
pub struct LogTrace<Event>(pub PhantomData<Event>);
