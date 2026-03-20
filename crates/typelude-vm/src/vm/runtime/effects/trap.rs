use crate::core::traits::MonadError;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VmTrap;

pub type ThrowVm<F, Reason, Trap = VmTrap> = <F as MonadError<Trap>>::Throw<Reason>;
