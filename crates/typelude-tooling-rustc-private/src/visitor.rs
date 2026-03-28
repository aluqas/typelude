#![allow(unused_extern_crates)]

extern crate rustc_infer;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_trait_selection;

use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::traits::solve::Goal;
use rustc_middle::ty::{self, Clause, TyCtxt, TypingMode, Upcast};
use rustc_trait_selection::solve::inspect::{
    InferCtxtProofTreeExt, InspectCandidate, InspectConfig, InspectGoal, ProofTreeVisitor,
};
use typelude_tooling_core::{HookId, TraceEventKind};

use crate::{
    collector::{HookContext, ItemContext},
    output::TraceEventExt,
};

pub fn collect_for_item<'tcx>(
    tcx: TyCtxt<'tcx>,
    ctx: &mut HookContext<'_>,
    item_ctx: &ItemContext,
    predicates: impl Iterator<Item = (Clause<'tcx>, rustc_span::Span)>,
) -> Result<(), String> {
    let infcx = tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let mut visitor = ProofTreeCollector::new(ctx, item_ctx);

    for (clause, _span) in predicates {
        if !visitor.ctx.can_emit() {
            visitor.ctx.stats.dropped_count += 1;
            break;
        }
        let goal = Goal {
            param_env: ty::ParamEnv::empty(),
            predicate: clause.upcast(tcx),
        };
        infcx.probe(|_| {
            infcx.visit_proof_tree(goal, &mut visitor);
        });
    }

    Ok(())
}

struct ProofTreeCollector<'v, 'c> {
    ctx: &'v mut HookContext<'c>,
    item_ctx: &'v ItemContext,
    stack: Vec<typelude_tooling_core::GoalId>,
}

impl<'v, 'c> ProofTreeCollector<'v, 'c> {
    fn new(ctx: &'v mut HookContext<'c>, item_ctx: &'v ItemContext) -> Self {
        Self {
            ctx,
            item_ctx,
            stack: Vec::new(),
        }
    }

    fn visit_candidate_inner(&mut self, candidate: &InspectCandidate<'_, '_>) {
        if !self.ctx.can_emit() {
            self.ctx.stats.dropped_count += 1;
            return;
        }
        let Some(goal_id) = self.stack.last().copied() else {
            return;
        };

        let candidate_id = self.ctx.alloc_candidate_id();
        self.ctx.stats.candidate_count += 1;

        let discovered = self
            .ctx
            .emitter
            .emit(TraceEventKind::CandidateDiscovered, format!("{:?}", candidate.kind()))
            .with_hook_id(HookId::TraitSolve)
            .with_item_id(self.item_ctx.item_id)
            .with_goal_id(goal_id)
            .with_candidate_id(candidate_id);
        self.ctx.emitter.write(&discovered);

        let tried = self
            .ctx
            .emitter
            .emit(TraceEventKind::CandidateTried, format!("{:?}", candidate.kind()))
            .with_hook_id(HookId::TraitSolve)
            .with_item_id(self.item_ctx.item_id)
            .with_goal_id(goal_id)
            .with_candidate_id(candidate_id);
        self.ctx.emitter.write(&tried);

        let result = self
            .ctx
            .emitter
            .emit(TraceEventKind::CandidateResult, format!("{:?}", candidate.kind()))
            .with_hook_id(HookId::TraitSolve)
            .with_item_id(self.item_ctx.item_id)
            .with_goal_id(goal_id)
            .with_candidate_id(candidate_id)
            .with_detail(format!("{:?}", candidate.result()));
        self.ctx.emitter.write(&result);

        candidate.visit_nested_no_probe(self);
    }
}

impl<'tcx> ProofTreeVisitor<'tcx> for ProofTreeCollector<'_, '_> {
    type Result = ();

    fn span(&self) -> rustc_span::Span {
        rustc_span::DUMMY_SP
    }

    fn config(&self) -> InspectConfig {
        InspectConfig {
            max_depth: self.ctx.config.max_depth,
        }
    }

    fn visit_goal(&mut self, goal: &InspectGoal<'_, 'tcx>) -> Self::Result {
        let predicate = format!("{:?}", goal.goal().predicate);
        if !self.ctx.focus_matches(&predicate) {
            return;
        }
        if !self.ctx.can_emit() {
            self.ctx.stats.dropped_count += 1;
            return;
        }

        let goal_id = self.ctx.alloc_goal_id();
        let parent_goal_id = self.stack.last().copied();
        let candidates = goal.candidates();
        self.ctx.stats.goal_count += 1;

        let discovered = self
            .ctx
            .emitter
            .emit(TraceEventKind::GoalDiscovered, predicate.clone())
            .with_hook_id(HookId::TraitSolve)
            .with_item_id(self.item_ctx.item_id)
            .with_goal_id(goal_id)
            .with_metadata("candidate_count", candidates.len().to_string())
            .with_metadata("collection_point", self.item_ctx.collection_point.as_str())
            .with_metadata("item_kind", self.item_ctx.item_kind.clone())
            .with_metadata("item_path", self.item_ctx.item_path.clone());
        let discovered = if let Some(parent) = parent_goal_id {
            discovered.with_parent_goal_id(parent)
        } else {
            discovered
        };
        self.ctx.emitter.write(&discovered);

        let entered = self
            .ctx
            .emitter
            .emit(TraceEventKind::GoalEntered, predicate)
            .with_hook_id(HookId::TraitSolve)
            .with_item_id(self.item_ctx.item_id)
            .with_goal_id(goal_id);
        let entered = if let Some(parent) = parent_goal_id {
            entered.with_parent_goal_id(parent)
        } else {
            entered
        };
        self.ctx.emitter.write(&entered);

        self.stack.push(goal_id);
        for candidate in &candidates {
            self.visit_candidate_inner(candidate);
        }
        self.stack.pop();

        let exited = self
            .ctx
            .emitter
            .emit(TraceEventKind::GoalExited, format!("{:?}", goal.goal().predicate))
            .with_hook_id(HookId::TraitSolve)
            .with_item_id(self.item_ctx.item_id)
            .with_goal_id(goal_id)
            .with_detail(format!("{:?}", goal.result()));
        self.ctx.emitter.write(&exited);
    }
}
