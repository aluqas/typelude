use core::marker::PhantomData;

use typelude_std::core::Value;

#[derive(Debug)]
pub struct LogRecord<Event>(pub PhantomData<Event>);

#[derive(Debug)]
pub struct LogTrace<Event>(pub PhantomData<Event>);

impl<Event> Value for LogRecord<Event> {}
impl<Event> Value for LogTrace<Event> {}
