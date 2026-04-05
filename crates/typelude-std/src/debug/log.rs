use core::marker::PhantomData;

use typelude_std::core::Eval;

#[derive(Debug)]
pub struct LogRecord<Event>(pub PhantomData<Event>);

#[derive(Debug)]
pub struct LogTrace<Event>(pub PhantomData<Event>);

impl<Event> Eval for LogRecord<Event> {
    type Output = Self;
}

impl<Event> Eval for LogTrace<Event> {
    type Output = Self;
}
