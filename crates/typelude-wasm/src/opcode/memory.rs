use core::marker::PhantomData;

use typenum::U0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Load<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Store<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Load8U<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI32Store8<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Load<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpI64Store<MemArg = U0>(pub PhantomData<MemArg>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpMemorySize<MemoryIdx = U0>(pub PhantomData<MemoryIdx>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OpMemoryGrow<MemoryIdx = U0>(pub PhantomData<MemoryIdx>);
