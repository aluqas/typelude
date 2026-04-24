use typelude_std::effect::MonadSuspend;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VmRequest;

pub type YieldVm<F, Request, Req = VmRequest> = <F as MonadSuspend<Req>>::Suspend<Request>;
