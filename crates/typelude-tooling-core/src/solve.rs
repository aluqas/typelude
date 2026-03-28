use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    CandidateId, GoalId, SubjectId, SubjectKind, ToolingError, ToolingResult, Trace,
    TraceEventKind, TraceId,
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
    pub predicate: String,
    pub result: String,
    pub depth: usize,
    pub candidates: Vec<GoalTreeCandidate>,
    pub children: Vec<GoalTreeGoal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalTreeCandidate {
    pub id: CandidateId,
    pub kind: String,
    pub result: String,
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
    predicate: String,
    result: String,
    candidates: BTreeMap<CandidateId, CandidateState>,
}

#[derive(Debug, Clone)]
struct CandidateState {
    kind: String,
    result: String,
    metadata: BTreeMap<String, String>,
}

impl GoalTree {
    pub fn from_trace(trace: &Trace) -> ToolingResult<Self> {
        let mut subjects = BTreeMap::<SubjectId, SubjectState>::new();
        let mut goals = BTreeMap::<GoalId, GoalState>::new();
        let mut unsupported_subjects = BTreeSet::<SubjectId>::new();

        for event in &trace.events {
            if event.kind == TraceEventKind::SubjectDiscovered {
                if let (Some(subject_id), Some(subject_kind)) =
                    (event.subject_id, event.subject_kind)
                {
                    subjects.entry(subject_id).or_insert_with(|| SubjectState {
                        kind: subject_kind,
                        label: event.title.clone(),
                        metadata: event.metadata.clone(),
                    });
                }
            }

            if let Some(goal_id) = event.goal_id {
                let subject_id = event.subject_id.ok_or_else(|| {
                    ToolingError::Parse(format!(
                        "goal event {} is missing subject_id",
                        event.id.value()
                    ))
                })?;
                let state = goals.entry(goal_id).or_insert_with(|| GoalState {
                    subject_id,
                    parent_goal_id: event.parent_goal_id,
                    predicate: event.title.clone(),
                    result: String::from("unknown"),
                    candidates: BTreeMap::new(),
                });
                state.subject_id = subject_id;
                if state.parent_goal_id.is_none() {
                    state.parent_goal_id = event.parent_goal_id;
                }
                if !event.title.is_empty() {
                    state.predicate = event.title.clone();
                }
                if event.kind == TraceEventKind::GoalExited {
                    if let Some(detail) = &event.detail {
                        state.result = detail.clone();
                    }
                }
                if let Some(candidate_id) = event.candidate_id {
                    let candidate =
                        state.candidates.entry(candidate_id).or_insert_with(|| CandidateState {
                            kind: event.title.clone(),
                            result: String::from("unknown"),
                            metadata: BTreeMap::new(),
                        });
                    if !event.title.is_empty() {
                        candidate.kind = event.title.clone();
                    }
                    if let Some(detail) = &event.detail {
                        candidate.result = detail.clone();
                    }
                    for (key, value) in &event.metadata {
                        candidate.metadata.insert(key.clone(), value.clone());
                    }
                }
            }

            if event.kind == TraceEventKind::ErrorRaised {
                if let Some(subject_id) = event.subject_id {
                    unsupported_subjects.insert(subject_id);
                }
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

        let unsupported =
            trace.events.iter().filter(|event| event.kind == TraceEventKind::ErrorRaised).count();

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

    #[must_use]
    pub fn render_summary_text(summary: &SolveSummary) -> String {
        let mut lines = vec![
            format!("subjects={}", summary.subjects),
            format!("root_goals={}", summary.root_goals),
            format!("goals={}", summary.goals),
            format!("candidates={}", summary.candidates),
            format!("max_goal_depth={}", summary.max_goal_depth),
            format!("avg_candidates_per_goal={:.2}", summary.avg_candidates_per_goal),
            format!("result.ok={}", summary.result_ok),
            format!("result.no_solution={}", summary.result_no_solution),
            format!("result.ambiguous={}", summary.result_ambiguous),
            format!("result.unsupported={}", summary.result_unsupported),
        ];

        if !summary.top_predicates.is_empty() {
            lines.push(String::from("top_predicates:"));
            for entry in &summary.top_predicates {
                lines.push(format!("{} :: {}", entry.count, entry.label));
            }
        }

        if !summary.top_candidate_kinds.is_empty() {
            lines.push(String::from("top_candidate_kinds:"));
            for entry in &summary.top_candidate_kinds {
                lines.push(format!("{} :: {}", entry.count, entry.label));
            }
        }

        lines.join("\n")
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
            result: candidate.result.clone(),
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
        result: state.result.clone(),
        depth,
        candidates,
        children,
    })
}

#[allow(clippy::too_many_arguments)]
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
    *predicate_counts.entry(goal.predicate.clone()).or_default() += 1;
    let lower = goal.result.to_ascii_lowercase();
    if lower.contains("no_solution")
        || lower.contains("no solution")
        || lower.contains("nosolution")
    {
        *result_no_solution += 1;
    } else if lower.contains("ambiguous") {
        *result_ambiguous += 1;
    } else if lower != "unknown" {
        *result_ok += 1;
    }

    for candidate in &goal.candidates {
        *candidate_count += 1;
        *candidate_counts.entry(candidate.kind.clone()).or_default() += 1;
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

#[cfg(test)]
mod tests {
    use crate::{CandidateId, EventId, GoalId, SubjectId, SubjectKind, TraceEvent};

    use super::*;

    fn push_subject(trace: &mut Trace, id: u64, label: &str) {
        let mut subject =
            TraceEvent::new(EventId::new(id), TraceEventKind::SubjectDiscovered, label);
        subject.subject_id = Some(SubjectId::new(10));
        subject.subject_kind = Some(SubjectKind::Predicate);
        trace.push(subject);
    }

    #[test]
    fn goal_tree_and_summary_are_stable() {
        let mut trace = Trace::new(TraceId::new(1));
        push_subject(&mut trace, 1, "RunWriter");

        let mut goal =
            TraceEvent::new(EventId::new(2), TraceEventKind::GoalEntered, "<T as Eval>");
        goal.subject_id = Some(SubjectId::new(10));
        goal.subject_kind = Some(SubjectKind::Predicate);
        goal.goal_id = Some(GoalId::new(1));
        trace.push(goal);

        let mut candidate =
            TraceEvent::new(EventId::new(3), TraceEventKind::CandidateResult, "ImplCandidate");
        candidate.subject_id = Some(SubjectId::new(10));
        candidate.subject_kind = Some(SubjectKind::Predicate);
        candidate.goal_id = Some(GoalId::new(1));
        candidate.candidate_id = Some(CandidateId::new(2));
        candidate.detail = Some(String::from("Ok(())"));
        trace.push(candidate);

        let mut nested =
            TraceEvent::new(EventId::new(4), TraceEventKind::GoalEntered, "<U as Eval>");
        nested.subject_id = Some(SubjectId::new(10));
        nested.subject_kind = Some(SubjectKind::Predicate);
        nested.goal_id = Some(GoalId::new(3));
        nested.parent_goal_id = Some(GoalId::new(1));
        trace.push(nested);

        let mut nested_exit =
            TraceEvent::new(EventId::new(5), TraceEventKind::GoalExited, "<U as Eval>");
        nested_exit.subject_id = Some(SubjectId::new(10));
        nested_exit.subject_kind = Some(SubjectKind::Predicate);
        nested_exit.goal_id = Some(GoalId::new(3));
        nested_exit.parent_goal_id = Some(GoalId::new(1));
        nested_exit.detail = Some(String::from("NoSolution"));
        trace.push(nested_exit);

        let mut goal_exit =
            TraceEvent::new(EventId::new(6), TraceEventKind::GoalExited, "<T as Eval>");
        goal_exit.subject_id = Some(SubjectId::new(10));
        goal_exit.subject_kind = Some(SubjectKind::Predicate);
        goal_exit.goal_id = Some(GoalId::new(1));
        goal_exit.detail = Some(String::from("Ok(())"));
        trace.push(goal_exit);

        let tree = GoalTree::from_trace(&trace).expect("goal tree should build");
        assert_eq!(tree.subjects.len(), 1);
        assert_eq!(tree.subjects[0].roots.len(), 1);
        assert_eq!(tree.subjects[0].roots[0].children.len(), 1);

        let summary = tree.summarize(&trace);
        assert_eq!(summary.subjects, 1);
        assert_eq!(summary.root_goals, 1);
        assert_eq!(summary.goals, 2);
        assert_eq!(summary.candidates, 1);
        assert_eq!(summary.result_ok, 1);
        assert_eq!(summary.result_no_solution, 1);
    }
}
