//! CLI product layer for typelude tooling.

mod commands;
mod process;
mod solve_renderer;
mod solve_view;

use clap::{Parser, Subcommand, ValueEnum};
use typelude_tooling_core::{HookId, RenderMode, ToolingResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputModeArg {
    Text,
    Json,
}

impl From<OutputModeArg> for RenderMode {
    fn from(value: OutputModeArg) -> Self {
        match value {
            OutputModeArg::Text => RenderMode::Text,
            OutputModeArg::Json => RenderMode::Json,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum HookArg {
    TraitSolve,
    Diagnostics,
    ItemStructure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OwnerMatchArg {
    Substring,
    Suffix,
    Exact,
    DefId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DefTreeFormatArg {
    Tree,
    Dot,
    Mermaid,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum DefTreeCompactArg {
    Off,
    Basic,
    Aggressive,
}

impl HookArg {
    pub(crate) fn as_env(self) -> &'static str {
        match self {
            Self::TraitSolve => "trait_solve",
            Self::Diagnostics => "diagnostics",
            Self::ItemStructure => "item_structure",
        }
    }

    pub(crate) fn hook_id(self) -> HookId {
        match self {
            Self::TraitSolve => HookId::TraitSolve,
            Self::Diagnostics => HookId::Diagnostics,
            Self::ItemStructure => HookId::ItemStructure,
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "typelude-tooling-cli")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Collect {
        #[arg(long)]
        output: std::path::PathBuf,
        #[arg(long, default_value = "check")]
        cargo_subcommand: String,
        #[arg(long)]
        package: Option<String>,
        #[arg(long)]
        manifest_path: Option<std::path::PathBuf>,
        #[arg(long, default_value = "nightly")]
        toolchain: String,
        #[arg(long, value_enum)]
        hook: Vec<HookArg>,
        #[arg(long)]
        subject_filter: Option<String>,
        #[arg(long, default_value_t = true)]
        rebuild_driver: bool,
    },
    Trace {
        #[arg(long)]
        input: std::path::PathBuf,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
    },
    SolveTree {
        #[arg(long)]
        input: std::path::PathBuf,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Json)]
        output: OutputModeArg,
        #[arg(long, value_enum, default_value_t = solve_view::CompactModeArg::Basic)]
        compact: solve_view::CompactModeArg,
        #[arg(long, default_value_t = false)]
        show_raw_kind: bool,
        #[arg(long, default_value_t = false)]
        show_full_predicate: bool,
        #[arg(long, value_enum)]
        result: Option<solve_view::SolveResultArg>,
        #[arg(long)]
        candidate_kind: Option<String>,
        #[arg(long)]
        max_depth: Option<usize>,
        #[arg(long)]
        subject: Option<String>,
        #[arg(long, default_value_t = false)]
        with_def: bool,
        #[arg(long)]
        def_owner: Option<String>,
        #[arg(long, value_enum, default_value_t = OwnerMatchArg::Exact)]
        def_owner_match: OwnerMatchArg,
        #[arg(long)]
        def_input: Option<std::path::PathBuf>,
        #[arg(long, default_value = "2021")]
        def_edition: String,
        #[arg(long)]
        package: Option<String>,
        #[arg(long)]
        manifest_path: Option<std::path::PathBuf>,
        #[arg(long, default_value = "nightly")]
        toolchain: String,
        #[arg(long, default_value_t = 32)]
        def_max_depth: usize,
        #[arg(long, default_value_t = true)]
        rebuild_driver: bool,
        #[arg(long, default_value_t = false)]
        analysis: bool,
    },
    SolveSummary {
        #[arg(long)]
        input: std::path::PathBuf,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
        #[arg(long, value_enum, default_value_t = solve_view::CompactModeArg::Basic)]
        compact: solve_view::CompactModeArg,
        #[arg(long, default_value_t = false)]
        show_raw_kind: bool,
        #[arg(long, default_value_t = false)]
        show_full_predicate: bool,
        #[arg(long, default_value_t = false)]
        include_distribution: bool,
        #[arg(long, default_value_t = 10)]
        top: usize,
        #[arg(long, value_enum)]
        result: Option<solve_view::SolveResultArg>,
        #[arg(long)]
        candidate_kind: Option<String>,
        #[arg(long)]
        max_depth: Option<usize>,
        #[arg(long)]
        subject: Option<String>,
    },
    SolveOwner {
        #[arg(long)]
        owner: String,
        #[arg(long, value_enum, default_value_t = OwnerMatchArg::Exact)]
        owner_match: OwnerMatchArg,
        #[arg(long)]
        package: Option<String>,
        #[arg(long)]
        manifest_path: Option<std::path::PathBuf>,
        #[arg(long, default_value = "nightly")]
        toolchain: String,
        #[arg(long, value_enum, default_value_t = solve_view::SolveViewArg::Summary)]
        view: solve_view::SolveViewArg,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
        #[arg(long, value_enum, default_value_t = solve_view::CompactModeArg::Basic)]
        compact: solve_view::CompactModeArg,
        #[arg(long, default_value_t = false)]
        show_raw_kind: bool,
        #[arg(long, default_value_t = false)]
        show_full_predicate: bool,
        #[arg(long, default_value_t = false)]
        include_distribution: bool,
        #[arg(long, default_value_t = 10)]
        top: usize,
        #[arg(long, default_value_t = true)]
        rebuild_driver: bool,
        #[arg(long, value_enum)]
        result: Option<solve_view::SolveResultArg>,
        #[arg(long)]
        candidate_kind: Option<String>,
        #[arg(long)]
        max_depth: Option<usize>,
        #[arg(long)]
        subject: Option<String>,
    },
    SolveImpl {
        #[arg(long)]
        owner: String,
        #[arg(long, value_enum, default_value_t = OwnerMatchArg::Exact)]
        owner_match: OwnerMatchArg,
        #[arg(long)]
        package: Option<String>,
        #[arg(long)]
        manifest_path: Option<std::path::PathBuf>,
        #[arg(long, default_value = "nightly")]
        toolchain: String,
        #[arg(long, value_enum, default_value_t = solve_view::SolveViewArg::Summary)]
        view: solve_view::SolveViewArg,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
        #[arg(long, value_enum, default_value_t = solve_view::CompactModeArg::Basic)]
        compact: solve_view::CompactModeArg,
        #[arg(long, default_value_t = false)]
        show_raw_kind: bool,
        #[arg(long, default_value_t = false)]
        show_full_predicate: bool,
        #[arg(long, default_value_t = false)]
        include_distribution: bool,
        #[arg(long, default_value_t = 10)]
        top: usize,
        #[arg(long, default_value_t = true)]
        rebuild_driver: bool,
        #[arg(long, value_enum)]
        result: Option<solve_view::SolveResultArg>,
        #[arg(long)]
        candidate_kind: Option<String>,
        #[arg(long)]
        max_depth: Option<usize>,
        #[arg(long)]
        subject: Option<String>,
    },
    SolveAssocItem {
        #[arg(long)]
        owner: String,
        #[arg(long, value_enum, default_value_t = OwnerMatchArg::Exact)]
        owner_match: OwnerMatchArg,
        #[arg(long)]
        package: Option<String>,
        #[arg(long)]
        manifest_path: Option<std::path::PathBuf>,
        #[arg(long, default_value = "nightly")]
        toolchain: String,
        #[arg(long, value_enum, default_value_t = solve_view::SolveViewArg::Summary)]
        view: solve_view::SolveViewArg,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
        #[arg(long, value_enum, default_value_t = solve_view::CompactModeArg::Basic)]
        compact: solve_view::CompactModeArg,
        #[arg(long, default_value_t = false)]
        show_raw_kind: bool,
        #[arg(long, default_value_t = false)]
        show_full_predicate: bool,
        #[arg(long, default_value_t = false)]
        include_distribution: bool,
        #[arg(long, default_value_t = 10)]
        top: usize,
        #[arg(long, default_value_t = true)]
        rebuild_driver: bool,
        #[arg(long, value_enum)]
        result: Option<solve_view::SolveResultArg>,
        #[arg(long)]
        candidate_kind: Option<String>,
        #[arg(long)]
        max_depth: Option<usize>,
        #[arg(long)]
        subject: Option<String>,
    },
    DefTree {
        #[arg(long)]
        owner: String,
        #[arg(long, value_enum, default_value_t = OwnerMatchArg::Exact)]
        owner_match: OwnerMatchArg,
        #[arg(long)]
        input: Option<std::path::PathBuf>,
        #[arg(long, default_value = "2021")]
        edition: String,
        #[arg(long)]
        package: Option<String>,
        #[arg(long)]
        manifest_path: Option<std::path::PathBuf>,
        #[arg(long, default_value = "nightly")]
        toolchain: String,
        #[arg(long, value_enum, default_value_t = DefTreeFormatArg::Tree)]
        format: DefTreeFormatArg,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
        #[arg(long, default_value_t = 32)]
        max_depth: usize,
        #[arg(long, default_value = "local")]
        scope: String,
        #[arg(long, default_value_t = true)]
        rebuild_driver: bool,
        #[arg(long, value_enum, default_value_t = DefTreeCompactArg::Off)]
        compact: DefTreeCompactArg,
        #[arg(long)]
        focus: Vec<String>,
        #[arg(long)]
        node: Option<u64>,
        #[arg(long, default_value_t = false, conflicts_with = "hide_sources")]
        show_sources: bool,
        #[arg(long, default_value_t = false)]
        hide_sources: bool,
        #[arg(long, default_value_t = false)]
        with_solve: bool,
        #[arg(long)]
        solve_owner: Option<String>,
        #[arg(long, value_enum)]
        solve_owner_match: Option<OwnerMatchArg>,
        #[arg(long, default_value_t = false)]
        analysis: bool,
    },
    SolveDiff {
        #[arg(long)]
        left: std::path::PathBuf,
        #[arg(long)]
        right: std::path::PathBuf,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
        #[arg(long, value_enum, default_value_t = solve_view::CompactModeArg::Basic)]
        compact: solve_view::CompactModeArg,
        #[arg(long, default_value_t = false)]
        show_raw_kind: bool,
        #[arg(long, default_value_t = false)]
        show_full_predicate: bool,
        #[arg(long, value_enum, default_value_t = solve_view::SolveDiffViewArg::All)]
        view: solve_view::SolveDiffViewArg,
        #[arg(long, default_value_t = 10)]
        top: usize,
    },
    Diag {
        #[arg(long)]
        input: std::path::PathBuf,
        #[arg(long)]
        trace_input: Option<std::path::PathBuf>,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
    },
    Render {
        #[arg(long)]
        value: String,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
    },
    Profile {
        #[arg(long)]
        trace_input: Option<std::path::PathBuf>,
        #[arg(long)]
        time_passes: Option<std::path::PathBuf>,
        #[arg(long)]
        type_sizes: Option<std::path::PathBuf>,
        #[arg(long)]
        self_profile_root: Option<std::path::PathBuf>,
        #[arg(long)]
        mir_root: Option<std::path::PathBuf>,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
    },
    CargoProfile {
        #[arg(long)]
        chrome_profiler_json: Option<std::path::PathBuf>,
        #[arg(long)]
        summarize_json: Option<std::path::PathBuf>,
        #[arg(long)]
        self_profile_prefix: Option<std::path::PathBuf>,
        #[arg(long, default_value_t = 10)]
        top: usize,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
    },
    Doctor {
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
    },
    Graph {
        #[arg(long)]
        input: Option<std::path::PathBuf>,
        #[arg(long)]
        expr: Option<String>,
        #[arg(long)]
        filter: Option<String>,
        #[arg(long, default_value_t = 0)]
        filter_depth: usize,
        #[arg(long, default_value = "dot")]
        format: String,
    },
    Analyze {
        #[arg(long)]
        input: Option<std::path::PathBuf>,
        #[arg(long)]
        expr: Option<String>,
        #[arg(long)]
        filter: Option<String>,
        #[arg(long, default_value_t = 0)]
        filter_depth: usize,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
    },
}

pub fn run(cli: Cli) -> ToolingResult<String> {
    commands::dispatch(cli)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use clap::Parser;
    use typelude_tooling_core::{
        CandidateId, CandidateKind, CandidateResult, EventId, GoalEntered, GoalExited, GoalId,
        GoalResult, HookId, PredicateRepr, SubjectDiscovered, SubjectId, SubjectKind, Trace,
        TraceEvent, TraceId, TracePayload,
    };

    use super::{Cli, DefTreeCompactArg, DefTreeFormatArg, OutputModeArg, OwnerMatchArg, run};

    const SAMPLE_CHROME_PROFILE: &str = r#"
[
  {"name":"typeck","cat":"Query","ph":"X","ts":1,"dur":100,"tid":3,"args":{"arg0":"program_fibonacci::demo[1]::_"}},
  {"name":"evaluate_obligation","cat":"Query","ph":"X","ts":2,"dur":60,"tid":3,"args":{"arg0":"CanonicalQueryInput { value: <typelude_vm::vm::runtime::run::RunVm<typelude_vm::vm::semantics::state::VmState<typelude_col::TArr<typelude_num::peano::Succ<typelude_num::peano::Zero>>>> as Trait> }"}}
]
"#;

    const SAMPLE_SUMMARIZE_JSON: &str = r#"
{
  "query_data": [
    {
      "label": "typeck",
      "time": { "secs": 2, "nanos": 0 },
      "self_time": { "secs": 1, "nanos": 500000000 },
      "number_of_cache_misses": 5,
      "number_of_cache_hits": 10,
      "invocation_count": 3,
      "blocked_time": { "secs": 0, "nanos": 0 },
      "incremental_load_time": { "secs": 0, "nanos": 0 },
      "incremental_hashing_time": { "secs": 0, "nanos": 0 }
    }
  ],
  "artifact_sizes": [
    { "label": "linked_artifact", "value": 1234 }
  ],
  "total_time": { "secs": 3, "nanos": 0 }
}
"#;

    fn unique_path(name: &str) -> std::path::PathBuf {
        let nanos =
            SystemTime::now().duration_since(UNIX_EPOCH).expect("time should advance").as_nanos();
        std::env::temp_dir().join(format!("{name}-{nanos}.txt"))
    }

    fn write_trace_fixture(path: &std::path::Path) {
        let mut trace = Trace::new(TraceId::new(1));
        trace.push(TraceEvent::new(
            EventId::new(1),
            TracePayload::SubjectDiscovered(SubjectDiscovered {
                hook_id: HookId::TraitSolve,
                subject_id: SubjectId::new(1),
                parent_subject_id: None,
                subject_kind: SubjectKind::Predicate,
                label: String::from("RunWriter"),
                metadata: BTreeMap::from([
                    (
                        String::from("owner_path"),
                        String::from("typelude_vm::core::writer_t::RunWriter"),
                    ),
                    (String::from("predicate_index"), String::from("0")),
                ]),
            }),
        ));

        trace.push(TraceEvent::new(
            EventId::new(2),
            TracePayload::GoalEntered(GoalEntered {
                hook_id: HookId::TraitSolve,
                subject_id: SubjectId::new(1),
                goal_id: GoalId::new(1),
                parent_goal_id: None,
                predicate: PredicateRepr::DebugText(String::from("<T as Eval>")),
                semantic_tags: Vec::new(),
            }),
        ));

        trace.push(TraceEvent::new(
            EventId::new(3),
            TracePayload::CandidateResult(CandidateResult {
                hook_id: HookId::TraitSolve,
                subject_id: SubjectId::new(1),
                goal_id: GoalId::new(1),
                candidate_id: CandidateId::new(1),
                candidate_kind: CandidateKind::Impl,
                result: GoalResult::Success,
                semantic_tags: Vec::new(),
                metadata: BTreeMap::from([(
                    String::from("raw_candidate_kind"),
                    String::from("ImplCandidate { source: user_impl, nested: ParamEnv }"),
                )]),
            }),
        ));

        trace.push(TraceEvent::new(
            EventId::new(4),
            TracePayload::GoalEntered(GoalEntered {
                hook_id: HookId::TraitSolve,
                subject_id: SubjectId::new(1),
                goal_id: GoalId::new(2),
                parent_goal_id: Some(GoalId::new(1)),
                predicate: PredicateRepr::DebugText(String::from("<U as Eval>")),
                semantic_tags: Vec::new(),
            }),
        ));

        trace.push(TraceEvent::new(
            EventId::new(5),
            TracePayload::GoalExited(GoalExited {
                hook_id: HookId::TraitSolve,
                subject_id: SubjectId::new(1),
                goal_id: GoalId::new(2),
                parent_goal_id: Some(GoalId::new(1)),
                predicate: PredicateRepr::DebugText(String::from("<U as Eval>")),
                result: GoalResult::NoSolution,
                semantic_tags: Vec::new(),
            }),
        ));

        trace.push(TraceEvent::new(
            EventId::new(6),
            TracePayload::GoalExited(GoalExited {
                hook_id: HookId::TraitSolve,
                subject_id: SubjectId::new(1),
                goal_id: GoalId::new(1),
                parent_goal_id: None,
                predicate: PredicateRepr::DebugText(String::from("<T as Eval>")),
                result: GoalResult::Success,
                semantic_tags: Vec::new(),
            }),
        ));
        fs::write(path, trace.to_json_lines().expect("trace fixture should render"))
            .expect("trace fixture should be written");
    }

    #[test]
    fn renders_trace_output() {
        let input = unique_path("typelude-trace");
        write_trace_fixture(&input);
        let cli = Cli::parse_from([
            "typelude-tooling-cli",
            "trace",
            "--input",
            input.to_str().expect("path should be valid utf-8"),
            "--output",
            "text",
        ]);
        let output = run(cli).expect("trace command should run");
        assert!(output.contains("GoalEntered"));
        fs::remove_file(input).expect("trace fixture should be removed");
    }

    #[test]
    fn renders_solve_tree_output() {
        let input = unique_path("typelude-solve-tree");
        write_trace_fixture(&input);
        let cli = Cli::parse_from([
            "typelude-tooling-cli",
            "solve-tree",
            "--input",
            input.to_str().expect("path should be valid utf-8"),
            "--output",
            "text",
        ]);
        let output = run(cli).expect("solve-tree command should run");
        assert!(output.contains("view_kind=solve_tree"));
        assert!(output.contains("collection_basis=trace_input"));
        assert!(output.contains("subject #1"));
        assert!(output.contains("predicate_index=0"));
        assert!(output.contains("candidate #1"));
        fs::remove_file(input).expect("trace fixture should be removed");
    }

    #[test]
    fn renders_solve_summary_output() {
        let input = unique_path("typelude-solve-summary");
        write_trace_fixture(&input);
        let cli = Cli::parse_from([
            "typelude-tooling-cli",
            "solve-summary",
            "--input",
            input.to_str().expect("path should be valid utf-8"),
            "--output",
            "text",
        ]);
        let output = run(cli).expect("solve-summary command should run");
        assert!(output.contains("view_kind=solve_summary"));
        assert!(output.contains("subjects=1"));
        assert!(output.contains("result.no_solution=1"));
        fs::remove_file(input).expect("trace fixture should be removed");
    }

    #[test]
    fn filters_solve_tree_by_result() {
        let input = unique_path("typelude-solve-tree-filter");
        write_trace_fixture(&input);
        let cli = Cli::parse_from([
            "typelude-tooling-cli",
            "solve-tree",
            "--input",
            input.to_str().expect("path should be valid utf-8"),
            "--output",
            "text",
            "--result",
            "no-solution",
        ]);
        let output = run(cli).expect("filtered solve-tree should run");
        assert!(output.contains("<U as Eval>"));
        assert!(output.contains("result=no_solution"));
        fs::remove_file(input).expect("trace fixture should be removed");
    }

    #[test]
    fn diffs_summaries_from_trace_inputs() {
        let left = unique_path("typelude-solve-diff-left");
        let right = unique_path("typelude-solve-diff-right");
        write_trace_fixture(&left);
        write_trace_fixture(&right);
        let cli = Cli::parse_from([
            "typelude-tooling-cli",
            "solve-diff",
            "--left",
            left.to_str().expect("path should be valid utf-8"),
            "--right",
            right.to_str().expect("path should be valid utf-8"),
            "--output",
            "text",
        ]);
        let output = run(cli).expect("solve-diff should run");
        assert!(output.contains("goals: left=2 right=2 delta=+0"));
        fs::remove_file(left).expect("left trace should be removed");
        fs::remove_file(right).expect("right trace should be removed");
    }

    #[test]
    fn parses_solve_impl_and_assoc_item_commands() {
        let solve_impl =
            Cli::parse_from(["typelude-tooling-cli", "solve-impl", "--owner", "RunWriter"]);
        assert!(matches!(solve_impl.command, super::Commands::SolveImpl { .. }));

        let solve_assoc =
            Cli::parse_from(["typelude-tooling-cli", "solve-assoc-item", "--owner", "OpIf"]);
        assert!(matches!(solve_assoc.command, super::Commands::SolveAssocItem { .. }));
    }

    #[test]
    fn parses_def_tree_command() {
        let cli = Cli::parse_from([
            "typelude-tooling-cli",
            "def-tree",
            "--input",
            ".saqula/ty_fibonacci.rs",
            "--owner",
            "Fib",
            "--owner-match",
            "suffix",
            "--format",
            "mermaid",
            "--output",
            "text",
            "--max-depth",
            "16",
        ]);

        let super::Commands::DefTree {
            owner,
            owner_match,
            input,
            format,
            output,
            max_depth,
            scope,
            compact,
            focus,
            node,
            hide_sources,
            with_solve,
            solve_owner,
            analysis,
            ..
        } = cli.command
        else {
            panic!("expected def-tree command");
        };
        assert_eq!(owner, "Fib");
        assert_eq!(owner_match, OwnerMatchArg::Suffix);
        assert_eq!(
            input.expect("input should parse"),
            std::path::PathBuf::from(".saqula/ty_fibonacci.rs")
        );
        assert_eq!(format, DefTreeFormatArg::Mermaid);
        assert_eq!(output, OutputModeArg::Text);
        assert_eq!(max_depth, 16);
        assert_eq!(scope, "local");
        assert_eq!(compact, DefTreeCompactArg::Off);
        assert!(focus.is_empty());
        assert_eq!(node, None);
        assert!(!hide_sources);
        assert!(!with_solve);
        assert_eq!(solve_owner, None);
        assert!(!analysis);
    }

    #[test]
    fn parses_def_tree_view_and_analysis_options() {
        let cli = Cli::parse_from([
            "typelude-tooling-cli",
            "def-tree",
            "--input",
            ".saqula/ty_fibonacci.rs",
            "--owner",
            "Fib",
            "--compact",
            "basic",
            "--focus",
            "WhileHelper",
            "--focus",
            "FibStep",
            "--node",
            "7",
            "--hide-sources",
            "--with-solve",
            "--solve-owner",
            "Fib",
            "--solve-owner-match",
            "suffix",
            "--analysis",
        ]);

        let super::Commands::DefTree {
            compact,
            focus,
            node,
            hide_sources,
            with_solve,
            solve_owner,
            solve_owner_match,
            analysis,
            ..
        } = cli.command
        else {
            panic!("expected def-tree command");
        };
        assert_eq!(compact, DefTreeCompactArg::Basic);
        assert_eq!(focus, vec![String::from("WhileHelper"), String::from("FibStep")]);
        assert_eq!(node, Some(7));
        assert!(hide_sources);
        assert!(with_solve);
        assert_eq!(solve_owner, Some(String::from("Fib")));
        assert_eq!(solve_owner_match, Some(OwnerMatchArg::Suffix));
        assert!(analysis);
    }

    #[test]
    fn parses_solve_tree_definition_overlay_options() {
        let cli = Cli::parse_from([
            "typelude-tooling-cli",
            "solve-tree",
            "--input",
            "/tmp/trace.ndjson",
            "--with-def",
            "--def-owner",
            "Fib",
            "--def-owner-match",
            "suffix",
            "--def-input",
            ".saqula/ty_fibonacci.rs",
            "--analysis",
        ]);

        let super::Commands::SolveTree {
            with_def,
            def_owner,
            def_owner_match,
            def_input,
            analysis,
            ..
        } = cli.command
        else {
            panic!("expected solve-tree command");
        };
        assert!(with_def);
        assert_eq!(def_owner, Some(String::from("Fib")));
        assert_eq!(def_owner_match, OwnerMatchArg::Suffix);
        assert_eq!(
            def_input.expect("def input should parse"),
            std::path::PathBuf::from(".saqula/ty_fibonacci.rs")
        );
        assert!(analysis);
    }

    #[test]
    fn solve_tree_json_keeps_raw_predicate() {
        let input = unique_path("typelude-solve-tree-json");
        write_trace_fixture(&input);
        let cli = Cli::parse_from([
            "typelude-tooling-cli",
            "solve-tree",
            "--input",
            input.to_str().expect("path should be valid utf-8"),
            "--output",
            "json",
            "--compact",
            "aggressive",
        ]);
        let output = run(cli).expect("solve-tree json should run");
        assert!(output.contains("<T as Eval>"));
        fs::remove_file(input).expect("trace fixture should be removed");
    }

    #[test]
    fn renders_diagnostic_output() {
        let input = unique_path("typelude-diag");
        fs::write(
            &input,
            "{\"$message_type\":\"diagnostic\",\"message\":\"expected a typelude Boolish value\",\"code\":{\"code\":\"E0277\"},\"level\":\"error\",\"spans\":[],\"children\":[]}\n",
        )
        .expect("diagnostic fixture should be written");
        let cli = Cli::parse_from([
            "typelude-tooling-cli",
            "diag",
            "--input",
            input.to_str().expect("path should be valid utf-8"),
            "--output",
            "text",
        ]);
        let output = run(cli).expect("diag command should run");
        assert!(output.starts_with("E0277: expected a typelude Boolish value"));
        fs::remove_file(input).expect("diagnostic fixture should be removed");
    }

    #[test]
    fn renders_cargo_profile_output() {
        let chrome_input = unique_path("typelude-cargo-profile-chrome");
        let summarize_input = unique_path("typelude-cargo-profile-summary");
        fs::write(&chrome_input, SAMPLE_CHROME_PROFILE)
            .expect("chrome profile fixture should be written");
        fs::write(&summarize_input, SAMPLE_SUMMARIZE_JSON)
            .expect("summarize fixture should be written");
        let cli = Cli::parse_from([
            "typelude-tooling-cli",
            "cargo-profile",
            "--chrome-profiler-json",
            chrome_input.to_str().expect("path should be valid utf-8"),
            "--summarize-json",
            summarize_input.to_str().expect("path should be valid utf-8"),
            "--output",
            "text",
        ]);
        let output = run(cli).expect("cargo-profile command should run");
        assert!(output.contains("self-profile self time is dominated by `typeck`"));
        assert!(output.contains("chrome.semantic_hotspots:"));
        assert!(output.contains("chrome.item_hotspots:"));
        fs::remove_file(chrome_input).expect("chrome fixture should be removed");
        fs::remove_file(summarize_input).expect("summarize fixture should be removed");
    }

    #[test]
    fn renders_type_expression() {
        let cli = Cli::parse_from([
            "typelude-tooling-cli",
            "render",
            "--value",
            "Array<U1, Array<U2, Nil>>",
            "--output",
            "text",
        ]);
        let output = run(cli).expect("render command should run");
        assert_eq!(output, "[1, 2]");
    }

    #[test]
    fn renders_profile_summary() {
        let trace_input = unique_path("typelude-profile-trace");
        write_trace_fixture(&trace_input);
        let cli = Cli::parse_from([
            "typelude-tooling-cli",
            "profile",
            "--trace-input",
            trace_input.to_str().expect("path should be valid utf-8"),
            "--output",
            "text",
        ]);
        let output = run(cli).expect("profile command should run");
        assert!(output.contains("semantic_step_count=6"));
        fs::remove_file(trace_input).expect("trace fixture should be removed");
    }

    #[test]
    fn output_mode_maps_to_render_mode() {
        let render_mode: typelude_tooling_core::RenderMode = OutputModeArg::Json.into();
        assert_eq!(render_mode, typelude_tooling_core::RenderMode::Json);
    }
}
