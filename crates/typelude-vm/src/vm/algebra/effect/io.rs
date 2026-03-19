use crate::core::traits::MonadSuspend;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VmRequest;

pub type YieldVm<F, Request> = <F as MonadSuspend<VmRequest>>::Suspend<Request>;
