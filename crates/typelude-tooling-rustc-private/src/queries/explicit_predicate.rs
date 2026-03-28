use std::panic::{AssertUnwindSafe, catch_unwind};

use rustc_infer::infer::TyCtxtInferExt;
use rustc_middle::{
    traits::solve::Goal,
    ty::{TypingMode, Upcast},
};
use rustc_trait_selection::solve::inspect::{
    InferCtxtProofTreeExt, InspectCandidate, InspectConfig, InspectGoal, ProofTreeVisitor,
};
use typelude_tooling_core::{
    CandidateDiscovered, CandidateKind, CandidateResult, CandidateTried, ErrorRaised,
    GoalDiscovered, GoalEntered, GoalExited, GoalResult, HookId, PredicateRepr, SemanticTag,
    SubjectId, TracePayload,
};

use crate::{
    error::{AnalysisError, AnalysisResult},
    queries::{Query, QueryContext},
    session::AnalysisSession,
    subjects::ResolvedSubject,
};

#[derive(Debug, Clone, Copy)]
pub struct SolveExplicitPredicateQuery<'tcx> {
    pub subject: ResolvedSubject<'tcx>,
    pub subject_id: SubjectId,
}

impl<'tcx> Query<'tcx> for SolveExplicitPredicateQuery<'tcx> {
    fn run(&self, context: &mut QueryContext<'_, '_, 'tcx>) -> AnalysisResult<()> {
        solve_explicit_predicate(context.session_mut(), self.subject, self.subject_id)
    }
}

pub(crate) fn run_owner_predicates<'tcx>(
    session: &mut AnalysisSession<'_, 'tcx>,
    owner_subject: ResolvedSubject<'tcx>,
    owner_matches: bool,
    owner_def_id: rustc_span::def_id::LocalDefId,
) -> AnalysisResult<()> {
    let mut owner_subject_id = None;
    let predicates =
        session.tcx.explicit_predicates_of(owner_def_id).instantiate_identity(session.tcx);

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
        };
        let mut context = QueryContext::new(session);
        query.run(&mut context)?;
    }

    Ok(())
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
            "SolveExplicitPredicateQuery requires an explicit predicate subject",
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
    session.emitter.write_payload(TracePayload::ErrorRaised(ErrorRaised {
        hook_id: Some(HookId::TraitSolve),
        subject_id: Some(subject_id),
        goal_id: None,
        result: GoalResult::Unsupported,
        message: detail,
    }));
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
        let candidate_kind = lower_candidate_kind(&format!("{:?}", candidate.kind()));
        let semantic_tags = semantic_tags_for_candidate(&candidate_kind);
        let metadata = BTreeMap::from([(
            String::from("raw_candidate_kind"),
            format!("{:?}", candidate.kind()),
        )]);

        self.session.emitter.write_payload(TracePayload::CandidateDiscovered(
            CandidateDiscovered {
                hook_id: HookId::TraitSolve,
                subject_id: self.subject_id,
                goal_id,
                candidate_id,
                candidate_kind: candidate_kind.clone(),
                semantic_tags: semantic_tags.clone(),
                metadata: metadata.clone(),
            },
        ));

        self.session.emitter.write_payload(TracePayload::CandidateTried(CandidateTried {
            hook_id: HookId::TraitSolve,
            subject_id: self.subject_id,
            goal_id,
            candidate_id,
            candidate_kind: candidate_kind.clone(),
            semantic_tags: semantic_tags.clone(),
            metadata: metadata.clone(),
        }));

        self.session.emitter.write_payload(TracePayload::CandidateResult(CandidateResult {
            hook_id: HookId::TraitSolve,
            subject_id: self.subject_id,
            goal_id,
            candidate_id,
            candidate_kind,
            result: lower_goal_result(&format!("{:?}", candidate.result())),
            semantic_tags,
            metadata,
        }));

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
        let predicate_debug = format!("{:?}", goal.goal().predicate);
        if !self.session.focus_matches(&predicate_debug) {
            return;
        }
        if !self.session.can_emit() {
            self.session.record_drop();
            return;
        }

        let goal_id = self.session.alloc_goal_id();
        let parent_goal_id = self.stack.last().copied();
        let candidates = goal.candidates();
        let predicate = lower_predicate_repr(&predicate_debug);
        let semantic_tags = semantic_tags_for_predicate(&predicate);
        self.session.stats.goal_count += 1;

        self.session.emitter.write_payload(TracePayload::GoalDiscovered(GoalDiscovered {
            hook_id: HookId::TraitSolve,
            subject_id: self.subject_id,
            goal_id,
            parent_goal_id,
            predicate: predicate.clone(),
            candidate_count: candidates.len(),
            semantic_tags: semantic_tags.clone(),
        }));

        self.session.emitter.write_payload(TracePayload::GoalEntered(GoalEntered {
            hook_id: HookId::TraitSolve,
            subject_id: self.subject_id,
            goal_id,
            parent_goal_id,
            predicate: predicate.clone(),
            semantic_tags: semantic_tags.clone(),
        }));

        self.stack.push(goal_id);
        for candidate in &candidates {
            self.visit_candidate_inner(candidate);
        }
        self.stack.pop();

        self.session.emitter.write_payload(TracePayload::GoalExited(GoalExited {
            hook_id: HookId::TraitSolve,
            subject_id: self.subject_id,
            goal_id,
            parent_goal_id,
            predicate,
            result: lower_goal_result(&format!("{:?}", goal.result())),
            semantic_tags,
        }));
    }
}

use std::collections::BTreeMap;

fn lower_predicate_repr(raw: &str) -> PredicateRepr {
    let text = raw.trim();
    if text.contains("AliasRelate(") {
        let inner = text
            .split_once("AliasRelate(")
            .and_then(|(_, rest)| rest.rsplit_once(')'))
            .map(|(body, _)| body)
            .unwrap_or_default();
        if let Some((lhs, rhs)) = split_top_level_once(inner) {
            return PredicateRepr::AliasRelate {
                lhs: lhs.to_string(),
                rhs: rhs.to_string(),
            };
        }
    }
    if text.contains("NormalizesTo(") {
        let inner = text
            .split_once("NormalizesTo(")
            .and_then(|(_, rest)| rest.rsplit_once(')'))
            .map(|(body, _)| body)
            .unwrap_or_default();
        if let Some((alias, target)) = split_top_level_once(inner) {
            return PredicateRepr::NormalizesTo {
                alias: alias.to_string(),
                target: target.to_string(),
            };
        }
    }
    if text.contains("TraitPredicate(") {
        let trait_path = if let Some(start) = text.find("TraitPredicate(") {
            let inner = &text[start + "TraitPredicate(".len()..];
            inner.split_once(')').map_or(inner, |(body, _)| body).to_string()
        } else {
            text.to_string()
        };
        return PredicateRepr::TraitPredicate {
            trait_path,
            self_ty: String::from("<unknown>"),
            args: Vec::new(),
        };
    }
    if text.is_empty() {
        PredicateRepr::Unknown
    } else {
        PredicateRepr::DebugText(text.to_string())
    }
}

fn lower_candidate_kind(raw: &str) -> CandidateKind {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("paramenv") {
        CandidateKind::ParamEnv
    } else if lower.contains("impl") {
        CandidateKind::Impl
    } else if lower.contains("builtin") {
        CandidateKind::Builtin
    } else if lower.contains("aliasrelate") {
        CandidateKind::AliasRelate
    } else if lower.contains("normalize") {
        CandidateKind::Normalize
    } else {
        CandidateKind::Unknown(raw.to_string())
    }
}

fn lower_goal_result(raw: &str) -> GoalResult {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("no_solution") || lower.contains("nosolution") {
        GoalResult::NoSolution
    } else if lower.contains("ambiguous") {
        GoalResult::Ambiguous
    } else if lower.contains("unsupported") {
        GoalResult::Unsupported
    } else if lower.contains("ok") || lower.contains("yes") {
        GoalResult::Success
    } else {
        GoalResult::Error
    }
}

fn semantic_tags_for_predicate(predicate: &PredicateRepr) -> Vec<SemanticTag> {
    let text = predicate.debug_text();
    let mut tags = Vec::new();
    if text.contains("EIf") {
        tags.push(SemanticTag::BranchLike);
    }
    if text.contains("EWhile") {
        tags.push(SemanticTag::LoopLike);
    }
    if text.contains("EGet") {
        tags.push(SemanticTag::LookupLike);
    }
    if text.contains("EMap") {
        tags.push(SemanticTag::MapLike);
    }
    if text.contains("EApp") {
        tags.push(SemanticTag::ApplyLike);
    }
    if text.contains("Helper") {
        tags.push(SemanticTag::HelperDispatchLike);
    }
    if text.contains("Op") {
        tags.push(SemanticTag::VmOpLike);
    }
    if tags.is_empty() {
        tags.push(SemanticTag::EvalLike);
    }
    tags
}

fn semantic_tags_for_candidate(candidate_kind: &CandidateKind) -> Vec<SemanticTag> {
    match candidate_kind {
        CandidateKind::AliasRelate => vec![SemanticTag::HelperDispatchLike],
        CandidateKind::Normalize => vec![SemanticTag::EvalLike],
        CandidateKind::ParamEnv | CandidateKind::Impl | CandidateKind::Builtin => {
            vec![SemanticTag::EvalLike]
        },
        CandidateKind::Unknown(_) => vec![SemanticTag::Unknown],
    }
}

fn split_top_level_once(input: &str) -> Option<(&str, &str)> {
    let mut depth = 0_i32;
    for (index, ch) in input.char_indices() {
        match ch {
            '<' | '(' | '[' | '{' => depth += 1,
            '>' | ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                let lhs = input[..index].trim();
                let rhs = input[index + 1..].trim();
                return Some((lhs, rhs));
            },
            _ => {},
        }
    }
    None
}
