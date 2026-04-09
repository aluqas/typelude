use std::path::PathBuf;

use typelude_tooling_core::{
    GoalTree, QueryMatchKind, QueryTargetKind, SolveAnalysis, SolveCollectionBasis, SolveFilters,
    SolveProvenance, SolveSummary, SolveTreeSemantics, SolveViewKind, ToolingResult, TraceId,
    build_solve_analysis, diff_analysis, filter_goal_tree, unsupported_error_count,
};

use super::{
    collect::{QueryKindArg, collect_trace},
    trace_io::read_trace,
};
use crate::{
    HookArg, OutputModeArg, OwnerMatchArg,
    solve_view::{
        SolveDiffViewArg, SolveRenderOptions, SolveViewArg, render_analysis_text,
        render_diff_text, render_solve_tree_text, render_summary_text,
    },
};

pub(crate) fn run_solve_tree(
    input: PathBuf,
    output: OutputModeArg,
    render: SolveRenderOptions,
    filters: SolveFilters,
) -> ToolingResult<String> {
    let trace = read_trace(&input)?;
    let tree = filter_goal_tree(&GoalTree::from_trace(&trace)?, &filters);
    let context = SolveProvenance {
        view_kind: SolveViewKind::Tree,
        tree_semantics: SolveTreeSemantics::RustcProofTree,
        collection_basis: SolveCollectionBasis::TraceInput,
        trace_id: Some(tree.trace_id.value()),
        input_path: Some(input.display().to_string()),
        filters: filters.clone(),
        ..SolveProvenance::default()
    };
    match output {
        OutputModeArg::Text => Ok(render_solve_tree_text(&tree, render, Some(&context))),
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
    let unsupported = unsupported_error_count(&trace, &tree);
    let analysis = build_solve_analysis(&tree, unsupported, top);
    let context = SolveProvenance {
        view_kind: if include_distribution {
            SolveViewKind::Analysis
        } else {
            SolveViewKind::Summary
        },
        tree_semantics: SolveTreeSemantics::RustcProofTree,
        collection_basis: SolveCollectionBasis::TraceInput,
        trace_id: Some(tree.trace_id.value()),
        input_path: Some(input.display().to_string()),
        filters: filters.clone(),
        ..SolveProvenance::default()
    };
    match (output, include_distribution) {
        (OutputModeArg::Text, false) => {
            Ok(render_summary_text(&analysis.summary, render, Some(&context)))
        },
        (OutputModeArg::Text, true) => Ok(render_analysis_text(&analysis, render, Some(&context))),
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
    let query_kind = query_target_kind(query_kind);
    let query_match = query_match_kind(owner_match);
    let manifest_path_display = manifest_path.as_ref().map(|path| path.display().to_string());
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
        Some(query_match),
    )?;
    let tree = filter_goal_tree(&GoalTree::from_trace(&trace)?, &filters);
    let unsupported = unsupported_error_count(&trace, &tree);
    let analysis = build_solve_analysis(&tree, unsupported, top);
    let context = SolveProvenance {
        view_kind: match view {
            SolveViewArg::Tree => SolveViewKind::Tree,
            SolveViewArg::Summary if include_distribution => SolveViewKind::Analysis,
            SolveViewArg::Summary => SolveViewKind::Summary,
        },
        tree_semantics: SolveTreeSemantics::RustcProofTree,
        collection_basis: SolveCollectionBasis::OwnerExplicitPredicates,
        trace_id: Some(tree.trace_id.value()),
        query_kind: Some(query_kind),
        query_owner: Some(owner.to_string()),
        query_match: Some(query_match),
        manifest_path: manifest_path_display,
        filters: filters.clone(),
        ..SolveProvenance::default()
    };
    match (view, output) {
        (SolveViewArg::Tree, OutputModeArg::Text) => {
            Ok(render_solve_tree_text(&tree, render, Some(&context)))
        },
        (SolveViewArg::Tree, OutputModeArg::Json) => Ok(serde_json::to_string_pretty(&tree)?),
        (SolveViewArg::Summary, OutputModeArg::Text) if include_distribution => {
            Ok(render_analysis_text(&analysis, render, Some(&context)))
        },
        (SolveViewArg::Summary, OutputModeArg::Text) => {
            Ok(render_summary_text(&analysis.summary, render, Some(&context)))
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

fn read_analysis_or_trace(
    path: impl AsRef<std::path::Path>,
    top: usize,
) -> ToolingResult<SolveAnalysis> {
    let input = std::fs::read_to_string(path)?;
    if let Ok(analysis) = serde_json::from_str::<SolveAnalysis>(&input) {
        return Ok(analysis);
    }
    if let Ok(summary) = serde_json::from_str::<SolveSummary>(&input) {
        return Ok(SolveAnalysis::from_summary(summary));
    }
    if let Some(summary) = parse_summary_text(&input) {
        return Ok(SolveAnalysis::from_summary(summary));
    }

    let trace = typelude_tooling_core::ingest::trace_from_json_lines(TraceId::new(1), &input)?;
    let tree = GoalTree::from_trace(&trace)?;
    Ok(build_solve_analysis(&tree, unsupported_error_count(&trace, &tree), top))
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
        if line.ends_with(':') {
            section = Some("skip");
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
                _ => continue,
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

fn query_target_kind(value: QueryKindArg) -> QueryTargetKind {
    match value {
        QueryKindArg::Owner => QueryTargetKind::AnyOwner,
        QueryKindArg::Impl => QueryTargetKind::ImplOwner,
        QueryKindArg::AssocItem => QueryTargetKind::AssocItemOwner,
    }
}

fn query_match_kind(value: OwnerMatchArg) -> QueryMatchKind {
    match value {
        OwnerMatchArg::Substring => QueryMatchKind::Substring,
        OwnerMatchArg::Suffix => QueryMatchKind::Suffix,
        OwnerMatchArg::Exact => QueryMatchKind::Exact,
        OwnerMatchArg::DefId => QueryMatchKind::DefId,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_summary_text;

    #[test]
    fn parse_summary_text_ignores_headers_and_analysis_sections() {
        let input = r#"
view_kind=solve_analysis
tree_semantics=rustc_proof_tree
collection_basis=trace_input
trace_id=1
subject_filter=<none>
result_filter=<none>
candidate_kind_filter=<none>
max_depth_filter=<none>

subjects=1
root_goals=1
goals=2
candidates=1
max_goal_depth=1
avg_candidates_per_goal=0.50
result.ok=1
result.no_solution=1
result.ambiguous=0
result.unsupported=0
top_predicates:
1 :: TraitPredicate(...)
top_candidate_kinds:
1 :: Impl
predicate_distribution:
1 :: TraitPredicate(...)
top_roots:
RunWriter::<T as Eval> :: goal=1 result=success goals=2 candidates=1 max_depth=1 unique_predicates=2 unique_candidate_kinds=1 dominant_predicate_family=TraitPredicate dominant_candidate_family=Impl
"#;

        let summary = parse_summary_text(input).expect("summary text should parse");
        assert_eq!(summary.subjects, 1);
        assert_eq!(summary.goals, 2);
        assert_eq!(summary.top_predicates.len(), 1);
        assert_eq!(summary.top_candidate_kinds.len(), 1);
    }
}
