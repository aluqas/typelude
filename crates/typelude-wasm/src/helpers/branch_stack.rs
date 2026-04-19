use typelude_col::TArr;
use typelude_std::core::Sub;
use typenum::{B1, U1, UInt, UTerm};

use crate::{
    frame::{BranchBlock, BranchLoop, ResolvedBlock, ResolvedLoop},
    state::WasmState,
};

/// branch stack から深さ `Depth` の分岐先を解決する helper。
pub trait ResolveBranch<Depth> {
    type Output;
}

/// `br_table` の index から実際の分岐深さを選ぶ helper。
pub trait SelectBrTableTarget<Default, Index> {
    type Output;
}

impl<Default, Index> SelectBrTableTarget<Default, Index> for typelude_col::TTerm {
    type Output = Default;
}

impl<Head, Tail, Default> SelectBrTableTarget<Default, UTerm> for TArr<Head, Tail> {
    type Output = Head;
}

impl<Head, Tail, Default, N, B> SelectBrTableTarget<Default, UInt<N, B>> for TArr<Head, Tail>
where
    UInt<N, B>: Sub<U1>,
    Tail: SelectBrTableTarget<Default, <UInt<N, B> as Sub<U1>>::Output>,
{
    type Output = <Tail as SelectBrTableTarget<Default, <UInt<N, B> as Sub<U1>>::Output>>::Output;
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

/// 解決済み branch target を新しい `WasmState` へ反映する helper。
pub trait BranchJump<Module, Store, Stack, Locals, Frames> {
    type Output;
}

impl<Continuation, RemainingBranches, Module, Store, Stack, Locals, Frames>
    BranchJump<Module, Store, Stack, Locals, Frames>
    for ResolvedBlock<Continuation, RemainingBranches>
{
    type Output = WasmState<Module, Store, Stack, Locals, Frames, RemainingBranches, Continuation>;
}

impl<LoopProgram, RemainingBranches, Module, Store, Stack, Locals, Frames>
    BranchJump<Module, Store, Stack, Locals, Frames>
    for ResolvedLoop<LoopProgram, RemainingBranches>
{
    type Output = WasmState<Module, Store, Stack, Locals, Frames, RemainingBranches, LoopProgram>;
}

#[doc(hidden)]
pub trait ContinueIfZero<Resolved, Module, Store, Stack, Locals, Frames, Branches, Rest> {
    type Output;
}

impl<Resolved, Module, Store, Stack, Locals, Frames, Branches, Rest>
    ContinueIfZero<Resolved, Module, Store, Stack, Locals, Frames, Branches, Rest> for B1
{
    type Output = WasmState<Module, Store, Stack, Locals, Frames, Branches, Rest>;
}

impl<Resolved, Module, Store, Stack, Locals, Frames, Branches, Rest>
    ContinueIfZero<Resolved, Module, Store, Stack, Locals, Frames, Branches, Rest> for typenum::B0
where
    Resolved: BranchJump<Module, Store, Stack, Locals, Frames>,
{
    type Output = <Resolved as BranchJump<Module, Store, Stack, Locals, Frames>>::Output;
}
