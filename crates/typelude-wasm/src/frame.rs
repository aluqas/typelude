use core::marker::PhantomData;

use typelude_std::core::Value;

/// 関数呼び出しから戻るための return frame。
///
/// 関数本体の評価前に frame stack へ積まれ、`return` や `OpEndFunc`
/// 到達時に復元すべき locals / branches / continuation program を保持します。
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
/// block 分岐先を表す内部マーカー。
pub struct BranchBlock<Continuation>(pub PhantomData<Continuation>);

impl<Continuation> Value for BranchBlock<Continuation> {}

#[doc(hidden)]
#[derive(Debug, Default)]
/// loop 分岐先を表す内部マーカー。
pub struct BranchLoop<LoopProgram>(pub PhantomData<LoopProgram>);

impl<LoopProgram> Value for BranchLoop<LoopProgram> {}

#[doc(hidden)]
#[derive(Debug, Default)]
/// branch 解決後の block 継続情報。
pub struct ResolvedBlock<Continuation, RemainingBranches>(
    pub PhantomData<(Continuation, RemainingBranches)>,
);

impl<Continuation, RemainingBranches> Value for ResolvedBlock<Continuation, RemainingBranches> {}

#[doc(hidden)]
#[derive(Debug, Default)]
/// branch 解決後の loop 継続情報。
pub struct ResolvedLoop<LoopProgram, RemainingBranches>(
    pub PhantomData<(LoopProgram, RemainingBranches)>,
);

impl<LoopProgram, RemainingBranches> Value for ResolvedLoop<LoopProgram, RemainingBranches> {}
