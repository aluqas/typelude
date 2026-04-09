use typelude_col::TArr;
use typelude_std::core::Sub;
use typenum::{B1, U1, UInt, UTerm};

use crate::{
    frame::{BranchBlock, BranchLoop, ResolvedBlock, ResolvedLoop},
    state::WasmState,
};

pub trait ResolveBranch<Depth> {
    type Output;
}

impl<Continuation, Tail> ResolveBranch<UTerm> for TArr<BranchBlock<Continuation>, Tail> {
    type Output = ResolvedBlock<Continuation, Tail>;
}

impl<LoopProgram, Tail> ResolveBranch<UTerm> for TArr<BranchLoop<LoopProgram>, Tail> {
    type Output = ResolvedLoop<LoopProgram, TArr<BranchLoop<LoopProgram>, Tail>>;
}

impl<Head, Tail, N, B> ResolveBranch<UInt<N, B>> for TArr<Head, Tail>
where
    UInt<N, B>: Sub<U1>,
    Tail: ResolveBranch<<UInt<N, B> as Sub<U1>>::Output>,
{
    type Output = <Tail as ResolveBranch<<UInt<N, B> as Sub<U1>>::Output>>::Output;
}

pub trait BranchJump<Module, Stack, Locals, Memory, Frames> {
    type Output;
}

impl<Continuation, RemainingBranches, Module, Stack, Locals, Memory, Frames>
    BranchJump<Module, Stack, Locals, Memory, Frames>
    for ResolvedBlock<Continuation, RemainingBranches>
{
    type Output =
        WasmState<Module, Stack, Locals, Memory, Frames, RemainingBranches, Continuation>;
}

impl<LoopProgram, RemainingBranches, Module, Stack, Locals, Memory, Frames>
    BranchJump<Module, Stack, Locals, Memory, Frames>
    for ResolvedLoop<LoopProgram, RemainingBranches>
{
    type Output = WasmState<Module, Stack, Locals, Memory, Frames, RemainingBranches, LoopProgram>;
}

#[doc(hidden)]
pub trait ContinueIfZero<Resolved, Module, Stack, Locals, Memory, Frames, Branches, Rest> {
    type Output;
}

impl<Resolved, Module, Stack, Locals, Memory, Frames, Branches, Rest>
    ContinueIfZero<Resolved, Module, Stack, Locals, Memory, Frames, Branches, Rest> for B1
{
    type Output = WasmState<Module, Stack, Locals, Memory, Frames, Branches, Rest>;
}

impl<Resolved, Module, Stack, Locals, Memory, Frames, Branches, Rest>
    ContinueIfZero<Resolved, Module, Stack, Locals, Memory, Frames, Branches, Rest> for typenum::B0
where
    Resolved: BranchJump<Module, Stack, Locals, Memory, Frames>,
{
    type Output = <Resolved as BranchJump<Module, Stack, Locals, Memory, Frames>>::Output;
}
