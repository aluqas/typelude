use core::marker::PhantomData;

use crate::composed::core::MonadWriter;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VmLog;

#[derive(Debug)]
pub struct LogEntry<Tag, Msg>(pub PhantomData<(Tag, Msg)>);

pub type PushLog<F, Tag, Msg> = <F as MonadWriter<VmLog>>::Tell<LogEntry<Tag, Msg>>;
