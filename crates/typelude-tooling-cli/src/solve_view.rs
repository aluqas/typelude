use std::collections::BTreeMap;

use clap::ValueEnum;
use serde::Serialize;
use typelude_tooling_core::{GoalTree, GoalTreeGoal, GoalTreeSubject, SolveSummary};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SolveResultArg {
    Ok,
    NoSolution,
    Ambiguous,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SolveViewArg {
    Tree,
    Summary,
}

#[derive(Debug, Clone, Default)]
pub struct SolveFilters {
    pub result: Option<SolveResultArg>,
    pub candidate_kind: Option<String>,
    pub max_depth: Option<usize>,
    pub subject: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SolveSummaryDiff {
    pub left: SolveSummary,
    pub right: SolveSummary,
    pub delta: SolveSummaryDelta,
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
    pub top_predicates: Vec<LabelDelta>,
    pub top_candidate_kinds: Vec<LabelDelta>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LabelDelta {
    pub label: String,
    pub left: usize,
    pub right: usize,
    pub delta: isize,
}

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

pub fn summarize_filtered_tree(tree: &GoalTree, unsupported_count: usize) -> SolveSummary {
    let mut predicate_counts = BTreeMap::<String, usize>::new();
    let mut candidate_counts = BTreeMap::<String, usize>::new();
    let mut result_ok = 0_usize;
    let mut result_no_solution = 0_usize;
    let mut result_ambiguous = 0_usize;
    let mut max_goal_depth = 0_usize;
    let mut goal_count = 0_usize;
    let mut candidate_count = 0_usize;
    let mut root_goal_count = 0_usize;

    for subject in &tree.subjects {
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

    SolveSummary {
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
        top_predicates: top_counts(predicate_counts, 5),
        top_candidate_kinds: top_counts(candidate_counts, 5),
    }
}

pub fn render_solve_tree_text(tree: &GoalTree) -> String {
    let mut lines = vec![format!("trace_id={}", tree.trace_id.value())];
    for subject in &tree.subjects {
        lines.push(format!(
            "subject #{} {:?} :: {}",
            subject.id.value(),
            subject.kind,
            subject.label
        ));
        for root in &subject.roots {
            render_goal(root, 1, &mut lines);
        }
    }
    lines.join("\n")
}

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
            lines.push(format!("{} :: {}", entry.count, compact_predicate(&entry.label)));
        }
    }
    if !summary.top_candidate_kinds.is_empty() {
        lines.push(String::from("top_candidate_kinds:"));
        for entry in &summary.top_candidate_kinds {
            lines.push(format!("{} :: {}", entry.count, compact_candidate_kind(&entry.label)));
        }
    }

    lines.join("\n")
}

pub fn render_diff_text(diff: &SolveSummaryDiff) -> String {
    let mut lines = vec![
        format_metric_diff(
            "subjects",
            diff.left.subjects,
            diff.right.subjects,
            diff.delta.subjects,
        ),
        format_metric_diff(
            "root_goals",
            diff.left.root_goals,
            diff.right.root_goals,
            diff.delta.root_goals,
        ),
        format_metric_diff("goals", diff.left.goals, diff.right.goals, diff.delta.goals),
        format_metric_diff(
            "candidates",
            diff.left.candidates,
            diff.right.candidates,
            diff.delta.candidates,
        ),
        format_metric_diff(
            "max_goal_depth",
            diff.left.max_goal_depth,
            diff.right.max_goal_depth,
            diff.delta.max_goal_depth,
        ),
        format!(
            "avg_candidates_per_goal: left={:.2} right={:.2} delta={:+.2}",
            diff.left.avg_candidates_per_goal,
            diff.right.avg_candidates_per_goal,
            diff.delta.avg_candidates_per_goal,
        ),
        format_metric_diff(
            "result.ok",
            diff.left.result_ok,
            diff.right.result_ok,
            diff.delta.result_ok,
        ),
        format_metric_diff(
            "result.no_solution",
            diff.left.result_no_solution,
            diff.right.result_no_solution,
            diff.delta.result_no_solution,
        ),
        format_metric_diff(
            "result.ambiguous",
            diff.left.result_ambiguous,
            diff.right.result_ambiguous,
            diff.delta.result_ambiguous,
        ),
        format_metric_diff(
            "result.unsupported",
            diff.left.result_unsupported,
            diff.right.result_unsupported,
            diff.delta.result_unsupported,
        ),
    ];

    if !diff.delta.top_predicates.is_empty() {
        lines.push(String::from("top_predicates:"));
        for entry in &diff.delta.top_predicates {
            lines.push(format!(
                "{} :: left={} right={} delta={:+}",
                compact_predicate(&entry.label),
                entry.left,
                entry.right,
                entry.delta
            ));
        }
    }
    if !diff.delta.top_candidate_kinds.is_empty() {
        lines.push(String::from("top_candidate_kinds:"));
        for entry in &diff.delta.top_candidate_kinds {
            lines.push(format!(
                "{} :: left={} right={} delta={:+}",
                compact_candidate_kind(&entry.label),
                entry.left,
                entry.right,
                entry.delta
            ));
        }
    }

    lines.join("\n")
}

pub fn diff_summaries(left: &SolveSummary, right: &SolveSummary) -> SolveSummaryDiff {
    SolveSummaryDiff {
        left: left.clone(),
        right: right.clone(),
        delta: SolveSummaryDelta {
            subjects: right.subjects as isize - left.subjects as isize,
            root_goals: right.root_goals as isize - left.root_goals as isize,
            goals: right.goals as isize - left.goals as isize,
            candidates: right.candidates as isize - left.candidates as isize,
            max_goal_depth: right.max_goal_depth as isize - left.max_goal_depth as isize,
            avg_candidates_per_goal: right.avg_candidates_per_goal - left.avg_candidates_per_goal,
            result_ok: right.result_ok as isize - left.result_ok as isize,
            result_no_solution: right.result_no_solution as isize
                - left.result_no_solution as isize,
            result_ambiguous: right.result_ambiguous as isize - left.result_ambiguous as isize,
            result_unsupported: right.result_unsupported as isize
                - left.result_unsupported as isize,
            top_predicates: diff_entries(&left.top_predicates, &right.top_predicates),
            top_candidate_kinds: diff_entries(
                &left.top_candidate_kinds,
                &right.top_candidate_kinds,
            ),
        },
    }
}

pub fn compact_candidate_kind(raw: &str) -> String {
    if raw.contains("TraitCandidate") && raw.contains("ParamEnv") {
        String::from("TraitCandidate::ParamEnv")
    } else if let Some(head) = raw.split(" {").next() {
        head.split_whitespace().next().unwrap_or(raw).to_string()
    } else {
        raw.to_string()
    }
}

pub fn compact_predicate(raw: &str) -> String {
    let mut text = raw.trim();
    if let Some(stripped) = text.strip_prefix("Binder { value: ") {
        text = stripped.strip_suffix(", bound_vars: [] }").unwrap_or(stripped);
    }
    for head in ["TraitPredicate(", "AliasRelate(", "NormalizesTo("] {
        if text.contains(head) {
            let prefix = head.trim_end_matches('(');
            return format!("{prefix}(...)");
        }
    }
    text.to_string()
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
                    candidate.kind.to_ascii_lowercase().contains(&needle.to_ascii_lowercase())
                })
                .unwrap_or(true)
        })
        .cloned()
        .collect::<Vec<_>>();

    let result_matches =
        filters.result.map(|filter| result_matches(filter, &goal.result)).unwrap_or(true);
    let candidate_matches = filters.candidate_kind.is_none() || !candidates.is_empty();

    if (result_matches && candidate_matches) || !children.is_empty() {
        Some(GoalTreeGoal {
            id: goal.id,
            parent_goal_id: goal.parent_goal_id,
            predicate: goal.predicate.clone(),
            result: goal.result.clone(),
            depth: goal.depth,
            candidates,
            children,
        })
    } else {
        None
    }
}

fn result_matches(filter: SolveResultArg, result: &str) -> bool {
    let lowered = result.to_ascii_lowercase();
    match filter {
        SolveResultArg::Ok => lowered.contains("ok"),
        SolveResultArg::NoSolution => {
            lowered.contains("no_solution")
                || lowered.contains("no solution")
                || lowered.contains("nosolution")
        },
        SolveResultArg::Ambiguous => lowered.contains("ambiguous"),
        SolveResultArg::Unsupported => lowered.contains("unsupported"),
    }
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
    } else if lower.contains("ok") {
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

fn top_counts(
    counts: BTreeMap<String, usize>,
    limit: usize,
) -> Vec<typelude_tooling_core::SolveSummaryEntry> {
    let mut entries = counts
        .into_iter()
        .map(|(label, count)| typelude_tooling_core::SolveSummaryEntry {
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

fn diff_entries(
    left: &[typelude_tooling_core::SolveSummaryEntry],
    right: &[typelude_tooling_core::SolveSummaryEntry],
) -> Vec<LabelDelta> {
    let mut labels = BTreeMap::<String, (usize, usize)>::new();
    for entry in left {
        labels.entry(entry.label.clone()).or_default().0 = entry.count;
    }
    for entry in right {
        labels.entry(entry.label.clone()).or_default().1 = entry.count;
    }
    let mut entries = labels
        .into_iter()
        .map(|(label, (left, right))| LabelDelta {
            label,
            left,
            right,
            delta: right as isize - left as isize,
        })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| b.delta.abs().cmp(&a.delta.abs()).then_with(|| a.label.cmp(&b.label)));
    entries.truncate(5);
    entries
}

fn render_goal(goal: &GoalTreeGoal, depth: usize, lines: &mut Vec<String>) {
    let indent = "  ".repeat(depth);
    lines.push(format!(
        "{indent}goal #{} result={} depth={} :: {}",
        goal.id.value(),
        goal.result,
        goal.depth,
        compact_predicate(&goal.predicate)
    ));
    for candidate in &goal.candidates {
        lines.push(format!(
            "{indent}  candidate #{} kind={} result={}",
            candidate.id.value(),
            compact_candidate_kind(&candidate.kind),
            candidate.result
        ));
    }
    for child in &goal.children {
        render_goal(child, depth + 1, lines);
    }
}

fn format_metric_diff(name: &str, left: usize, right: usize, delta: isize) -> String {
    format!("{name}: left={left} right={right} delta={delta:+}")
}

#[cfg(test)]
mod tests {
    use typelude_tooling_core::{
        CandidateId, GoalId, GoalTreeCandidate, SubjectId, SubjectKind, TraceId,
    };

    use super::*;

    fn sample_tree() -> GoalTree {
        GoalTree {
            trace_id: TraceId::new(1),
            subjects: vec![GoalTreeSubject {
                id: SubjectId::new(1),
                kind: SubjectKind::Predicate,
                label: String::from("RunWriter"),
                metadata: BTreeMap::new(),
                roots: vec![GoalTreeGoal {
                    id: GoalId::new(1),
                    parent_goal_id: None,
                    predicate: String::from(
                        "Binder { value: TraitPredicate(<T as Eval>, polarity:Positive), bound_vars: [] }",
                    ),
                    result: String::from("NoSolution"),
                    depth: 0,
                    candidates: vec![GoalTreeCandidate {
                        id: CandidateId::new(1),
                        kind: String::from("RigidAlias { result: Ok(...) }"),
                        result: String::from("Ok(Yes)"),
                        metadata: BTreeMap::new(),
                    }],
                    children: vec![],
                }],
            }],
        }
    }

    #[test]
    fn compacts_solver_labels() {
        assert_eq!(compact_candidate_kind("RigidAlias { result: Ok(...) }"), "RigidAlias");
        assert_eq!(
            compact_predicate(
                "Binder { value: TraitPredicate(<T as Eval>, polarity:Positive), bound_vars: [] }"
            ),
            "TraitPredicate(...)"
        );
    }

    #[test]
    fn filters_tree_by_result() {
        let tree = sample_tree();
        let filtered = filter_goal_tree(
            &tree,
            &SolveFilters {
                result: Some(SolveResultArg::NoSolution),
                ..SolveFilters::default()
            },
        );
        assert_eq!(filtered.subjects.len(), 1);
        assert_eq!(filtered.subjects[0].roots.len(), 1);
    }

    #[test]
    fn diffs_summary_scalars() {
        let left = SolveSummary {
            subjects: 1,
            root_goals: 1,
            goals: 2,
            candidates: 2,
            max_goal_depth: 1,
            avg_candidates_per_goal: 1.0,
            result_ok: 1,
            result_no_solution: 1,
            result_ambiguous: 0,
            result_unsupported: 0,
            top_predicates: vec![],
            top_candidate_kinds: vec![],
        };
        let right = SolveSummary {
            goals: 4,
            candidates: 3,
            ..left.clone()
        };
        let diff = diff_summaries(&left, &right);
        assert_eq!(diff.delta.goals, 2);
        assert_eq!(diff.delta.candidates, 1);
    }
}
