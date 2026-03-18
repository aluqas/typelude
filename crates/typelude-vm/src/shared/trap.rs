use core::marker::PhantomData;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StackUnderflow;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalUnderflow;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReturnUnderflow;

#[derive(Debug)]
pub struct BadLocalIndex<Idx>(pub PhantomData<Idx>);

#[derive(Debug)]
pub struct BadMemoryIndex<Idx>(pub PhantomData<Idx>);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InvalidCondition;
