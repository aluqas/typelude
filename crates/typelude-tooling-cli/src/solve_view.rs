use std::collections::BTreeMap;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use typelude_tooling_core::{
    CandidateKind, GoalResult, GoalTree, GoalTreeGoal, GoalTreeSubject, PredicateRepr,
    SolveSummary, SolveSummaryEntry,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CompactModeArg {
    Off,
    Basic,
    Aggressive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SolveDiffViewArg {
    Summary,
    Distribution,
    All,
}

#[derive(Debug, Clone, Copy)]
pub struct SolveRenderOptions {
    pub compact: CompactModeArg,
    pub show_raw_kind: bool,
    pub show_full_predicate: bool,
}

impl Default for SolveRenderOptions {
    fn default() -> Self {
        Self {
            compact: CompactModeArg::Basic,
            show_raw_kind: false,
            show_full_predicate: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SolveFilters {
    pub result: Option<SolveResultArg>,
    pub candidate_kind: Option<String>,
    pub max_depth: Option<usize>,
    pub subject: Option<String>,
}

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

pub fn render_solve_tree_text(tree: &GoalTree, options: SolveRenderOptions) -> String {
    let mut lines = vec![format!("trace_id={}", tree.trace_id.value())];
    for subject in &tree.subjects {
        lines.push(format!(
            "subject #{} {:?} :: {}",
            subject.id.value(),
            subject.kind,
            maybe_compact_predicate(&subject.label, options)
        ));
        for root in &subject.roots {
            render_goal(root, 1, &mut lines, options);
        }
    }
    lines.join("\n")
}

pub fn render_summary_text(summary: &SolveSummary, options: SolveRenderOptions) -> String {
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
            lines.push(format!(
                "{} :: {}",
                entry.count,
                maybe_compact_predicate(&entry.label, options)
            ));
        }
    }
    if !summary.top_candidate_kinds.is_empty() {
        lines.push(String::from("top_candidate_kinds:"));
        for entry in &summary.top_candidate_kinds {
            lines.push(format!(
                "{} :: {}",
                entry.count,
                maybe_compact_candidate_kind(&entry.label, options)
            ));
        }
    }

    lines.join("\n")
}

pub fn render_analysis_text(analysis: &SolveAnalysis, options: SolveRenderOptions) -> String {
    let mut lines = vec![render_summary_text(&analysis.summary, options)];

    if !analysis.predicate_distribution.is_empty() {
        lines.push(String::from("predicate_distribution:"));
        for entry in &analysis.predicate_distribution {
            lines.push(format!(
                "{} :: {}",
                entry.count,
                maybe_compact_predicate(&entry.label, options)
            ));
        }
    }

    if !analysis.candidate_kind_distribution.is_empty() {
        lines.push(String::from("candidate_kind_distribution:"));
        for entry in &analysis.candidate_kind_distribution {
            lines.push(format!(
                "{} :: {}",
                entry.count,
                maybe_compact_candidate_kind(&entry.label, options)
            ));
        }
    }

    if !analysis.predicate_family_distribution.is_empty() {
        lines.push(String::from("predicate_families:"));
        for entry in &analysis.predicate_family_distribution {
            lines.push(format!("{} :: {}", entry.count, entry.label));
        }
    }

    if !analysis.candidate_family_distribution.is_empty() {
        lines.push(String::from("candidate_kind_families:"));
        for entry in &analysis.candidate_family_distribution {
            lines.push(format!("{} :: {}", entry.count, entry.label));
        }
    }

    if !analysis.top_roots.is_empty() {
        lines.push(String::from("top_roots:"));
        for root in &analysis.top_roots {
            lines.push(format!(
                "{} :: goal={} result={} goals={} candidates={} max_depth={} unique_predicates={} unique_candidate_kinds={} dominant_predicate_family={} dominant_candidate_family={}",
                maybe_compact_predicate(
                    &format!("{}::{}", root.subject_label, root.predicate),
                    options
                ),
                root.goal_id,
                compact_result(&root.result, options.compact),
                root.goal_count,
                root.candidate_count,
                root.max_depth,
                root.unique_predicates,
                root.unique_candidate_kinds,
                root.dominant_predicate_family,
                root.dominant_candidate_family,
            ));
        }
    }

    lines.join("\n")
}

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

pub fn render_diff_text(
    diff: &SolveAnalysisDiff,
    view: SolveDiffViewArg,
    options: SolveRenderOptions,
) -> String {
    let mut lines = Vec::new();

    if matches!(view, SolveDiffViewArg::Summary | SolveDiffViewArg::All) {
        lines.extend([
            format_metric_diff(
                "subjects",
                diff.left.summary.subjects,
                diff.right.summary.subjects,
                diff.summary_delta.subjects,
            ),
            format_metric_diff(
                "root_goals",
                diff.left.summary.root_goals,
                diff.right.summary.root_goals,
                diff.summary_delta.root_goals,
            ),
            format_metric_diff(
                "goals",
                diff.left.summary.goals,
                diff.right.summary.goals,
                diff.summary_delta.goals,
            ),
            format_metric_diff(
                "candidates",
                diff.left.summary.candidates,
                diff.right.summary.candidates,
                diff.summary_delta.candidates,
            ),
            format_metric_diff(
                "max_goal_depth",
                diff.left.summary.max_goal_depth,
                diff.right.summary.max_goal_depth,
                diff.summary_delta.max_goal_depth,
            ),
            format!(
                "avg_candidates_per_goal: left={:.2} right={:.2} delta={:+.2}",
                diff.left.summary.avg_candidates_per_goal,
                diff.right.summary.avg_candidates_per_goal,
                diff.summary_delta.avg_candidates_per_goal
            ),
            format_metric_diff(
                "result.ok",
                diff.left.summary.result_ok,
                diff.right.summary.result_ok,
                diff.summary_delta.result_ok,
            ),
            format_metric_diff(
                "result.no_solution",
                diff.left.summary.result_no_solution,
                diff.right.summary.result_no_solution,
                diff.summary_delta.result_no_solution,
            ),
            format_metric_diff(
                "result.ambiguous",
                diff.left.summary.result_ambiguous,
                diff.right.summary.result_ambiguous,
                diff.summary_delta.result_ambiguous,
            ),
            format_metric_diff(
                "result.unsupported",
                diff.left.summary.result_unsupported,
                diff.right.summary.result_unsupported,
                diff.summary_delta.result_unsupported,
            ),
        ]);
    }

    if matches!(view, SolveDiffViewArg::Distribution | SolveDiffViewArg::All) {
        if !diff.predicate_distribution_delta.is_empty() {
            lines.push(String::from("predicate_distribution:"));
            for entry in &diff.predicate_distribution_delta {
                lines.push(format!(
                    "{} :: left={} right={} delta={:+}",
                    maybe_compact_predicate(&entry.label, options),
                    entry.left,
                    entry.right,
                    entry.delta
                ));
            }
        }
        if !diff.candidate_kind_distribution_delta.is_empty() {
            lines.push(String::from("candidate_kind_distribution:"));
            for entry in &diff.candidate_kind_distribution_delta {
                lines.push(format!(
                    "{} :: left={} right={} delta={:+}",
                    maybe_compact_candidate_kind(&entry.label, options),
                    entry.left,
                    entry.right,
                    entry.delta
                ));
            }
        }
        if !diff.predicate_family_distribution_delta.is_empty() {
            lines.push(String::from("predicate_families:"));
            for entry in &diff.predicate_family_distribution_delta {
                lines.push(format!(
                    "{} :: left={} right={} delta={:+}",
                    entry.label, entry.left, entry.right, entry.delta
                ));
            }
        }
        if !diff.candidate_family_distribution_delta.is_empty() {
            lines.push(String::from("candidate_kind_families:"));
            for entry in &diff.candidate_family_distribution_delta {
                lines.push(format!(
                    "{} :: left={} right={} delta={:+}",
                    entry.label, entry.left, entry.right, entry.delta
                ));
            }
        }
        if !diff.root_delta.is_empty() {
            lines.push(String::from("top_roots:"));
            for entry in &diff.root_delta {
                lines.push(format!(
                    "{} :: goals left={} right={} delta={:+} candidates left={} right={} delta={:+} depth left={} right={} delta={:+}",
                    maybe_compact_predicate(
                        &format!("{}::{}", entry.subject_label, entry.predicate),
                        options
                    ),
                    entry.left_goal_count,
                    entry.right_goal_count,
                    entry.delta_goal_count,
                    entry.left_candidate_count,
                    entry.right_candidate_count,
                    entry.delta_candidate_count,
                    entry.left_max_depth,
                    entry.right_max_depth,
                    entry.delta_max_depth
                ));
            }
        }
    }

    lines.join("\n")
}

pub fn compact_candidate_kind(raw: &str, mode: CompactModeArg) -> String {
    match mode {
        CompactModeArg::Off => raw.to_string(),
        CompactModeArg::Basic => compact_candidate_kind_basic(raw),
        CompactModeArg::Aggressive => {
            compact_length(&compact_path_segments(&compact_candidate_kind_basic(raw), 3), 120)
        },
    }
}

pub fn compact_predicate(raw: &str, mode: CompactModeArg) -> String {
    match mode {
        CompactModeArg::Off => raw.to_string(),
        CompactModeArg::Basic => compact_predicate_basic(raw),
        CompactModeArg::Aggressive => compact_length(&compact_predicate_aggressive(raw), 120),
    }
}

fn maybe_compact_candidate_kind(raw: &str, options: SolveRenderOptions) -> String {
    if options.show_raw_kind {
        raw.to_string()
    } else {
        compact_candidate_kind(raw, options.compact)
    }
}

fn maybe_compact_predicate(raw: &str, options: SolveRenderOptions) -> String {
    if options.show_full_predicate {
        raw.to_string()
    } else {
        compact_predicate(raw, options.compact)
    }
}

fn compact_candidate_kind_basic(raw: &str) -> String {
    candidate_family(raw)
}

fn compact_predicate_basic(raw: &str) -> String {
    let mut text = raw.trim();
    if let Some(stripped) = text.strip_prefix("Binder { value: ") {
        text = stripped.strip_suffix(", bound_vars: [] }").unwrap_or(stripped);
    }
    let family = predicate_family(text);
    if family != "Other" {
        return format!("{family}(...)");
    }
    text.to_string()
}

fn compact_predicate_aggressive(raw: &str) -> String {
    let basic = compact_predicate_basic(raw);
    compact_path_segments(&basic, 3)
}

fn compact_result(raw: &str, mode: CompactModeArg) -> String {
    match mode {
        CompactModeArg::Off => raw.to_string(),
        CompactModeArg::Basic => {
            raw.split_once('(').map_or_else(|| raw.to_string(), |(head, _)| format!("{head}(...)"))
        },
        CompactModeArg::Aggressive => raw
            .split_once("::")
            .map_or_else(|| compact_length(raw, 120), |(_, tail)| compact_length(tail, 120)),
    }
}

fn compact_goal_result(result: GoalResult, mode: CompactModeArg) -> String {
    compact_result(result.label(), mode)
}

fn compact_length(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        text.to_string()
    } else {
        format!("{}...", &text[..limit.saturating_sub(3)])
    }
}

fn compact_path_segments(text: &str, keep: usize) -> String {
    let parts = text.split("::").collect::<Vec<_>>();
    if parts.len() > keep {
        parts[parts.len() - keep..].join("::")
    } else {
        text.to_string()
    }
}

fn predicate_family(raw: &str) -> String {
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

fn candidate_family(raw: &str) -> String {
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

fn unwrap_binder(raw: &str) -> &str {
    let text = raw.trim();
    if let Some(stripped) = text.strip_prefix("Binder { value: ") {
        stripped.strip_suffix(", bound_vars: [] }").unwrap_or(stripped)
    } else {
        text
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
                    candidate_kind_text(&candidate.kind)
                        .to_ascii_lowercase()
                        .contains(&needle.to_ascii_lowercase())
                })
                .unwrap_or(true)
        })
        .cloned()
        .collect::<Vec<_>>();

    let result_matches =
        filters.result.map(|filter| result_matches(filter, goal.result)).unwrap_or(true);
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

fn result_matches(filter: SolveResultArg, value: GoalResult) -> bool {
    match filter {
        SolveResultArg::Ok => matches!(value, GoalResult::Success),
        SolveResultArg::NoSolution => matches!(value, GoalResult::NoSolution),
        SolveResultArg::Ambiguous => matches!(value, GoalResult::Ambiguous),
        SolveResultArg::Unsupported => {
            matches!(value, GoalResult::Unsupported | GoalResult::Error)
        },
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

fn render_goal(
    goal: &GoalTreeGoal,
    depth: usize,
    lines: &mut Vec<String>,
    options: SolveRenderOptions,
) {
    let indent = "  ".repeat(depth);
    lines.push(format!(
        "{indent}goal #{} result={} depth={} :: {}",
        goal.id.value(),
        compact_goal_result(goal.result, options.compact),
        goal.depth,
        maybe_compact_predicate(&predicate_text(&goal.predicate), options)
    ));
    for candidate in &goal.candidates {
        lines.push(format!(
            "{indent}  candidate #{} kind={} result={}",
            candidate.id.value(),
            maybe_compact_candidate_kind(&candidate_kind_text(&candidate.kind), options),
            compact_goal_result(candidate.result, options.compact)
        ));
    }
    for child in &goal.children {
        render_goal(child, depth + 1, lines, options);
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
                    semantic_tags: Vec::new(),
                    candidates: vec![GoalTreeCandidate {
                        id: CandidateId::new(1),
                        kind: CandidateKind::Unknown(String::from(
                            "TraitCandidate { source: ParamEnv(ImplSource) }",
                        )),
                        result: GoalResult::Success,
                        semantic_tags: Vec::new(),
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
    fn compacts_solver_labels() {
        assert_eq!(
            compact_candidate_kind(
                "TraitCandidate { source: ParamEnv(ImplSource) }",
                CompactModeArg::Basic
            ),
            "TraitCandidate::ParamEnv"
        );
        assert_eq!(
            compact_predicate(
                "Binder { value: TraitPredicate(<T as Eval>, polarity:Positive), bound_vars: [] }",
                CompactModeArg::Basic
            ),
            "TraitPredicate(...)"
        );
    }

    #[test]
    fn aggressive_compaction_truncates_and_shortens_paths() {
        let rendered = compact_predicate(
            "Binder { value: AliasRelate(typelude_vm::very::long::path::Type<Another<ReallyLongThing>>), bound_vars: [] }",
            CompactModeArg::Aggressive,
        );
        assert!(rendered.contains("AliasRelate"));
        assert!(!rendered.contains("typelude_vm::very::long::path"));
    }

    #[test]
    fn filters_tree_by_result() {
        let tree = sample_tree();
        let filtered = filter_goal_tree(&tree, &SolveFilters {
            result: Some(SolveResultArg::NoSolution),
            ..SolveFilters::default()
        });
        assert_eq!(filtered.subjects.len(), 1);
        assert_eq!(filtered.subjects[0].roots.len(), 1);
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
        right.predicate_distribution.push(LabelCount {
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
    fn raw_override_keeps_original_text() {
        let rendered = render_solve_tree_text(&sample_tree(), SolveRenderOptions {
            compact: CompactModeArg::Basic,
            show_raw_kind: true,
            show_full_predicate: true,
        });
        assert!(rendered.contains("Binder { value: TraitPredicate"));
        assert!(rendered.contains("TraitCandidate { source: ParamEnv"));
    }
}
