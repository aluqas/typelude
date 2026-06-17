use serde::{Deserialize, Serialize};

use crate::{GoalResult, GoalTree, GoalTreeGoal, GoalTreeSubject};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolveResultFilter {
    Ok,
    NoSolution,
    Ambiguous,
    Unsupported,
}

impl SolveResultFilter {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::NoSolution => "no_solution",
            Self::Ambiguous => "ambiguous",
            Self::Unsupported => "unsupported",
        }
    }

    #[must_use]
    pub const fn matches(self, value: GoalResult) -> bool {
        match self {
            Self::Ok => matches!(value, GoalResult::Success),
            Self::NoSolution => matches!(value, GoalResult::NoSolution),
            Self::Ambiguous => matches!(value, GoalResult::Ambiguous),
            Self::Unsupported => matches!(value, GoalResult::Unsupported | GoalResult::Error),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolveFilters {
    pub result: Option<SolveResultFilter>,
    pub candidate_kind: Option<String>,
    pub max_depth: Option<usize>,
    pub subject: Option<String>,
}

#[must_use]
pub fn filter_goal_tree(tree: &GoalTree, filters: &SolveFilters) -> GoalTree {
    let mut subjects = Vec::new();
    for subject in &tree.subjects {
        if !subject_matches(subject, filters.subject.as_deref()) {
            continue;
        }
        let roots =
            subject.roots.iter().filter_map(|goal| filter_goal(goal, filters)).collect::<Vec<_>>();
        if !roots.is_empty() {
            subjects.push(GoalTreeSubject {
                id: subject.id,
                kind: subject.kind,
                label: subject.label.clone(),
                metadata: subject.metadata.clone(),
                roots,
            });
        }
    }
    GoalTree {
        trace_id: tree.trace_id,
        subjects,
    }
}

fn subject_matches(subject: &GoalTreeSubject, pattern: Option<&str>) -> bool {
    pattern
        .map(|pattern| {
            let pattern = pattern.to_ascii_lowercase();
            subject.label.to_ascii_lowercase().contains(&pattern)
                || subject
                    .metadata
                    .values()
                    .any(|value| value.to_ascii_lowercase().contains(&pattern))
        })
        .unwrap_or(true)
}

fn filter_goal(goal: &GoalTreeGoal, filters: &SolveFilters) -> Option<GoalTreeGoal> {
    if filters.max_depth.is_some_and(|max_depth| goal.depth > max_depth) {
        return None;
    }

    let children =
        goal.children.iter().filter_map(|child| filter_goal(child, filters)).collect::<Vec<_>>();
    let candidates = goal
        .candidates
        .iter()
        .filter(|candidate| {
            filters
                .candidate_kind
                .as_ref()
                .map(|needle| {
                    candidate
                        .kind
                        .label()
                        .to_ascii_lowercase()
                        .contains(&needle.to_ascii_lowercase())
                })
                .unwrap_or(true)
        })
        .cloned()
        .collect::<Vec<_>>();

    let result_matches = filters.result.map(|filter| filter.matches(goal.result)).unwrap_or(true);
    let candidate_matches = filters.candidate_kind.is_none() || !candidates.is_empty();

    if (result_matches && candidate_matches) || !children.is_empty() {
        Some(GoalTreeGoal {
            id: goal.id,
            parent_goal_id: goal.parent_goal_id,
            predicate: goal.predicate.clone(),
            result: goal.result,
            depth: goal.depth,
            semantic_tags: goal.semantic_tags.clone(),
            candidates,
            children,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{SolveFilters, SolveResultFilter, filter_goal_tree};
    use crate::{
        CandidateId, CandidateKind, GoalId, GoalTreeCandidate, PredicateRepr, SemanticTag,
        SubjectId, SubjectKind, TraceId,
    };

    fn sample_tree() -> crate::GoalTree {
        crate::GoalTree {
            trace_id: TraceId::new(1),
            subjects: vec![crate::GoalTreeSubject {
                id: SubjectId::new(1),
                kind: SubjectKind::Predicate,
                label: String::from("RunWriter"),
                metadata: BTreeMap::from([(
                    String::from("owner_path"),
                    String::from("typelude_vm::core::writer_t::RunWriter"),
                )]),
                roots: vec![crate::GoalTreeGoal {
                    id: GoalId::new(1),
                    parent_goal_id: None,
                    predicate: PredicateRepr::DebugText(String::from("<T as Eval>")),
                    result: crate::GoalResult::NoSolution,
                    depth: 0,
                    semantic_tags: vec![SemanticTag::EvalLike],
                    candidates: vec![GoalTreeCandidate {
                        id: CandidateId::new(1),
                        kind: CandidateKind::Impl,
                        result: crate::GoalResult::Success,
                        semantic_tags: Vec::new(),
                        metadata: BTreeMap::new(),
                    }],
                    children: vec![],
                }],
            }],
        }
    }

    #[test]
    fn filters_tree_by_result() {
        let filtered = filter_goal_tree(
            &sample_tree(),
            &SolveFilters {
                result: Some(SolveResultFilter::NoSolution),
                ..SolveFilters::default()
            },
        );
        assert_eq!(filtered.subjects.len(), 1);
        assert_eq!(filtered.subjects[0].roots.len(), 1);
    }

    #[test]
    fn filters_tree_by_subject_metadata() {
        let filtered = filter_goal_tree(
            &sample_tree(),
            &SolveFilters {
                subject: Some(String::from("writer_t")),
                ..SolveFilters::default()
            },
        );
        assert_eq!(filtered.subjects.len(), 1);
    }
}
