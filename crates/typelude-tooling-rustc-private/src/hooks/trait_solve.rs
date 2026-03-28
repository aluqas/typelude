use std::panic::{AssertUnwindSafe, catch_unwind};

use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::{
    traits::solve::Goal,
    ty::{Clause, TypingMode, Upcast},
};
use rustc_span::def_id::LocalDefId;
use rustc_trait_selection::solve::inspect::{
    InferCtxtProofTreeExt, InspectCandidate, InspectConfig, InspectGoal, ProofTreeVisitor,
};
use typelude_tooling_core::{HookId, SubjectId, SubjectKind, TraceEventKind};

use crate::{
    emit::TraceEventExt,
    error::{AnalysisError, AnalysisResult},
    hooks::Hook,
    queries::{Query, QueryContext, SolveExplicitPredicateQuery},
    session::AnalysisSession,
    subjects::ResolvedSubject,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct TraitSolveHook;

pub type SweepExplicitPredicatesHook = TraitSolveHook;

impl<'tcx> Hook<'tcx> for TraitSolveHook {
    fn run(&self, session: &mut AnalysisSession<'_, 'tcx>) -> AnalysisResult<()> {
        let crate_items = session.tcx.hir_crate_items(());
        for item_id in crate_items.free_items() {
            if !session.can_emit() {
                session.record_drop();
                break;
            }

            let item = session.tcx.hir_item(item_id);
            let def_id = item_id.owner_id.def_id;
            let owner_subject = match item.kind {
                rustc_hir::ItemKind::Impl(..) => ResolvedSubject::Impl(def_id),
                _ => ResolvedSubject::Item(def_id),
            };
            solve_owner_predicates(session, owner_subject, def_id)?;

            if let rustc_hir::ItemKind::Impl(impl_data) = item.kind {
                for impl_item_id in impl_data.items {
                    if !session.can_emit() {
                        session.record_drop();
                        break;
                    }
                    let assoc_def_id = impl_item_id.owner_id.def_id;
                    solve_owner_predicates(
                        session,
                        ResolvedSubject::AssocItem(assoc_def_id),
                        assoc_def_id,
                    )?;
                }
            }
        }
        Ok(())
    }
}

pub fn solve_explicit_predicate<'tcx>(
    session: &mut AnalysisSession<'_, 'tcx>,
    subject: ResolvedSubject<'tcx>,
    subject_id: SubjectId,
) -> AnalysisResult<()> {
    let ResolvedSubject::ExplicitPredicate {
        clause,
        owner,
        ..
    } = subject
    else {
        return Err(AnalysisError::new(
            "SolveExplicitPredicateHook requires an explicit predicate subject",
        ));
    };

    let infcx = session.tcx.infer_ctxt().build(TypingMode::non_body_analysis());
    let goal = Goal {
        param_env: session.tcx.param_env(owner),
        predicate: clause.upcast(session.tcx),
    };
    let mut visitor = ProofTreeCollector::new(session, subject_id);
    let result = catch_unwind(AssertUnwindSafe(|| {
        infcx.probe(|_| {
            infcx.visit_proof_tree(goal, &mut visitor);
        });
    }));
    if let Err(payload) = result {
        emit_unsupported(visitor.session, subject_id, panic_message(payload));
    }
    Ok(())
}

fn solve_owner_predicates<'tcx>(
    session: &mut AnalysisSession<'_, 'tcx>,
    owner_subject: ResolvedSubject<'tcx>,
    owner_def_id: LocalDefId,
) -> AnalysisResult<()> {
    let owner_matches = subject_matches_filter(session, owner_subject);
    let mut owner_subject_id = None;
    let predicates: Vec<(Clause<'tcx>, rustc_span::Span)> = session
        .tcx
        .explicit_predicates_of(owner_def_id)
        .instantiate_identity(session.tcx)
        .into_iter()
        .collect();

    for (index, (clause, _span)) in predicates.into_iter().enumerate() {
        if !session.can_emit() {
            session.record_drop();
            break;
        }

        let predicate_subject = ResolvedSubject::ExplicitPredicate {
            owner: owner_def_id,
            index,
            clause,
        };
        let predicate_matches = subject_matches_filter(session, predicate_subject);
        if !predicate_matches && !owner_matches {
            continue;
        }

        let owner_subject_id = *owner_subject_id
            .get_or_insert_with(|| session.ensure_subject(HookId::TraitSolve, owner_subject));
        let predicate_subject_id = session.ensure_subject(HookId::TraitSolve, predicate_subject);
        if predicate_subject_id == owner_subject_id {
            continue;
        }

        let query = SolveExplicitPredicateQuery {
            subject: predicate_subject,
            subject_id: predicate_subject_id,
            runner: solve_explicit_predicate,
        };
        let mut context = QueryContext::new(session);
        query.run(&mut context)?;
    }

    Ok(())
}

fn subject_matches_filter<'tcx>(
    session: &AnalysisSession<'_, 'tcx>,
    subject: ResolvedSubject<'tcx>,
) -> bool {
    let label = subject.label(session.tcx);
    let metadata = subject.metadata(session.tcx);
    session.subject_matches(&label, &metadata)
}

fn emit_unsupported(session: &mut AnalysisSession<'_, '_>, subject_id: SubjectId, detail: String) {
    if !session.can_emit() {
        session.record_drop();
        return;
    }
    let event = session
        .emitter
        .emit(TraceEventKind::ErrorRaised, "unsupported_goal")
        .with_hook_id(HookId::TraitSolve)
        .with_subject(subject_id, SubjectKind::Predicate)
        .with_detail(detail)
        .with_metadata("result", "unsupported");
    session.emitter.write(&event);
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        String::from("trait solving panicked")
    }
}

struct ProofTreeCollector<'v, 'c, 'tcx> {
    session: &'v mut AnalysisSession<'c, 'tcx>,
    subject_id: SubjectId,
    stack: Vec<typelude_tooling_core::GoalId>,
}

impl<'v, 'c, 'tcx> ProofTreeCollector<'v, 'c, 'tcx> {
    fn new(session: &'v mut AnalysisSession<'c, 'tcx>, subject_id: SubjectId) -> Self {
        Self {
            session,
            subject_id,
            stack: Vec::new(),
        }
    }

    fn visit_candidate_inner(&mut self, candidate: &InspectCandidate<'_, 'tcx>) {
        if !self.session.can_emit() {
            self.session.record_drop();
            return;
        }
        let Some(goal_id) = self.stack.last().copied() else {
            return;
        };

        let candidate_id = self.session.alloc_candidate_id();
        self.session.stats.candidate_count += 1;

        let discovered = self
            .session
            .emitter
            .emit(TraceEventKind::CandidateDiscovered, format!("{:?}", candidate.kind()))
            .with_hook_id(HookId::TraitSolve)
            .with_subject(self.subject_id, SubjectKind::Predicate)
            .with_goal_id(goal_id)
            .with_candidate_id(candidate_id);
        self.session.emitter.write(&discovered);

        let tried = self
            .session
            .emitter
            .emit(TraceEventKind::CandidateTried, format!("{:?}", candidate.kind()))
            .with_hook_id(HookId::TraitSolve)
            .with_subject(self.subject_id, SubjectKind::Predicate)
            .with_goal_id(goal_id)
            .with_candidate_id(candidate_id);
        self.session.emitter.write(&tried);

        let result = self
            .session
            .emitter
            .emit(TraceEventKind::CandidateResult, format!("{:?}", candidate.kind()))
            .with_hook_id(HookId::TraitSolve)
            .with_subject(self.subject_id, SubjectKind::Predicate)
            .with_goal_id(goal_id)
            .with_candidate_id(candidate_id)
            .with_detail(format!("{:?}", candidate.result()));
        self.session.emitter.write(&result);

        candidate.visit_nested_no_probe(self);
    }
}

impl<'tcx> ProofTreeVisitor<'tcx> for ProofTreeCollector<'_, '_, 'tcx> {
    type Result = ();

    fn span(&self) -> rustc_span::Span {
        rustc_span::DUMMY_SP
    }

    fn config(&self) -> InspectConfig {
        InspectConfig {
            max_depth: self.session.max_depth(),
        }
    }

    fn visit_goal(&mut self, goal: &InspectGoal<'_, 'tcx>) -> Self::Result {
        let predicate = format!("{:?}", goal.goal().predicate);
        if !self.session.focus_matches(&predicate) {
            return;
        }
        if !self.session.can_emit() {
            self.session.record_drop();
            return;
        }

        let goal_id = self.session.alloc_goal_id();
        let parent_goal_id = self.stack.last().copied();
        let candidates = goal.candidates();
        self.session.stats.goal_count += 1;

        let mut discovered = self
            .session
            .emitter
            .emit(TraceEventKind::GoalDiscovered, predicate.clone())
            .with_hook_id(HookId::TraitSolve)
            .with_subject(self.subject_id, SubjectKind::Predicate)
            .with_goal_id(goal_id)
            .with_metadata("candidate_count", candidates.len().to_string());
        if let Some(parent) = parent_goal_id {
            discovered = discovered.with_parent_goal_id(parent);
        }
        self.session.emitter.write(&discovered);

        let mut entered = self
            .session
            .emitter
            .emit(TraceEventKind::GoalEntered, predicate)
            .with_hook_id(HookId::TraitSolve)
            .with_subject(self.subject_id, SubjectKind::Predicate)
            .with_goal_id(goal_id);
        if let Some(parent) = parent_goal_id {
            entered = entered.with_parent_goal_id(parent);
        }
        self.session.emitter.write(&entered);

        self.stack.push(goal_id);
        for candidate in &candidates {
            self.visit_candidate_inner(candidate);
        }
        self.stack.pop();

        let exited = self
            .session
            .emitter
            .emit(TraceEventKind::GoalExited, format!("{:?}", goal.goal().predicate))
            .with_hook_id(HookId::TraitSolve)
            .with_subject(self.subject_id, SubjectKind::Predicate)
            .with_goal_id(goal_id)
            .with_detail(format!("{:?}", goal.result()));
        self.session.emitter.write(&exited);
    }
}

#[cfg(test)]
mod tests {
    use super::{SweepExplicitPredicatesHook, TraitSolveHook};

    #[test]
    fn sweep_alias_matches_trait_solve_type() {
        let _sweep: SweepExplicitPredicatesHook = TraitSolveHook;
    }
}
