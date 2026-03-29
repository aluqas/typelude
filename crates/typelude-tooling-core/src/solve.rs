use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    CandidateId, CandidateKind, GoalId, GoalResult, PredicateRepr, SubjectId, SubjectKind,
    ToolingError, ToolingResult, Trace, TraceId, TracePayload, semantic::SemanticTag,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalTree {
    pub trace_id: TraceId,
    pub subjects: Vec<GoalTreeSubject>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalTreeSubject {
    pub id: SubjectId,
    pub kind: SubjectKind,
    pub label: String,
    pub metadata: BTreeMap<String, String>,
    pub roots: Vec<GoalTreeGoal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalTreeGoal {
    pub id: GoalId,
    pub parent_goal_id: Option<GoalId>,
    pub predicate: PredicateRepr,
    pub result: GoalResult,
    pub depth: usize,
    pub semantic_tags: Vec<SemanticTag>,
    pub candidates: Vec<GoalTreeCandidate>,
    pub children: Vec<GoalTreeGoal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalTreeCandidate {
    pub id: CandidateId,
    pub kind: CandidateKind,
    pub result: GoalResult,
    pub semantic_tags: Vec<SemanticTag>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolveSummary {
    pub subjects: usize,
    pub root_goals: usize,
    pub goals: usize,
    pub candidates: usize,
    pub max_goal_depth: usize,
    pub avg_candidates_per_goal: f64,
    pub result_ok: usize,
    pub result_no_solution: usize,
    pub result_ambiguous: usize,
    pub result_unsupported: usize,
    pub top_predicates: Vec<SolveSummaryEntry>,
    pub top_candidate_kinds: Vec<SolveSummaryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolveSummaryEntry {
    pub label: String,
    pub count: usize,
}

#[derive(Debug, Clone)]
struct SubjectState {
    kind: SubjectKind,
    label: String,
    metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
struct GoalState {
    subject_id: SubjectId,
    parent_goal_id: Option<GoalId>,
    predicate: PredicateRepr,
    result: GoalResult,
    semantic_tags: Vec<SemanticTag>,
    candidates: BTreeMap<CandidateId, CandidateState>,
}

#[derive(Debug, Clone)]
struct CandidateState {
    kind: CandidateKind,
    result: GoalResult,
    semantic_tags: Vec<SemanticTag>,
    metadata: BTreeMap<String, String>,
}

impl GoalTree {
    pub fn from_trace(trace: &Trace) -> ToolingResult<Self> {
        let mut subjects = BTreeMap::<SubjectId, SubjectState>::new();
        let mut goals = BTreeMap::<GoalId, GoalState>::new();
        let mut unsupported_subjects = BTreeSet::<SubjectId>::new();

        for event in &trace.events {
            match &event.payload {
                TracePayload::SubjectDiscovered(data) => {
                    subjects.entry(data.subject_id).or_insert_with(|| SubjectState {
                        kind: data.subject_kind,
                        label: data.label.clone(),
                        metadata: data.metadata.clone(),
                    });
                },
                TracePayload::GoalDiscovered(data) => {
                    goals.entry(data.goal_id).or_insert_with(|| GoalState {
                        subject_id: data.subject_id,
                        parent_goal_id: data.parent_goal_id,
                        predicate: data.predicate.clone(),
                        result: GoalResult::Unsupported,
                        semantic_tags: data.semantic_tags.clone(),
                        candidates: BTreeMap::new(),
                    });
                },
                TracePayload::GoalEntered(data) => {
                    let state = goals.entry(data.goal_id).or_insert_with(|| GoalState {
                        subject_id: data.subject_id,
                        parent_goal_id: data.parent_goal_id,
                        predicate: data.predicate.clone(),
                        result: GoalResult::Unsupported,
                        semantic_tags: data.semantic_tags.clone(),
                        candidates: BTreeMap::new(),
                    });
                    state.subject_id = data.subject_id;
                    state.parent_goal_id = data.parent_goal_id;
                    state.predicate = data.predicate.clone();
                    state.semantic_tags = data.semantic_tags.clone();
                },
                TracePayload::GoalExited(data) => {
                    let state = goals.entry(data.goal_id).or_insert_with(|| GoalState {
                        subject_id: data.subject_id,
                        parent_goal_id: data.parent_goal_id,
                        predicate: data.predicate.clone(),
                        result: data.result,
                        semantic_tags: data.semantic_tags.clone(),
                        candidates: BTreeMap::new(),
                    });
                    state.subject_id = data.subject_id;
                    state.parent_goal_id = data.parent_goal_id;
                    state.predicate = data.predicate.clone();
                    state.result = data.result;
                    state.semantic_tags = data.semantic_tags.clone();
                },
                TracePayload::CandidateDiscovered(data) => {
                    let goal = goals.entry(data.goal_id).or_insert_with(|| GoalState {
                        subject_id: data.subject_id,
                        parent_goal_id: None,
                        predicate: PredicateRepr::Unknown,
                        result: GoalResult::Unsupported,
                        semantic_tags: Vec::new(),
                        candidates: BTreeMap::new(),
                    });
                    goal.candidates.entry(data.candidate_id).or_insert_with(|| CandidateState {
                        kind: data.candidate_kind.clone(),
                        result: GoalResult::Unsupported,
                        semantic_tags: data.semantic_tags.clone(),
                        metadata: data.metadata.clone(),
                    });
                },
                TracePayload::CandidateTried(data) => {
                    let goal = goals.entry(data.goal_id).or_insert_with(|| GoalState {
                        subject_id: data.subject_id,
                        parent_goal_id: None,
                        predicate: PredicateRepr::Unknown,
                        result: GoalResult::Unsupported,
                        semantic_tags: Vec::new(),
                        candidates: BTreeMap::new(),
                    });
                    let candidate =
                        goal.candidates.entry(data.candidate_id).or_insert_with(|| {
                            CandidateState {
                                kind: data.candidate_kind.clone(),
                                result: GoalResult::Unsupported,
                                semantic_tags: data.semantic_tags.clone(),
                                metadata: data.metadata.clone(),
                            }
                        });
                    candidate.kind = data.candidate_kind.clone();
                    candidate.semantic_tags = data.semantic_tags.clone();
                    candidate.metadata = data.metadata.clone();
                },
                TracePayload::CandidateResult(data) => {
                    let goal = goals.entry(data.goal_id).or_insert_with(|| GoalState {
                        subject_id: data.subject_id,
                        parent_goal_id: None,
                        predicate: PredicateRepr::Unknown,
                        result: GoalResult::Unsupported,
                        semantic_tags: Vec::new(),
                        candidates: BTreeMap::new(),
                    });
                    let candidate =
                        goal.candidates.entry(data.candidate_id).or_insert_with(|| {
                            CandidateState {
                                kind: data.candidate_kind.clone(),
                                result: data.result,
                                semantic_tags: data.semantic_tags.clone(),
                                metadata: data.metadata.clone(),
                            }
                        });
                    candidate.kind = data.candidate_kind.clone();
                    candidate.result = data.result;
                    candidate.semantic_tags = data.semantic_tags.clone();
                    candidate.metadata = data.metadata.clone();
                },
                TracePayload::ErrorRaised(data) => {
                    if let Some(subject_id) = data.subject_id {
                        unsupported_subjects.insert(subject_id);
                    }
                },
                TracePayload::RunStarted(..)
                | TracePayload::RunFinished(..)
                | TracePayload::DiagnosticEmitted(..)
                | TracePayload::RelationDeclared(..)
                | TracePayload::Info(..) => {},
            }
        }

        let mut children_by_goal = BTreeMap::<GoalId, Vec<GoalId>>::new();
        let mut roots_by_subject = BTreeMap::<SubjectId, Vec<GoalId>>::new();
        for (goal_id, state) in &goals {
            if let Some(parent_goal_id) = state.parent_goal_id {
                children_by_goal.entry(parent_goal_id).or_default().push(*goal_id);
            } else {
                roots_by_subject.entry(state.subject_id).or_default().push(*goal_id);
            }
        }

        for subject_id in unsupported_subjects {
            roots_by_subject.entry(subject_id).or_default();
        }

        let mut rendered_subjects = Vec::new();
        for (subject_id, root_ids) in roots_by_subject {
            let subject = subjects.remove(&subject_id).unwrap_or(SubjectState {
                kind: SubjectKind::Predicate,
                label: format!("subject#{}", subject_id.value()),
                metadata: BTreeMap::new(),
            });
            let roots = root_ids
                .into_iter()
                .map(|goal_id| build_goal(goal_id, 0, &goals, &children_by_goal))
                .collect::<ToolingResult<Vec<_>>>()?;
            rendered_subjects.push(GoalTreeSubject {
                id: subject_id,
                kind: subject.kind,
                label: subject.label,
                metadata: subject.metadata,
                roots,
            });
        }

        Ok(Self {
            trace_id: trace.id,
            subjects: rendered_subjects,
        })
    }

    #[must_use]
    pub fn summarize(&self, trace: &Trace) -> SolveSummary {
        let mut predicate_counts = BTreeMap::<String, usize>::new();
        let mut candidate_counts = BTreeMap::<String, usize>::new();
        let mut result_ok = 0_usize;
        let mut result_no_solution = 0_usize;
        let mut result_ambiguous = 0_usize;
        let mut max_goal_depth = 0_usize;
        let mut goal_count = 0_usize;
        let mut candidate_count = 0_usize;
        let mut root_goal_count = 0_usize;

        for subject in &self.subjects {
            root_goal_count += subject.roots.len();
            for root in &subject.roots {
                walk_goal(
                    root,
                    &mut predicate_counts,
                    &mut candidate_counts,
                    &mut result_ok,
                    &mut result_no_solution,
                    &mut result_ambiguous,
                    &mut max_goal_depth,
                    &mut goal_count,
                    &mut candidate_count,
                );
            }
        }

        let unsupported = trace
            .events
            .iter()
            .filter(|event| matches!(event.payload, TracePayload::ErrorRaised(..)))
            .count();

        SolveSummary {
            subjects: self.subjects.len(),
            root_goals: root_goal_count,
            goals: goal_count,
            candidates: candidate_count,
            max_goal_depth,
            avg_candidates_per_goal: if goal_count == 0 {
                0.0
            } else {
                candidate_count as f64 / goal_count as f64
            },
            result_ok,
            result_no_solution,
            result_ambiguous,
            result_unsupported: unsupported,
            top_predicates: top_entries(predicate_counts, 5),
            top_candidate_kinds: top_entries(candidate_counts, 5),
        }
    }
}

fn build_goal(
    goal_id: GoalId,
    depth: usize,
    goals: &BTreeMap<GoalId, GoalState>,
    children_by_goal: &BTreeMap<GoalId, Vec<GoalId>>,
) -> ToolingResult<GoalTreeGoal> {
    let state = goals.get(&goal_id).ok_or_else(|| {
        ToolingError::Parse(format!("goal {} was referenced but not discovered", goal_id.value()))
    })?;
    let candidates = state
        .candidates
        .iter()
        .map(|(candidate_id, candidate)| GoalTreeCandidate {
            id: *candidate_id,
            kind: candidate.kind.clone(),
            result: candidate.result,
            semantic_tags: candidate.semantic_tags.clone(),
            metadata: candidate.metadata.clone(),
        })
        .collect();
    let children = children_by_goal
        .get(&goal_id)
        .into_iter()
        .flat_map(|children| children.iter().copied())
        .map(|child_id| build_goal(child_id, depth + 1, goals, children_by_goal))
        .collect::<ToolingResult<Vec<_>>>()?;

    Ok(GoalTreeGoal {
        id: goal_id,
        parent_goal_id: state.parent_goal_id,
        predicate: state.predicate.clone(),
        result: state.result,
        depth,
        semantic_tags: state.semantic_tags.clone(),
        candidates,
        children,
    })
}

fn walk_goal(
    goal: &GoalTreeGoal,
    predicate_counts: &mut BTreeMap<String, usize>,
    candidate_counts: &mut BTreeMap<String, usize>,
    result_ok: &mut usize,
    result_no_solution: &mut usize,
    result_ambiguous: &mut usize,
    max_goal_depth: &mut usize,
    goal_count: &mut usize,
    candidate_count: &mut usize,
) {
    *goal_count += 1;
    *max_goal_depth = (*max_goal_depth).max(goal.depth);
    *predicate_counts.entry(goal.predicate.debug_text()).or_default() += 1;

    match goal.result {
        GoalResult::Success => *result_ok += 1,
        GoalResult::NoSolution => *result_no_solution += 1,
        GoalResult::Ambiguous => *result_ambiguous += 1,
        GoalResult::Unsupported | GoalResult::Error => {},
    }

    for candidate in &goal.candidates {
        *candidate_count += 1;
        *candidate_counts.entry(candidate.kind.label()).or_default() += 1;
    }

    for child in &goal.children {
        walk_goal(
            child,
            predicate_counts,
            candidate_counts,
            result_ok,
            result_no_solution,
            result_ambiguous,
            max_goal_depth,
            goal_count,
            candidate_count,
        );
    }
}

fn top_entries(counts: BTreeMap<String, usize>, limit: usize) -> Vec<SolveSummaryEntry> {
    let mut entries = counts
        .into_iter()
        .map(|(label, count)| SolveSummaryEntry {
            label,
            count,
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right.count.cmp(&left.count).then_with(|| left.label.cmp(&right.label))
    });
    entries.truncate(limit);
    entries
}
