use core::marker::PhantomData;

use typelude_std::core::Value;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TraceTag;

#[derive(Debug)]
pub struct TraceRecord<Event>(pub PhantomData<Event>);

#[derive(Debug)]
pub struct TraceBundle<Source, Core>(pub PhantomData<(Source, Core)>);

impl Value for TraceTag {}
impl<Event> Value for TraceRecord<Event> {}
impl<Source, Core> Value for TraceBundle<Source, Core> {}
