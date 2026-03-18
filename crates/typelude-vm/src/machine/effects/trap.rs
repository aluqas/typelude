use crate::machine::{effects::Effects, result::Trap};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrapAsResult;

pub trait RaiseTrap<Reason, M> {
    type Output;
}

impl<Reason, M, TracePolicy, FuelPolicy, IoPolicy> RaiseTrap<Reason, M>
    for Effects<TracePolicy, FuelPolicy, TrapAsResult, IoPolicy>
{
    type Output = Trap<Reason, M>;
}
