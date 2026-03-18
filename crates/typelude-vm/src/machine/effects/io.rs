use crate::machine::{effects::Effects, result::Suspend};
pub use crate::shared::request::HostRequest;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SuspendIoPolicy;

pub trait SuspendIo<Request, M> {
    type Output;
}

impl<Request, M, TracePolicy, FuelPolicy, TrapPolicy> SuspendIo<Request, M>
    for Effects<TracePolicy, FuelPolicy, TrapPolicy, SuspendIoPolicy>
{
    type Output = Suspend<Request, M>;
}
