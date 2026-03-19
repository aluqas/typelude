use crate::core::MonadError;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VmTrap;

pub type ThrowVm<F, Reason> = <F as MonadError<VmTrap>>::Throw<Reason>;

pub use crate::shared::trap::{
    BadLocalIndex, BadMemoryIndex, InvalidCondition, LocalUnderflow, ReturnUnderflow,
    StackUnderflow,
};
