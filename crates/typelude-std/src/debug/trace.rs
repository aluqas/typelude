use core::marker::PhantomData;

use typelude_std::core::Eval;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TraceTag;

#[derive(Debug)]
pub struct TraceRecord<Event>(pub PhantomData<Event>);

#[derive(Debug)]
pub struct TraceBundle<Source, Core>(pub PhantomData<(Source, Core)>);

impl<Event> Eval for TraceRecord<Event> {
    type Output = Self;
}

impl<Source, Core> Eval for TraceBundle<Source, Core> {
    type Output = Self;
}
