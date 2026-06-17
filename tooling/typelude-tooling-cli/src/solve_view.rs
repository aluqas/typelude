use clap::ValueEnum;
use typelude_tooling_core::{
    GoalTree, SolveAnalysis, SolveAnalysisDiff, SolveProvenance, SolveResultFilter, SolveSummary,
};

use crate::solve_renderer::{
    compact_candidate_kind, compact_predicate, format_metric_diff, render_goal,
    render_root_section, render_subject,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum SolveResultArg {
    Ok,
    NoSolution,
    Ambiguous,
    Unsupported,
}

impl From<SolveResultArg> for SolveResultFilter {
    fn from(value: SolveResultArg) -> Self {
        match value {
            SolveResultArg::Ok => Self::Ok,
            SolveResultArg::NoSolution => Self::NoSolution,
            SolveResultArg::Ambiguous => Self::Ambiguous,
            SolveResultArg::Unsupported => Self::Unsupported,
        }
    }
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

pub fn render_solve_tree_text(
    tree: &GoalTree,
    options: SolveRenderOptions,
    context: Option<&SolveProvenance>,
) -> String {
    let mut lines = context.map_or_else(
        || vec![format!("trace_id={}", tree.trace_id.value())],
        |context| context.header_lines(Some(tree.trace_id.value())),
    );
    for subject in &tree.subjects {
        render_subject(subject, &mut lines, options);
        for root in &subject.roots {
            render_goal(root, 1, &mut lines, options);
        }
    }
    lines.join("\n")
}

pub fn render_summary_text(
    summary: &SolveSummary,
    options: SolveRenderOptions,
    context: Option<&SolveProvenance>,
) -> String {
    let mut lines = context.map_or_else(Vec::new, |context| context.header_lines(None));
    lines.extend([
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
    ]);

    if !summary.top_predicates.is_empty() {
        lines.push(String::from("top_predicates:"));
        for entry in &summary.top_predicates {
            lines.push(format!(
                "{} :: {}",
                entry.count,
                compact_predicate(&entry.label, options.compact)
            ));
        }
    }
    if !summary.top_candidate_kinds.is_empty() {
        lines.push(String::from("top_candidate_kinds:"));
        for entry in &summary.top_candidate_kinds {
            lines.push(format!(
                "{} :: {}",
                entry.count,
                compact_candidate_kind(&entry.label, options.compact)
            ));
        }
    }

    lines.join("\n")
}

pub fn render_analysis_text(
    analysis: &SolveAnalysis,
    options: SolveRenderOptions,
    context: Option<&SolveProvenance>,
) -> String {
    let mut lines = vec![render_summary_text(&analysis.summary, options, context)];

    if !analysis.predicate_distribution.is_empty() {
        lines.push(String::from("predicate_distribution:"));
        for entry in &analysis.predicate_distribution {
            lines.push(format!(
                "{} :: {}",
                entry.count,
                compact_predicate(&entry.label, options.compact)
            ));
        }
    }

    if !analysis.candidate_kind_distribution.is_empty() {
        lines.push(String::from("candidate_kind_distribution:"));
        for entry in &analysis.candidate_kind_distribution {
            lines.push(format!(
                "{} :: {}",
                entry.count,
                compact_candidate_kind(&entry.label, options.compact)
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
        render_root_section(&analysis.top_roots, &mut lines, options);
    }

    let deepest_roots = analysis.deepest_roots();
    if !deepest_roots.is_empty() {
        lines.push(String::from("deepest_roots:"));
        render_root_section(&deepest_roots, &mut lines, options);
    }

    let branchiest_roots = analysis.branchiest_roots();
    if !branchiest_roots.is_empty() {
        lines.push(String::from("branchiest_roots:"));
        render_root_section(&branchiest_roots, &mut lines, options);
    }

    lines.join("\n")
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
                    compact_predicate(&entry.label, options.compact),
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
                    compact_candidate_kind(&entry.label, options.compact),
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
                    compact_predicate(&format!("{}::{}", entry.subject_label, entry.predicate), options.compact),
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use typelude_tooling_core::{
        CandidateId, CandidateKind, GoalId, GoalResult, GoalTreeCandidate, GoalTreeGoal,
        GoalTreeSubject, LabelCount, PredicateRepr, QueryMatchKind, QueryTargetKind,
        SolveCollectionBasis, SolveFilters, SolveProvenance, SolveTreeSemantics, SolveViewKind,
        SubjectId, SubjectKind, TraceId, build_solve_analysis, diff_analysis, filter_goal_tree,
    };

    use super::*;

    fn sample_tree() -> GoalTree {
        GoalTree {
            trace_id: TraceId::new(1),
            subjects: vec![GoalTreeSubject {
                id: SubjectId::new(1),
                kind: SubjectKind::Predicate,
                label: String::from("typelude_vm::core::writer_t::RunWriter"),
                metadata: BTreeMap::from([
                    (
                        String::from("owner_path"),
                        String::from("typelude_vm::core::writer_t::RunWriter"),
                    ),
                    (String::from("predicate_index"), String::from("0")),
                ]),
                roots: vec![GoalTreeGoal {
                    id: GoalId::new(1),
                    parent_goal_id: None,
                    predicate: PredicateRepr::DebugText(String::from(
                        "Binder { value: TraitPredicate(<very::long::path::T as Eval>, polarity:Positive), bound_vars: [] }",
                    )),
                    result: GoalResult::NoSolution,
                    depth: 0,
                    semantic_tags: vec![typelude_tooling_core::SemanticTag::EvalLike],
                    candidates: vec![GoalTreeCandidate {
                        id: CandidateId::new(1),
                        kind: CandidateKind::Unknown(String::from(
                            "TraitCandidate { source: ParamEnv(ImplSource) }",
                        )),
                        result: GoalResult::Success,
                        semantic_tags: vec![
                            typelude_tooling_core::SemanticTag::HelperDispatchLike,
                        ],
                        metadata: BTreeMap::from([(
                            String::from("raw_candidate_kind"),
                            String::from(
                                "TraitCandidate { source: ParamEnv(ImplSource), detail: Some(LongerReason) }",
                            ),
                        )]),
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
        let filtered = filter_goal_tree(
            &tree,
            &SolveFilters {
                result: Some(SolveResultFilter::NoSolution),
                ..SolveFilters::default()
            },
        );
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
            typelude_tooling_core::predicate_family(
                "Binder { value: AliasRelate(<T as Foo>, Bar), bound_vars: [] }"
            ),
            "AliasRelate"
        );
        assert_eq!(
            typelude_tooling_core::candidate_family(
                "TraitCandidate { source: ParamEnv(ImplSource) }"
            ),
            "TraitCandidate::ParamEnv"
        );
    }

    #[test]
    fn raw_override_keeps_original_text() {
        let rendered = render_solve_tree_text(
            &sample_tree(),
            SolveRenderOptions {
                compact: CompactModeArg::Basic,
                show_raw_kind: true,
                show_full_predicate: true,
            },
            None,
        );
        assert!(rendered.contains("Binder { value: TraitPredicate"));
        assert!(rendered.contains("TraitCandidate { source: ParamEnv"));
    }

    #[test]
    fn render_tree_text_includes_header_and_subject_metadata() {
        let rendered = render_solve_tree_text(
            &sample_tree(),
            SolveRenderOptions::default(),
            Some(&SolveProvenance {
                view_kind: SolveViewKind::Tree,
                tree_semantics: SolveTreeSemantics::RustcProofTree,
                collection_basis: SolveCollectionBasis::TraceInput,
                trace_id: Some(1),
                input_path: Some(String::from("/tmp/sample.trace")),
                filters: SolveFilters {
                    subject: Some(String::from("RunWriter")),
                    ..SolveFilters::default()
                },
                ..SolveProvenance::default()
            }),
        );
        assert!(rendered.contains("view_kind=solve_tree"));
        assert!(
            rendered
                .contains("subject #1 [predicate] owner=typelude_vm::core::writer_t::RunWriter")
        );
        assert!(rendered.contains("predicate_index=0"));
        assert!(rendered.contains("label :: typelude_vm::core::writer_t::RunWriter"));
    }

    #[test]
    fn render_tree_text_includes_goal_and_candidate_details() {
        let rendered = render_solve_tree_text(&sample_tree(), SolveRenderOptions::default(), None);
        assert!(
            rendered
                .contains("goal #1 result=no_solution depth=0 candidates=1 semantic=eval_like")
        );
        assert!(rendered.contains("candidate #1 kind=TraitCandidate::ParamEnv"));
        assert!(rendered.contains("result=success"));
        assert!(rendered.contains("semantic=helper_dispatch_like"));
        assert!(rendered.contains("raw=TraitCandidate"));
    }

    #[test]
    fn render_analysis_text_adds_multiple_root_views() {
        let rendered = render_analysis_text(
            &build_solve_analysis(&sample_tree(), 0, 10),
            SolveRenderOptions::default(),
            Some(&SolveProvenance {
                view_kind: SolveViewKind::Analysis,
                tree_semantics: SolveTreeSemantics::RustcProofTree,
                collection_basis: SolveCollectionBasis::TraceInput,
                trace_id: Some(1),
                ..SolveProvenance::default()
            }),
        );
        assert!(rendered.contains("top_roots:"));
        assert!(rendered.contains("deepest_roots:"));
        assert!(rendered.contains("branchiest_roots:"));
    }

    #[test]
    fn provenance_header_uses_typed_query_fields() {
        let rendered = render_summary_text(
            &build_solve_analysis(&sample_tree(), 0, 10).summary,
            SolveRenderOptions::default(),
            Some(&SolveProvenance {
                view_kind: SolveViewKind::Summary,
                tree_semantics: SolveTreeSemantics::RustcProofTree,
                collection_basis: SolveCollectionBasis::OwnerExplicitPredicates,
                query_kind: Some(QueryTargetKind::ImplOwner),
                query_match: Some(QueryMatchKind::Exact),
                query_owner: Some(String::from("array::Get")),
                ..SolveProvenance::default()
            }),
        );
        assert!(rendered.contains("query_kind=impl"));
        assert!(rendered.contains("query_match=exact"));
    }
}
