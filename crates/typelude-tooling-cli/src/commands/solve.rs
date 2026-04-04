use std::path::PathBuf;

use typelude_tooling_core::{GoalTree, SolveSummary, ToolingResult, Trace, TraceId};

use crate::solve_view::{
    SolveAnalysis,
    SolveDiffViewArg,
    SolveFilters,
    SolveRenderOptions,
    SolveViewArg,
    build_solve_analysis,
    diff_analysis,
    filter_goal_tree,
    render_analysis_text,
    render_diff_text,
    render_solve_tree_text,
    render_summary_text,
};
use crate::{HookArg, OwnerMatchArg, OutputModeArg};

use super::collect::{collect_trace, QueryKindArg};
use super::trace_io::read_trace;

pub(crate) fn run_solve_tree(
    input: PathBuf,
    output: OutputModeArg,
    render: SolveRenderOptions,
    filters: SolveFilters,
) -> ToolingResult<String> {
    let trace = read_trace(&input)?;
    let tree = filter_goal_tree(&GoalTree::from_trace(&trace)?, &filters);
    match output {
        OutputModeArg::Text => Ok(render_solve_tree_text(&tree, render)),
        OutputModeArg::Json => Ok(serde_json::to_string_pretty(&tree)?),
    }
}

pub(crate) fn run_solve_summary(
    input: PathBuf,
    output: OutputModeArg,
    render: SolveRenderOptions,
    include_distribution: bool,
    top: usize,
    filters: SolveFilters,
) -> ToolingResult<String> {
    let trace = read_trace(&input)?;
    let tree = filter_goal_tree(&GoalTree::from_trace(&trace)?, &filters);
    let unsupported = filtered_unsupported_count(&trace, &tree);
    let analysis = build_solve_analysis(&tree, unsupported, top);
    match (output, include_distribution) {
        (OutputModeArg::Text, false) => Ok(render_summary_text(&analysis.summary, render)),
        (OutputModeArg::Text, true) => Ok(render_analysis_text(&analysis, render)),
        (OutputModeArg::Json, false) => Ok(serde_json::to_string_pretty(&analysis.summary)?),
        (OutputModeArg::Json, true) => Ok(serde_json::to_string_pretty(&analysis)?),
    }
}

pub(crate) fn run_solve_query(
    query_kind: QueryKindArg,
    owner: &str,
    owner_match: OwnerMatchArg,
    package: Option<String>,
    manifest_path: Option<PathBuf>,
    toolchain: &str,
    view: SolveViewArg,
    output: OutputModeArg,
    render: SolveRenderOptions,
    include_distribution: bool,
    top: usize,
    rebuild_driver: bool,
    filters: SolveFilters,
) -> ToolingResult<String> {
    let trace = collect_trace(
        "check",
        package,
        manifest_path,
        toolchain,
        &[HookArg::TraitSolve],
        None,
        rebuild_driver,
        Some(owner),
        Some(query_kind),
        Some(owner_match),
    )?;
    let tree = filter_goal_tree(&GoalTree::from_trace(&trace)?, &filters);
    let unsupported = filtered_unsupported_count(&trace, &tree);
    let analysis = build_solve_analysis(&tree, unsupported, top);
    match (view, output) {
        (SolveViewArg::Tree, OutputModeArg::Text) => Ok(render_solve_tree_text(&tree, render)),
        (SolveViewArg::Tree, OutputModeArg::Json) => Ok(serde_json::to_string_pretty(&tree)?),
        (SolveViewArg::Summary, OutputModeArg::Text) if include_distribution => {
            Ok(render_analysis_text(&analysis, render))
        },
        (SolveViewArg::Summary, OutputModeArg::Text) => {
            Ok(render_summary_text(&analysis.summary, render))
        },
        (SolveViewArg::Summary, OutputModeArg::Json) if include_distribution => {
            Ok(serde_json::to_string_pretty(&analysis)?)
        },
        (SolveViewArg::Summary, OutputModeArg::Json) => {
            Ok(serde_json::to_string_pretty(&analysis.summary)?)
        },
    }
}

pub(crate) fn run_solve_diff(
    left: PathBuf,
    right: PathBuf,
    output: OutputModeArg,
    render: SolveRenderOptions,
    view: SolveDiffViewArg,
    top: usize,
) -> ToolingResult<String> {
    let left = read_analysis_or_trace(&left, top)?;
    let right = read_analysis_or_trace(&right, top)?;
    let diff = diff_analysis(&left, &right);
    match output {
        OutputModeArg::Text => Ok(render_diff_text(&diff, view, render)),
        OutputModeArg::Json => Ok(serde_json::to_string_pretty(&diff)?),
    }
}

fn read_analysis_or_trace(path: impl AsRef<std::path::Path>, top: usize) -> ToolingResult<SolveAnalysis> {
    let input = std::fs::read_to_string(path)?;
    if let Ok(analysis) = serde_json::from_str::<SolveAnalysis>(&input) {
        return Ok(analysis);
    }
    if let Ok(summary) = serde_json::from_str::<SolveSummary>(&input) {
        return Ok(SolveAnalysis {
            summary,
            predicate_distribution: Vec::new(),
            candidate_kind_distribution: Vec::new(),
            predicate_family_distribution: Vec::new(),
            candidate_family_distribution: Vec::new(),
            top_roots: Vec::new(),
        });
    }
    if let Some(summary) = parse_summary_text(&input) {
        return Ok(SolveAnalysis {
            summary,
            predicate_distribution: Vec::new(),
            candidate_kind_distribution: Vec::new(),
            predicate_family_distribution: Vec::new(),
            candidate_family_distribution: Vec::new(),
            top_roots: Vec::new(),
        });
    }

    let trace = typelude_tooling_core::ingest::trace_from_json_lines(TraceId::new(1), &input)?;
    let tree = GoalTree::from_trace(&trace)?;
    Ok(build_solve_analysis(
        &tree,
        filtered_unsupported_count(&trace, &tree),
        top,
    ))
}

fn parse_summary_text(input: &str) -> Option<SolveSummary> {
    let mut values = std::collections::BTreeMap::<String, String>::new();
    let mut top_predicates = Vec::new();
    let mut top_candidate_kinds = Vec::new();
    let mut section = None::<&str>;
    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if line == "top_predicates:" {
            section = Some("predicates");
            continue;
        }
        if line == "top_candidate_kinds:" {
            section = Some("candidate_kinds");
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            values.insert(key.to_owned(), value.to_owned());
            section = None;
            continue;
        }
        if let Some((count, label)) = line.split_once(" :: ") {
            let entry = typelude_tooling_core::SolveSummaryEntry {
                label: label.to_owned(),
                count: count.parse().ok()?,
            };
            match section {
                Some("predicates") => top_predicates.push(entry),
                Some("candidate_kinds") => top_candidate_kinds.push(entry),
                _ => return None,
            }
        }
    }
    Some(SolveSummary {
        subjects: values.get("subjects")?.parse().ok()?,
        root_goals: values.get("root_goals")?.parse().ok()?,
        goals: values.get("goals")?.parse().ok()?,
        candidates: values.get("candidates")?.parse().ok()?,
        max_goal_depth: values.get("max_goal_depth")?.parse().ok()?,
        avg_candidates_per_goal: values.get("avg_candidates_per_goal")?.parse().ok()?,
        result_ok: values.get("result.ok")?.parse().ok()?,
        result_no_solution: values.get("result.no_solution")?.parse().ok()?,
        result_ambiguous: values.get("result.ambiguous")?.parse().ok()?,
        result_unsupported: values.get("result.unsupported")?.parse().ok()?,
        top_predicates,
        top_candidate_kinds,
    })
}

fn filtered_unsupported_count(trace: &Trace, tree: &GoalTree) -> usize {
    use typelude_tooling_core::TraceEventKind;
    let subject_ids = tree
        .subjects
        .iter()
        .map(|subject| subject.id)
        .collect::<std::collections::BTreeSet<_>>();
    trace
        .events
        .iter()
        .filter(|event| event.kind() == TraceEventKind::ErrorRaised)
        .filter(|event| {
            event
                .subject_id()
                .is_some_and(|subject_id| subject_ids.contains(&subject_id))
        })
        .count()
}
