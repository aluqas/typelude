use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    CandidateKind, GoalResult, GoalTree, GoalTreeGoal, GoalTreeSubject, PredicateRepr,
    SolveSummary, SolveSummaryEntry, Trace, TraceEventKind,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelCount {
    pub label: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootHotspot {
    pub subject_label: String,
    pub goal_id: u64,
    pub predicate: String,
    pub result: String,
    pub goal_count: usize,
    pub candidate_count: usize,
    pub max_depth: usize,
    #[serde(default)]
    pub unique_predicates: usize,
    #[serde(default)]
    pub unique_candidate_kinds: usize,
    #[serde(default)]
    pub dominant_predicate_family: String,
    #[serde(default)]
    pub dominant_candidate_family: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolveAnalysis {
    pub summary: SolveSummary,
    #[serde(default)]
    pub predicate_distribution: Vec<LabelCount>,
    #[serde(default)]
    pub candidate_kind_distribution: Vec<LabelCount>,
    #[serde(default)]
    pub predicate_family_distribution: Vec<LabelCount>,
    #[serde(default)]
    pub candidate_family_distribution: Vec<LabelCount>,
    #[serde(default)]
    pub top_roots: Vec<RootHotspot>,
}

impl SolveAnalysis {
    #[must_use]
    pub fn from_summary(summary: SolveSummary) -> Self {
        Self {
            summary,
            predicate_distribution: Vec::new(),
            candidate_kind_distribution: Vec::new(),
            predicate_family_distribution: Vec::new(),
            candidate_family_distribution: Vec::new(),
            top_roots: Vec::new(),
        }
    }

    #[must_use]
    pub fn deepest_roots(&self) -> Vec<RootHotspot> {
        let mut roots = self.top_roots.clone();
        roots.sort_by(|left, right| {
            right
                .max_depth
                .cmp(&left.max_depth)
                .then_with(|| right.goal_count.cmp(&left.goal_count))
                .then_with(|| left.goal_id.cmp(&right.goal_id))
        });
        roots
    }

    #[must_use]
    pub fn branchiest_roots(&self) -> Vec<RootHotspot> {
        let mut roots = self.top_roots.clone();
        roots.sort_by(|left, right| {
            right
                .candidate_count
                .cmp(&left.candidate_count)
                .then_with(|| right.goal_count.cmp(&left.goal_count))
                .then_with(|| left.goal_id.cmp(&right.goal_id))
        });
        roots
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LabelDelta {
    pub label: String,
    pub left: usize,
    pub right: usize,
    pub delta: isize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RootHotspotDelta {
    pub subject_label: String,
    pub predicate: String,
    pub left_goal_count: usize,
    pub right_goal_count: usize,
    pub delta_goal_count: isize,
    pub left_candidate_count: usize,
    pub right_candidate_count: usize,
    pub delta_candidate_count: isize,
    pub left_max_depth: usize,
    pub right_max_depth: usize,
    pub delta_max_depth: isize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SolveSummaryDelta {
    pub subjects: isize,
    pub root_goals: isize,
    pub goals: isize,
    pub candidates: isize,
    pub max_goal_depth: isize,
    pub avg_candidates_per_goal: f64,
    pub result_ok: isize,
    pub result_no_solution: isize,
    pub result_ambiguous: isize,
    pub result_unsupported: isize,
}

#[derive(Debug, Clone, Serialize)]
pub struct SolveAnalysisDiff {
    pub left: SolveAnalysis,
    pub right: SolveAnalysis,
    pub summary_delta: SolveSummaryDelta,
    pub predicate_distribution_delta: Vec<LabelDelta>,
    pub candidate_kind_distribution_delta: Vec<LabelDelta>,
    pub predicate_family_distribution_delta: Vec<LabelDelta>,
    pub candidate_family_distribution_delta: Vec<LabelDelta>,
    pub root_delta: Vec<RootHotspotDelta>,
}

#[must_use]
pub fn build_solve_analysis(
    tree: &GoalTree,
    unsupported_count: usize,
    top_n: usize,
) -> SolveAnalysis {
    let mut predicate_counts = BTreeMap::<String, usize>::new();
    let mut candidate_counts = BTreeMap::<String, usize>::new();
    let mut predicate_family_counts = BTreeMap::<String, usize>::new();
    let mut candidate_family_counts = BTreeMap::<String, usize>::new();
    let mut result_ok = 0_usize;
    let mut result_no_solution = 0_usize;
    let mut result_ambiguous = 0_usize;
    let mut max_goal_depth = 0_usize;
    let mut goal_count = 0_usize;
    let mut candidate_count = 0_usize;
    let mut root_goal_count = 0_usize;
    let mut top_roots = Vec::new();

    for subject in &tree.subjects {
        root_goal_count += subject.roots.len();
        for root in &subject.roots {
            walk_goal(
                root,
                &mut predicate_counts,
                &mut candidate_counts,
                &mut predicate_family_counts,
                &mut candidate_family_counts,
                &mut result_ok,
                &mut result_no_solution,
                &mut result_ambiguous,
                &mut max_goal_depth,
                &mut goal_count,
                &mut candidate_count,
            );
            top_roots.push(root_hotspot(subject, root));
        }
    }

    top_roots.sort_by(|left, right| {
        right
            .goal_count
            .cmp(&left.goal_count)
            .then_with(|| right.candidate_count.cmp(&left.candidate_count))
            .then_with(|| left.goal_id.cmp(&right.goal_id))
    });
    top_roots.truncate(top_n);

    let summary = SolveSummary {
        subjects: tree.subjects.len(),
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
        result_unsupported: unsupported_count,
        top_predicates: to_summary_entries(&predicate_counts, 5),
        top_candidate_kinds: to_summary_entries(&candidate_counts, 5),
    };

    SolveAnalysis {
        summary,
        predicate_distribution: top_counts(predicate_counts, top_n),
        candidate_kind_distribution: top_counts(candidate_counts, top_n),
        predicate_family_distribution: top_counts(predicate_family_counts, top_n),
        candidate_family_distribution: top_counts(candidate_family_counts, top_n),
        top_roots,
    }
}

#[must_use]
pub fn diff_analysis(left: &SolveAnalysis, right: &SolveAnalysis) -> SolveAnalysisDiff {
    SolveAnalysisDiff {
        left: left.clone(),
        right: right.clone(),
        summary_delta: SolveSummaryDelta {
            subjects: right.summary.subjects as isize - left.summary.subjects as isize,
            root_goals: right.summary.root_goals as isize - left.summary.root_goals as isize,
            goals: right.summary.goals as isize - left.summary.goals as isize,
            candidates: right.summary.candidates as isize - left.summary.candidates as isize,
            max_goal_depth: right.summary.max_goal_depth as isize
                - left.summary.max_goal_depth as isize,
            avg_candidates_per_goal: right.summary.avg_candidates_per_goal
                - left.summary.avg_candidates_per_goal,
            result_ok: right.summary.result_ok as isize - left.summary.result_ok as isize,
            result_no_solution: right.summary.result_no_solution as isize
                - left.summary.result_no_solution as isize,
            result_ambiguous: right.summary.result_ambiguous as isize
                - left.summary.result_ambiguous as isize,
            result_unsupported: right.summary.result_unsupported as isize
                - left.summary.result_unsupported as isize,
        },
        predicate_distribution_delta: diff_label_counts(
            &left.predicate_distribution,
            &right.predicate_distribution,
        ),
        candidate_kind_distribution_delta: diff_label_counts(
            &left.candidate_kind_distribution,
            &right.candidate_kind_distribution,
        ),
        predicate_family_distribution_delta: diff_label_counts(
            &left.predicate_family_distribution,
            &right.predicate_family_distribution,
        ),
        candidate_family_distribution_delta: diff_label_counts(
            &left.candidate_family_distribution,
            &right.candidate_family_distribution,
        ),
        root_delta: diff_roots(&left.top_roots, &right.top_roots),
    }
}

#[must_use]
pub fn unsupported_error_count(trace: &Trace, tree: &GoalTree) -> usize {
    let subject_ids = tree.subjects.iter().map(|subject| subject.id).collect::<BTreeSet<_>>();
    trace
        .events
        .iter()
        .filter(|event| event.kind() == TraceEventKind::ErrorRaised)
        .filter(|event| {
            event.subject_id().is_some_and(|subject_id| subject_ids.contains(&subject_id))
        })
        .count()
}

#[must_use]
pub fn predicate_family(raw: &str) -> String {
    let text = unwrap_binder(raw);
    for family in [
        "TraitPredicate",
        "AliasRelate",
        "NormalizesTo",
        "Subtype",
        "Projection",
        "RegionOutlives",
        "TypeOutlives",
        "ConstArgHasType",
        "WellFormed",
    ] {
        if text.contains(&format!("{family}(")) {
            return family.to_string();
        }
    }
    if let Some((head, _)) = text.split_once('(') {
        let head = head.trim();
        if !head.is_empty() {
            return head.to_string();
        }
    }
    String::from("Other")
}

#[must_use]
pub fn candidate_family(raw: &str) -> String {
    if raw.contains("TraitCandidate") && raw.contains("ParamEnv") {
        String::from("TraitCandidate::ParamEnv")
    } else if raw.contains("TraitCandidate") && raw.contains("Impl") {
        String::from("TraitCandidate::Impl")
    } else if raw.contains("TraitCandidate") {
        String::from("TraitCandidate")
    } else if let Some(head) = raw.split(" {").next() {
        head.split_whitespace().next().unwrap_or(raw).to_string()
    } else {
        raw.to_string()
    }
}

#[must_use]
pub fn unwrap_binder(raw: &str) -> &str {
    let text = raw.trim();
    if let Some(stripped) = text.strip_prefix("Binder { value: ") {
        stripped.strip_suffix(", bound_vars: [] }").unwrap_or(stripped)
    } else {
        text
    }
}

fn walk_goal(
    goal: &GoalTreeGoal,
    predicate_counts: &mut BTreeMap<String, usize>,
    candidate_counts: &mut BTreeMap<String, usize>,
    predicate_family_counts: &mut BTreeMap<String, usize>,
    candidate_family_counts: &mut BTreeMap<String, usize>,
    result_ok: &mut usize,
    result_no_solution: &mut usize,
    result_ambiguous: &mut usize,
    max_goal_depth: &mut usize,
    goal_count: &mut usize,
    candidate_count: &mut usize,
) {
    *goal_count += 1;
    *max_goal_depth = (*max_goal_depth).max(goal.depth);
    let predicate = predicate_text(&goal.predicate);
    *predicate_counts.entry(predicate.clone()).or_default() += 1;
    *predicate_family_counts.entry(predicate_family(&predicate)).or_default() += 1;

    match goal.result {
        GoalResult::Success => *result_ok += 1,
        GoalResult::NoSolution => *result_no_solution += 1,
        GoalResult::Ambiguous => *result_ambiguous += 1,
        GoalResult::Unsupported | GoalResult::Error => {},
    }

    for candidate in &goal.candidates {
        *candidate_count += 1;
        let kind = candidate_kind_text(&candidate.kind);
        *candidate_counts.entry(kind.clone()).or_default() += 1;
        *candidate_family_counts.entry(candidate_family(&kind)).or_default() += 1;
    }

    for child in &goal.children {
        walk_goal(
            child,
            predicate_counts,
            candidate_counts,
            predicate_family_counts,
            candidate_family_counts,
            result_ok,
            result_no_solution,
            result_ambiguous,
            max_goal_depth,
            goal_count,
            candidate_count,
        );
    }
}

fn root_hotspot(subject: &GoalTreeSubject, root: &GoalTreeGoal) -> RootHotspot {
    let mut goal_count = 0;
    let mut candidate_count = 0;
    let mut max_depth = 0;
    let mut predicate_counts = BTreeMap::<String, usize>::new();
    let mut candidate_kind_counts = BTreeMap::<String, usize>::new();
    let mut predicate_family_counts = BTreeMap::<String, usize>::new();
    let mut candidate_family_counts = BTreeMap::<String, usize>::new();
    collect_root_stats(
        root,
        &mut goal_count,
        &mut candidate_count,
        &mut max_depth,
        &mut predicate_counts,
        &mut candidate_kind_counts,
        &mut predicate_family_counts,
        &mut candidate_family_counts,
    );
    RootHotspot {
        subject_label: subject.label.clone(),
        goal_id: root.id.value(),
        predicate: predicate_text(&root.predicate),
        result: String::from(root.result.label()),
        goal_count,
        candidate_count,
        max_depth,
        unique_predicates: predicate_counts.len(),
        unique_candidate_kinds: candidate_kind_counts.len(),
        dominant_predicate_family: dominant_label(&predicate_family_counts),
        dominant_candidate_family: dominant_label(&candidate_family_counts),
    }
}

fn collect_root_stats(
    goal: &GoalTreeGoal,
    goal_count: &mut usize,
    candidate_count: &mut usize,
    max_depth: &mut usize,
    predicate_counts: &mut BTreeMap<String, usize>,
    candidate_kind_counts: &mut BTreeMap<String, usize>,
    predicate_family_counts: &mut BTreeMap<String, usize>,
    candidate_family_counts: &mut BTreeMap<String, usize>,
) {
    *goal_count += 1;
    *candidate_count += goal.candidates.len();
    *max_depth = (*max_depth).max(goal.depth);
    let predicate = predicate_text(&goal.predicate);
    *predicate_counts.entry(predicate.clone()).or_default() += 1;
    *predicate_family_counts.entry(predicate_family(&predicate)).or_default() += 1;
    for candidate in &goal.candidates {
        let kind = candidate_kind_text(&candidate.kind);
        *candidate_kind_counts.entry(kind.clone()).or_default() += 1;
        *candidate_family_counts.entry(candidate_family(&kind)).or_default() += 1;
    }
    for child in &goal.children {
        collect_root_stats(
            child,
            goal_count,
            candidate_count,
            max_depth,
            predicate_counts,
            candidate_kind_counts,
            predicate_family_counts,
            candidate_family_counts,
        );
    }
}

fn dominant_label(counts: &BTreeMap<String, usize>) -> String {
    counts
        .iter()
        .max_by(|left, right| left.1.cmp(right.1).then_with(|| right.0.cmp(left.0)))
        .map(|(label, _)| label.clone())
        .unwrap_or_else(|| String::from("None"))
}

fn predicate_text(predicate: &PredicateRepr) -> String {
    predicate.debug_text()
}

fn candidate_kind_text(kind: &CandidateKind) -> String {
    kind.label()
}

fn to_summary_entries(counts: &BTreeMap<String, usize>, limit: usize) -> Vec<SolveSummaryEntry> {
    top_counts(counts.clone(), limit)
        .into_iter()
        .map(|entry| SolveSummaryEntry {
            label: entry.label,
            count: entry.count,
        })
        .collect()
}

fn top_counts(counts: BTreeMap<String, usize>, limit: usize) -> Vec<LabelCount> {
    let mut entries = counts
        .into_iter()
        .map(|(label, count)| LabelCount {
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

fn diff_label_counts(left: &[LabelCount], right: &[LabelCount]) -> Vec<LabelDelta> {
    let mut counts = BTreeMap::<String, (usize, usize)>::new();
    for entry in left {
        counts.entry(entry.label.clone()).or_default().0 = entry.count;
    }
    for entry in right {
        counts.entry(entry.label.clone()).or_default().1 = entry.count;
    }

    let mut deltas = counts
        .into_iter()
        .map(|(label, (left, right))| LabelDelta {
            label,
            left,
            right,
            delta: right as isize - left as isize,
        })
        .collect::<Vec<_>>();
    deltas.sort_by(|left, right| {
        right
            .delta
            .unsigned_abs()
            .cmp(&left.delta.unsigned_abs())
            .then_with(|| left.label.cmp(&right.label))
    });
    deltas
}

fn diff_roots(left: &[RootHotspot], right: &[RootHotspot]) -> Vec<RootHotspotDelta> {
    let mut roots =
        BTreeMap::<(String, String), (Option<&RootHotspot>, Option<&RootHotspot>)>::new();
    for entry in left {
        roots.entry((entry.subject_label.clone(), entry.predicate.clone())).or_default().0 =
            Some(entry);
    }
    for entry in right {
        roots.entry((entry.subject_label.clone(), entry.predicate.clone())).or_default().1 =
            Some(entry);
    }

    let mut deltas = roots
        .into_iter()
        .map(|((subject_label, predicate), (left, right))| {
            let left_goal_count = left.map_or(0, |entry| entry.goal_count);
            let right_goal_count = right.map_or(0, |entry| entry.goal_count);
            let left_candidate_count = left.map_or(0, |entry| entry.candidate_count);
            let right_candidate_count = right.map_or(0, |entry| entry.candidate_count);
            let left_max_depth = left.map_or(0, |entry| entry.max_depth);
            let right_max_depth = right.map_or(0, |entry| entry.max_depth);
            RootHotspotDelta {
                subject_label,
                predicate,
                left_goal_count,
                right_goal_count,
                delta_goal_count: right_goal_count as isize - left_goal_count as isize,
                left_candidate_count,
                right_candidate_count,
                delta_candidate_count: right_candidate_count as isize
                    - left_candidate_count as isize,
                left_max_depth,
                right_max_depth,
                delta_max_depth: right_max_depth as isize - left_max_depth as isize,
            }
        })
        .collect::<Vec<_>>();
    deltas.sort_by(|left, right| {
        right
            .delta_goal_count
            .unsigned_abs()
            .cmp(&left.delta_goal_count.unsigned_abs())
            .then_with(|| {
                right
                    .delta_candidate_count
                    .unsigned_abs()
                    .cmp(&left.delta_candidate_count.unsigned_abs())
            })
            .then_with(|| left.subject_label.cmp(&right.subject_label))
            .then_with(|| left.predicate.cmp(&right.predicate))
    });
    deltas
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        SolveAnalysis, build_solve_analysis, candidate_family, diff_analysis, predicate_family,
        unsupported_error_count,
    };
    use crate::{
        CandidateId, CandidateKind, GoalId, GoalResult, GoalTree, GoalTreeCandidate, GoalTreeGoal,
        PredicateRepr, SemanticTag, SubjectId, SubjectKind, Trace, TraceEvent, TraceId,
        TracePayload,
    };

    fn sample_tree() -> GoalTree {
        GoalTree {
            trace_id: TraceId::new(1),
            subjects: vec![crate::GoalTreeSubject {
                id: SubjectId::new(1),
                kind: SubjectKind::Predicate,
                label: String::from("typelude_vm::core::writer_t::RunWriter"),
                metadata: BTreeMap::new(),
                roots: vec![GoalTreeGoal {
                    id: GoalId::new(1),
                    parent_goal_id: None,
                    predicate: PredicateRepr::DebugText(String::from(
                        "Binder { value: TraitPredicate(<very::long::path::T as Eval>, polarity:Positive), bound_vars: [] }",
                    )),
                    result: GoalResult::NoSolution,
                    depth: 0,
                    semantic_tags: vec![SemanticTag::EvalLike],
                    candidates: vec![GoalTreeCandidate {
                        id: CandidateId::new(1),
                        kind: CandidateKind::Unknown(String::from(
                            "TraitCandidate { source: ParamEnv(ImplSource) }",
                        )),
                        result: GoalResult::Success,
                        semantic_tags: vec![SemanticTag::HelperDispatchLike],
                        metadata: BTreeMap::new(),
                    }],
                    children: vec![GoalTreeGoal {
                        id: GoalId::new(2),
                        parent_goal_id: Some(GoalId::new(1)),
                        predicate: PredicateRepr::DebugText(String::from(
                            "NormalizesTo(very::long::assoc::Type)",
                        )),
                        result: GoalResult::Success,
                        depth: 1,
                        semantic_tags: Vec::new(),
                        candidates: vec![],
                        children: vec![],
                    }],
                }],
            }],
        }
    }

    #[test]
    fn builds_analysis_with_top_roots() {
        let analysis = build_solve_analysis(&sample_tree(), 0, 10);
        assert_eq!(analysis.summary.subjects, 1);
        assert_eq!(analysis.top_roots.len(), 1);
        assert_eq!(analysis.top_roots[0].goal_count, 2);
        assert!(!analysis.top_roots[0].dominant_predicate_family.is_empty());
        assert_eq!(analysis.candidate_family_distribution[0].label, "TraitCandidate::ParamEnv");
    }

    #[test]
    fn diffs_analysis_scalars_and_distribution() {
        let left = build_solve_analysis(&sample_tree(), 0, 10);
        let mut right = left.clone();
        right.summary.goals = 4;
        right.predicate_distribution.push(super::LabelCount {
            label: String::from("ExtraPredicate"),
            count: 2,
        });
        let diff = diff_analysis(&left, &right);
        assert_eq!(diff.summary_delta.goals, 2);
        assert!(
            diff.predicate_distribution_delta.iter().any(|entry| entry.label == "ExtraPredicate")
        );
    }

    #[test]
    fn family_helpers_normalize_solver_shapes() {
        assert_eq!(
            predicate_family("Binder { value: AliasRelate(<T as Foo>, Bar), bound_vars: [] }"),
            "AliasRelate"
        );
        assert_eq!(
            candidate_family("TraitCandidate { source: ParamEnv(ImplSource) }"),
            "TraitCandidate::ParamEnv"
        );
    }

    #[test]
    fn reports_unsupported_for_filtered_subjects() {
        let tree = sample_tree();
        let mut trace = Trace::new(TraceId::new(1));
        trace.push(TraceEvent::new(
            crate::EventId::new(1),
            TracePayload::ErrorRaised(crate::ErrorRaised {
                hook_id: None,
                subject_id: Some(SubjectId::new(1)),
                goal_id: None,
                result: GoalResult::Error,
                message: String::from("boom"),
            }),
        ));
        assert_eq!(unsupported_error_count(&trace, &tree), 1);
    }

    #[test]
    fn derives_secondary_root_rankings() {
        let analysis = build_solve_analysis(&sample_tree(), 0, 10);
        assert_eq!(analysis.deepest_roots().len(), analysis.top_roots.len());
        assert_eq!(analysis.branchiest_roots().len(), analysis.top_roots.len());
    }

    #[test]
    fn can_build_empty_analysis_from_summary() {
        let analysis =
            SolveAnalysis::from_summary(sample_tree().summarize(&Trace::new(TraceId::new(1))));
        assert!(analysis.top_roots.is_empty());
    }
}
