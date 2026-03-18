use core::marker::PhantomData;

#[derive(Debug)]
pub struct HostRequest<Sig, Args>(pub PhantomData<(Sig, Args)>);
