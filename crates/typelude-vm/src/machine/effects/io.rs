use core::marker::PhantomData;

use crate::machine::{effects::Effects, result::Suspend};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SuspendIoPolicy;

#[derive(Debug)]
pub struct HostRequest<Sig, Args>(pub PhantomData<(Sig, Args)>);

pub trait SuspendIo<Request, M> {
    type Output;
}

impl<Request, M, TracePolicy, FuelPolicy, TrapPolicy> SuspendIo<Request, M>
    for Effects<TracePolicy, FuelPolicy, TrapPolicy, SuspendIoPolicy>
{
    type Output = Suspend<Request, M>;
}
