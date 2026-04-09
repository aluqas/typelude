use core::marker::PhantomData;

use typelude_std::core::Value;

#[derive(Debug, Default)]
pub struct ReturnFrame<ReturnLocals, ReturnBranches, ReturnProgram>(
    pub PhantomData<(ReturnLocals, ReturnBranches, ReturnProgram)>,
);

impl<ReturnLocals, ReturnBranches, ReturnProgram> Value
    for ReturnFrame<ReturnLocals, ReturnBranches, ReturnProgram>
{
}

#[doc(hidden)]
#[derive(Debug, Default)]
pub struct BranchBlock<Continuation>(pub PhantomData<Continuation>);

impl<Continuation> Value for BranchBlock<Continuation> {}

#[doc(hidden)]
#[derive(Debug, Default)]
pub struct BranchLoop<LoopProgram>(pub PhantomData<LoopProgram>);

impl<LoopProgram> Value for BranchLoop<LoopProgram> {}

#[doc(hidden)]
#[derive(Debug, Default)]
pub struct ResolvedBlock<Continuation, RemainingBranches>(
    pub PhantomData<(Continuation, RemainingBranches)>,
);

impl<Continuation, RemainingBranches> Value for ResolvedBlock<Continuation, RemainingBranches> {}

#[doc(hidden)]
#[derive(Debug, Default)]
pub struct ResolvedLoop<LoopProgram, RemainingBranches>(
    pub PhantomData<(LoopProgram, RemainingBranches)>,
);

impl<LoopProgram, RemainingBranches> Value for ResolvedLoop<LoopProgram, RemainingBranches> {}
