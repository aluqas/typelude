//! Runtime trace channels.

use core::marker::PhantomData;

use typelude_std::{
    core::Eval,
    std::col::array::{Concat, IsList},
};

use crate::{
    core::{
        either_t::{EitherLift, EitherT},
        id::IdK,
        state_t::{StateLift, StateT},
        suspend_t::{SuspendLift, SuspendT},
        traits::MonadWriter,
        writer_t::{WriterLift, WriterT},
    },
    vm::{
        protocol::trace_event::TraceEvent,
        runtime::effects::{io::VmRequest, trap::VmTrap},
    },
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
pub type PushSourceTrace<F, Instr, Trace = VmSourceTrace> =
    <F as MonadWriter<Trace>>::Tell<SourceTraceEvent<Instr>>;
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
                <WriterT<CoreTrace, IdK> as MonadWriter<CoreTrace>>::Tell<CoreTraceEvent<Instr>>,
            >,
        >,
    >,
>;

/// Concatenates a newer trace bundle onto an existing trace bundle during
/// resume.
pub trait AppendTraceBundle<Other> {
    type Output;
}

impl<SourceA, CoreA, SourceB, CoreB> AppendTraceBundle<TraceBundle<SourceB, CoreB>>
    for TraceBundle<SourceA, CoreA>
where
    SourceA: Concat<SourceB> + IsList,
    SourceB: IsList,
    CoreA: Concat<CoreB> + IsList,
    CoreB: IsList,
{
    type Output =
        TraceBundle<<SourceA as Concat<SourceB>>::Output, <CoreA as Concat<CoreB>>::Output>;
}

impl<Source, Core> Eval for TraceBundle<Source, Core> {
    type Output = Self;
}

#[derive(Debug)]
pub struct LogTrace<Event>(pub PhantomData<Event>);
