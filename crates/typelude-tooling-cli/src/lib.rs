//! CLI product layer for typelude tooling.

mod solve_view;

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use typelude_tooling_core::{
    GoalTree, HookId, RenderMode, SolveSummary, ToolingError, ToolingResult, Trace,
    TraceEventKind, TraceId,
};
use typelude_tooling_rustc::{
    MirArtifactCollector, MirArtifactConfig, RustcDiagnosticsCollector, RustcDiagnosticsConfig,
    SelfProfileCollector, SelfProfileConfig, TimePassesCollector, TypeSizesCollector,
};
use typelude_tooling_typelude::{
    GraphAnalysis, TraceGraphBuilder, TypeExpr, TypeludeDiagnosticEnricher,
    TypeludeMetricEnricher, TypeludeRenderer,
};

use crate::solve_view::{
    CompactModeArg, SolveAnalysis, SolveDiffViewArg, SolveFilters, SolveRenderOptions,
    SolveResultArg, SolveViewArg, build_solve_analysis, diff_analysis, filter_goal_tree,
    render_analysis_text, render_diff_text, render_solve_tree_text, render_summary_text,
};

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
enum QueryKindArg {
    Owner,
    Impl,
    AssocItem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OwnerMatchArg {
    Substring,
    Suffix,
    Exact,
    DefId,
}

impl HookArg {
    fn as_env(self) -> &'static str {
        match self {
            Self::TraitSolve => "trait_solve",
            Self::Diagnostics => "diagnostics",
            Self::ItemStructure => "item_structure",
        }
    }

    fn hook_id(self) -> HookId {
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
        output: PathBuf,
        #[arg(long, default_value = "check")]
        cargo_subcommand: String,
        #[arg(long)]
        package: Option<String>,
        #[arg(long)]
        manifest_path: Option<PathBuf>,
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
        input: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
    },
    SolveTree {
        #[arg(long)]
        input: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Json)]
        output: OutputModeArg,
        #[arg(long, value_enum, default_value_t = CompactModeArg::Basic)]
        compact: CompactModeArg,
        #[arg(long, default_value_t = false)]
        show_raw_kind: bool,
        #[arg(long, default_value_t = false)]
        show_full_predicate: bool,
        #[arg(long, value_enum)]
        result: Option<SolveResultArg>,
        #[arg(long)]
        candidate_kind: Option<String>,
        #[arg(long)]
        max_depth: Option<usize>,
        #[arg(long)]
        subject: Option<String>,
    },
    SolveSummary {
        #[arg(long)]
        input: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
        #[arg(long, value_enum, default_value_t = CompactModeArg::Basic)]
        compact: CompactModeArg,
        #[arg(long, default_value_t = false)]
        show_raw_kind: bool,
        #[arg(long, default_value_t = false)]
        show_full_predicate: bool,
        #[arg(long, default_value_t = false)]
        include_distribution: bool,
        #[arg(long, default_value_t = 10)]
        top: usize,
        #[arg(long, value_enum)]
        result: Option<SolveResultArg>,
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
        manifest_path: Option<PathBuf>,
        #[arg(long, default_value = "nightly")]
        toolchain: String,
        #[arg(long, value_enum, default_value_t = SolveViewArg::Summary)]
        view: SolveViewArg,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
        #[arg(long, value_enum, default_value_t = CompactModeArg::Basic)]
        compact: CompactModeArg,
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
        result: Option<SolveResultArg>,
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
        manifest_path: Option<PathBuf>,
        #[arg(long, default_value = "nightly")]
        toolchain: String,
        #[arg(long, value_enum, default_value_t = SolveViewArg::Summary)]
        view: SolveViewArg,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
        #[arg(long, value_enum, default_value_t = CompactModeArg::Basic)]
        compact: CompactModeArg,
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
        result: Option<SolveResultArg>,
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
        manifest_path: Option<PathBuf>,
        #[arg(long, default_value = "nightly")]
        toolchain: String,
        #[arg(long, value_enum, default_value_t = SolveViewArg::Summary)]
        view: SolveViewArg,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
        #[arg(long, value_enum, default_value_t = CompactModeArg::Basic)]
        compact: CompactModeArg,
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
        result: Option<SolveResultArg>,
        #[arg(long)]
        candidate_kind: Option<String>,
        #[arg(long)]
        max_depth: Option<usize>,
        #[arg(long)]
        subject: Option<String>,
    },
    SolveDiff {
        #[arg(long)]
        left: PathBuf,
        #[arg(long)]
        right: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
        #[arg(long, value_enum, default_value_t = CompactModeArg::Basic)]
        compact: CompactModeArg,
        #[arg(long, default_value_t = false)]
        show_raw_kind: bool,
        #[arg(long, default_value_t = false)]
        show_full_predicate: bool,
        #[arg(long, value_enum, default_value_t = SolveDiffViewArg::All)]
        view: SolveDiffViewArg,
        #[arg(long, default_value_t = 10)]
        top: usize,
    },
    Diag {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        trace_input: Option<PathBuf>,
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
        trace_input: Option<PathBuf>,
        #[arg(long)]
        time_passes: Option<PathBuf>,
        #[arg(long)]
        type_sizes: Option<PathBuf>,
        #[arg(long)]
        self_profile_root: Option<PathBuf>,
        #[arg(long)]
        mir_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
    },
    Doctor {
        #[arg(long, value_enum, default_value_t = OutputModeArg::Text)]
        output: OutputModeArg,
    },
    Graph {
        #[arg(long)]
        input: Option<PathBuf>,
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
        input: Option<PathBuf>,
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
    match cli.command {
        Commands::Collect {
            output,
            cargo_subcommand,
            package,
            manifest_path,
            toolchain,
            hook,
            subject_filter,
            rebuild_driver,
        } => run_collect(
            output,
            &cargo_subcommand,
            package,
            manifest_path,
            &toolchain,
            &hook,
            subject_filter,
            rebuild_driver,
        ),
        Commands::Trace {
            input,
            output,
        } => run_trace(input, output),
        Commands::SolveTree {
            input,
            output,
            compact,
            show_raw_kind,
            show_full_predicate,
            result,
            candidate_kind,
            max_depth,
            subject,
        } => run_solve_tree(
            input,
            output,
            SolveRenderOptions {
                compact,
                show_raw_kind,
                show_full_predicate,
            },
            SolveFilters {
                result,
                candidate_kind,
                max_depth,
                subject,
            },
        ),
        Commands::SolveSummary {
            input,
            output,
            compact,
            show_raw_kind,
            show_full_predicate,
            include_distribution,
            top,
            result,
            candidate_kind,
            max_depth,
            subject,
        } => run_solve_summary(
            input,
            output,
            SolveRenderOptions {
                compact,
                show_raw_kind,
                show_full_predicate,
            },
            include_distribution,
            top,
            SolveFilters {
                result,
                candidate_kind,
                max_depth,
                subject,
            },
        ),
        Commands::SolveOwner {
            owner,
            owner_match,
            package,
            manifest_path,
            toolchain,
            view,
            output,
            compact,
            show_raw_kind,
            show_full_predicate,
            include_distribution,
            top,
            rebuild_driver,
            result,
            candidate_kind,
            max_depth,
            subject,
        } => run_solve_query(
            QueryKindArg::Owner,
            &owner,
            owner_match,
            package,
            manifest_path,
            &toolchain,
            view,
            output,
            SolveRenderOptions {
                compact,
                show_raw_kind,
                show_full_predicate,
            },
            include_distribution,
            top,
            rebuild_driver,
            SolveFilters {
                result,
                candidate_kind,
                max_depth,
                subject,
            },
        ),
        Commands::SolveImpl {
            owner,
            owner_match,
            package,
            manifest_path,
            toolchain,
            view,
            output,
            compact,
            show_raw_kind,
            show_full_predicate,
            include_distribution,
            top,
            rebuild_driver,
            result,
            candidate_kind,
            max_depth,
            subject,
        } => run_solve_query(
            QueryKindArg::Impl,
            &owner,
            owner_match,
            package,
            manifest_path,
            &toolchain,
            view,
            output,
            SolveRenderOptions {
                compact,
                show_raw_kind,
                show_full_predicate,
            },
            include_distribution,
            top,
            rebuild_driver,
            SolveFilters {
                result,
                candidate_kind,
                max_depth,
                subject,
            },
        ),
        Commands::SolveAssocItem {
            owner,
            owner_match,
            package,
            manifest_path,
            toolchain,
            view,
            output,
            compact,
            show_raw_kind,
            show_full_predicate,
            include_distribution,
            top,
            rebuild_driver,
            result,
            candidate_kind,
            max_depth,
            subject,
        } => run_solve_query(
            QueryKindArg::AssocItem,
            &owner,
            owner_match,
            package,
            manifest_path,
            &toolchain,
            view,
            output,
            SolveRenderOptions {
                compact,
                show_raw_kind,
                show_full_predicate,
            },
            include_distribution,
            top,
            rebuild_driver,
            SolveFilters {
                result,
                candidate_kind,
                max_depth,
                subject,
            },
        ),
        Commands::SolveDiff {
            left,
            right,
            output,
            compact,
            show_raw_kind,
            show_full_predicate,
            view,
            top,
        } => run_solve_diff(
            left,
            right,
            output,
            SolveRenderOptions {
                compact,
                show_raw_kind,
                show_full_predicate,
            },
            view,
            top,
        ),
        Commands::Diag {
            input,
            trace_input,
            output,
        } => run_diag(input, trace_input, output),
        Commands::Render {
            value,
            output,
        } => run_render(&value, output),
        Commands::Profile {
            trace_input,
            time_passes,
            type_sizes,
            self_profile_root,
            mir_root,
            output,
        } => {
            run_profile(trace_input, time_passes, type_sizes, self_profile_root, mir_root, output)
        },
        Commands::Doctor {
            output,
        } => run_doctor(output),
        Commands::Graph {
            input,
            expr,
            filter,
            filter_depth,
            format,
        } => run_graph(input, expr, filter.as_deref(), filter_depth, &format),
        Commands::Analyze {
            input,
            expr,
            filter,
            filter_depth,
            output,
        } => run_analyze(input, expr, filter.as_deref(), filter_depth, output),
    }
}

fn run_collect(
    output: PathBuf,
    cargo_subcommand: &str,
    package: Option<String>,
    manifest_path: Option<PathBuf>,
    toolchain: &str,
    hooks: &[HookArg],
    subject_filter: Option<String>,
    rebuild_driver: bool,
) -> ToolingResult<String> {
    let trace = collect_trace(
        cargo_subcommand,
        package,
        manifest_path,
        toolchain,
        hooks,
        subject_filter.clone(),
        rebuild_driver,
        None,
        None,
        None,
    )?;
    fs::write(&output, trace.to_json_lines()?)?;

    let hooks_text = if hooks.is_empty() {
        String::from("default")
    } else {
        hooks.iter().map(|hook| format!("{:?}", hook.hook_id())).collect::<Vec<_>>().join(", ")
    };
    Ok(format!(
        "collected {} events into {} using hooks: {} subject_filter={}",
        trace.events.len(),
        output.display(),
        hooks_text,
        subject_filter.unwrap_or_else(|| String::from("<none>"))
    ))
}

fn collect_trace(
    cargo_subcommand: &str,
    package: Option<String>,
    manifest_path: Option<PathBuf>,
    toolchain: &str,
    hooks: &[HookArg],
    subject_filter: Option<String>,
    rebuild_driver: bool,
    owner_query: Option<&str>,
    query_kind: Option<QueryKindArg>,
    query_match: Option<OwnerMatchArg>,
) -> ToolingResult<Trace> {
    let driver_path = ensure_driver(toolchain, rebuild_driver)?;
    let mut command = Command::new("cargo");
    command.arg(format!("+{toolchain}")).arg(cargo_subcommand).arg("--quiet");
    if let Some(package) = package {
        command.args(["-p", &package]);
    }
    if let Some(manifest_path) = manifest_path {
        command.arg("--manifest-path").arg(manifest_path);
    }
    command.env("RUSTC_WRAPPER", driver_path);
    command.env("TYPELUDE_TOOLING_SUMMARY_ONLY", "0");
    if let Some(subject_filter) = &subject_filter {
        command.env("TYPELUDE_TOOLING_SUBJECT_FILTER", subject_filter);
    }
    if let Some(owner_query) = owner_query {
        command.env("TYPELUDE_TOOLING_QUERY_OWNER", owner_query);
    }
    if let Some(query_kind) = query_kind {
        let value = match query_kind {
            QueryKindArg::Owner => "owner",
            QueryKindArg::Impl => "impl",
            QueryKindArg::AssocItem => "assoc_item",
        };
        command.env("TYPELUDE_TOOLING_QUERY_KIND", value);
    }
    if let Some(query_match) = query_match {
        let value = match query_match {
            OwnerMatchArg::Substring => "substring",
            OwnerMatchArg::Suffix => "suffix",
            OwnerMatchArg::Exact => "exact",
            OwnerMatchArg::DefId => "def_id",
        };
        command.env("TYPELUDE_TOOLING_QUERY_MATCH", value);
    }
    if !hooks.is_empty() {
        let enabled = hooks.iter().map(|hook| hook.as_env()).collect::<Vec<_>>().join(",");
        command.env("TYPELUDE_TOOLING_HOOKS", enabled);
    }

    let result = command.output()?;
    if !result.status.success() {
        return Err(ToolingError::Command(format!(
            "cargo {cargo_subcommand} failed with status {}",
            result.status
        )));
    }

    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);
    let combined = format!("{stdout}\n{stderr}");
    let trace = Trace::from_json_lines(TraceId::new(1), &combined)?;
    if trace.events.is_empty() {
        return Err(ToolingError::Parse(String::from(
            "no trace events were collected from rustc_private output",
        )));
    }
    Ok(trace)
}

fn run_trace(input: PathBuf, output: OutputModeArg) -> ToolingResult<String> {
    let trace = read_trace(&input)?;
    match output {
        OutputModeArg::Text => Ok(render_trace_text(&trace)),
        OutputModeArg::Json => Ok(serde_json::to_string_pretty(&trace)?),
    }
}

fn run_solve_tree(
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

fn run_solve_summary(
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

fn run_solve_query(
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

fn run_solve_diff(
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

fn run_diag(
    input: PathBuf,
    trace_input: Option<PathBuf>,
    output: OutputModeArg,
) -> ToolingResult<String> {
    let enricher = TypeludeDiagnosticEnricher::new();
    let mut diagnostics = RustcDiagnosticsCollector::new(RustcDiagnosticsConfig)
        .collect_from_path(input)?
        .into_iter()
        .map(|diagnostic| enricher.enrich(&diagnostic))
        .collect::<Vec<_>>();
    if let Some(trace_input) = trace_input {
        let trace = read_trace(&trace_input)?;
        let diagnostic_count = trace
            .events
            .iter()
            .filter(|event| event.kind == TraceEventKind::DiagnosticEmitted)
            .count();
        if diagnostic_count > 0 {
            for diagnostic in &mut diagnostics {
                diagnostic
                    .metadata
                    .insert(String::from("trace_diagnostics"), diagnostic_count.to_string());
            }
        }
    }
    let explanations = diagnostics
        .iter()
        .map(|diagnostic| enricher.explain_failure(diagnostic))
        .collect::<Vec<_>>();

    match output {
        OutputModeArg::Text => Ok(render_diagnostics_text(&diagnostics, &explanations)),
        OutputModeArg::Json => Ok(serde_json::to_string_pretty(&DiagOutput {
            diagnostics,
            explanations,
        })?),
    }
}

fn run_render(value: &str, output: OutputModeArg) -> ToolingResult<String> {
    let rendered = TypeludeRenderer::new().render_type_expression(value, output.into());
    match output {
        OutputModeArg::Text => Ok(rendered.text),
        OutputModeArg::Json => Ok(serde_json::to_string_pretty(&rendered)?),
    }
}

fn run_profile(
    trace_input: Option<PathBuf>,
    time_passes: Option<PathBuf>,
    type_sizes: Option<PathBuf>,
    self_profile_root: Option<PathBuf>,
    mir_root: Option<PathBuf>,
    output: OutputModeArg,
) -> ToolingResult<String> {
    let mut metrics = Vec::new();
    let mut artifacts = ProfileArtifacts::default();

    if let Some(trace_input) = trace_input {
        let trace = read_trace(&trace_input)?;
        metrics.extend(TypeludeMetricEnricher::new().enrich_trace(&trace));
    }
    if let Some(path) = time_passes {
        metrics.extend(TimePassesCollector::new().collect_from_str(&fs::read_to_string(path)?)?);
    }
    if let Some(path) = type_sizes {
        metrics.extend(TypeSizesCollector::new().collect_from_str(&fs::read_to_string(path)?)?);
    }
    if let Some(root) = self_profile_root {
        let report = SelfProfileCollector::new().collect(&SelfProfileConfig {
            root,
        })?;
        metrics.extend(report.metrics);
        artifacts.self_profile = report.artifacts;
    }
    if let Some(root) = mir_root {
        let report = MirArtifactCollector::new().collect(&MirArtifactConfig {
            root,
        })?;
        artifacts.mir = report.mir_files;
        artifacts.dot = report.dot_files;
    }

    match output {
        OutputModeArg::Text => Ok(render_profile_text(&metrics, &artifacts)),
        OutputModeArg::Json => Ok(serde_json::to_string_pretty(&ProfileOutput {
            metrics,
            artifacts,
        })?),
    }
}

fn run_graph(
    input: Option<PathBuf>,
    expr: Option<String>,
    filter: Option<&str>,
    filter_depth: usize,
    format: &str,
) -> ToolingResult<String> {
    let mut graph = build_graph(input.as_deref(), expr.as_deref())?;
    if let Some(filter) = filter {
        graph = TraceGraphBuilder::new().subgraph(&graph, filter, filter_depth);
    }

    match format {
        "json" => Ok(serde_json::to_string_pretty(&graph)?),
        "mermaid" => Ok(TraceGraphBuilder::new().to_mermaid(&graph)),
        _ => Ok(TraceGraphBuilder::new().to_dot(&graph)),
    }
}

fn run_analyze(
    input: Option<PathBuf>,
    expr: Option<String>,
    filter: Option<&str>,
    filter_depth: usize,
    output: OutputModeArg,
) -> ToolingResult<String> {
    let mut graph = build_graph(input.as_deref(), expr.as_deref())?;
    if let Some(filter) = filter {
        graph = TraceGraphBuilder::new().subgraph(&graph, filter, filter_depth);
    }
    let analysis = GraphAnalysis::new(&graph);
    let summary = analysis.summarize();
    let critical = analysis.critical_path();
    let hot = analysis.hot_nodes(5);

    match output {
        OutputModeArg::Text => {
            let mut lines = vec![
                format!("node_count={}", summary.node_count),
                format!("edge_count={}", summary.edge_count),
                format!("root_count={}", summary.root_count),
                format!("max_depth={}", summary.max_depth),
                format!("has_cycles={}", summary.has_cycles),
            ];
            if !critical.is_empty() {
                lines.push(format!(
                    "critical_path={}",
                    critical
                        .iter()
                        .map(|id| id.value().to_string())
                        .collect::<Vec<_>>()
                        .join(" -> ")
                ));
            }
            if !hot.is_empty() {
                lines.push(format!(
                    "hot_nodes={}",
                    hot.into_iter()
                        .map(|(id, degree)| format!("{}:{}", id.value(), degree))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            Ok(lines.join("\n"))
        },
        OutputModeArg::Json => Ok(serde_json::to_string_pretty(&AnalyzeOutput {
            node_count: summary.node_count,
            edge_count: summary.edge_count,
            root_count: summary.root_count,
            max_depth: summary.max_depth,
            has_cycles: summary.has_cycles,
            distribution: DistributionOutput {
                goal: summary.distribution.goal,
                candidate: summary.distribution.candidate,
                expression: summary.distribution.expression,
                semantic: summary.distribution.semantic,
            },
            critical_path: critical.iter().map(|id| id.value()).collect(),
            hot_nodes: hot
                .into_iter()
                .map(|(id, out_degree)| HotNode {
                    id: id.value(),
                    out_degree,
                })
                .collect(),
        })?),
    }
}

fn run_doctor(output: OutputModeArg) -> ToolingResult<String> {
    let nightly = Command::new("rustup").args(["run", "nightly", "rustc", "-V"]).output();
    let nightly_available = nightly.as_ref().is_ok_and(|result| result.status.success());
    let driver_path = driver_path();
    let driver_exists = driver_path.exists();
    let report = DoctorReport {
        nightly_available,
        driver_exists,
        required_env_vars: vec![String::from("RUSTC_WRAPPER")],
        supported_flags: vec![
            String::from("collect"),
            String::from("trace"),
            String::from("solve-tree"),
            String::from("solve-summary"),
            String::from("solve-owner"),
            String::from("solve-impl"),
            String::from("solve-assoc-item"),
            String::from("solve-diff"),
            String::from("graph"),
            String::from("analyze"),
            String::from("-Z dump-mir=all"),
            String::from("-Z time-passes"),
            String::from("-Z print-type-sizes"),
            String::from("-Z self-profile"),
        ],
    };

    match output {
        OutputModeArg::Text => Ok(format!(
            "nightly_available: {}\ndriver_exists: {}\nrequired_env_vars: {}\nsupported_flags: {}",
            report.nightly_available,
            report.driver_exists,
            report.required_env_vars.join(", "),
            report.supported_flags.join(", ")
        )),
        OutputModeArg::Json => Ok(serde_json::to_string_pretty(&report)?),
    }
}

fn build_graph(
    input: Option<&Path>,
    expr: Option<&str>,
) -> ToolingResult<typelude_tooling_core::Graph> {
    match (input, expr) {
        (Some(path), None) => {
            let trace = read_trace(path)?;
            Ok(TraceGraphBuilder::new().build(&trace))
        },
        (None, Some(expr)) => {
            let type_expr = TypeExpr::parse(expr).ok_or_else(|| {
                ToolingError::Parse(String::from("could not parse type expression"))
            })?;
            Ok(type_expr.lift().to_graph())
        },
        _ => Err(ToolingError::Parse(String::from("provide exactly one of --input or --expr"))),
    }
}

fn read_trace(path: impl AsRef<Path>) -> ToolingResult<Trace> {
    let input = fs::read_to_string(path)?;
    Trace::from_json_lines(TraceId::new(1), &input)
}

fn read_analysis_or_trace(path: impl AsRef<Path>, top: usize) -> ToolingResult<SolveAnalysis> {
    let input = fs::read_to_string(path)?;
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

    let trace = Trace::from_json_lines(TraceId::new(1), &input)?;
    let tree = GoalTree::from_trace(&trace)?;
    Ok(build_solve_analysis(&tree, filtered_unsupported_count(&trace, &tree), top))
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
    let subject_ids =
        tree.subjects.iter().map(|subject| subject.id).collect::<std::collections::BTreeSet<_>>();
    trace
        .events
        .iter()
        .filter(|event| event.kind == TraceEventKind::ErrorRaised)
        .filter(|event| {
            event.subject_id.is_some_and(|subject_id| subject_ids.contains(&subject_id))
        })
        .count()
}

fn render_trace_text(trace: &Trace) -> String {
    trace
        .events
        .iter()
        .map(|event| {
            let mut prefix = format!("{:?}", event.kind);
            if let Some(goal_id) = event.goal_id {
                prefix.push_str(&format!(" goal={}", goal_id.value()));
            }
            if let Some(candidate_id) = event.candidate_id {
                prefix.push_str(&format!(" cand={}", candidate_id.value()));
            }
            if let Some(subject_id) = event.subject_id {
                prefix.push_str(&format!(" subject={}", subject_id.value()));
            }
            if let Some(parent_subject_id) = event.parent_subject_id {
                prefix.push_str(&format!(" parent_subject={}", parent_subject_id.value()));
            }
            if let Some(parent_goal_id) = event.parent_goal_id {
                prefix.push_str(&format!(" parent={}", parent_goal_id.value()));
            }
            if let Some(detail) = &event.detail {
                format!("{prefix} :: {} :: {detail}", event.title)
            } else {
                format!("{prefix} :: {}", event.title)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_diagnostics_text(
    diagnostics: &[typelude_tooling_core::DiagnosticRecord],
    explanations: &[Option<String>],
) -> String {
    diagnostics
        .iter()
        .zip(explanations.iter())
        .map(|(diagnostic, explanation)| {
            let code = diagnostic.code.as_deref().unwrap_or("no-code");
            let mut line = format!("{code}: {}", diagnostic.message);
            if let Some(exp) = explanation {
                line.push('\n');
                for text_line in exp.lines() {
                    line.push_str("  ");
                    line.push_str(text_line);
                    line.push('\n');
                }
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_profile_text(
    metrics: &[typelude_tooling_core::MetricRecord],
    artifacts: &ProfileArtifacts,
) -> String {
    let mut lines = metrics
        .iter()
        .map(|metric| format!("{}={}", metric.name, metric.value))
        .collect::<Vec<_>>();
    lines.push(format!("mir_files={}", artifacts.mir.len()));
    lines.push(format!("dot_files={}", artifacts.dot.len()));
    lines.push(format!("self_profile_artifacts={}", artifacts.self_profile.len()));
    lines.join("\n")
}

fn ensure_driver(toolchain: &str, rebuild: bool) -> ToolingResult<PathBuf> {
    let path = driver_path();
    if path.exists() && !rebuild {
        return Ok(path);
    }

    let status = Command::new("cargo")
        .arg(format!("+{toolchain}"))
        .args(["build", "-p", "typelude-tooling-rustc-private", "--bin", "typelude-rustc-driver"])
        .status()?;
    if !status.success() {
        return Err(ToolingError::Command(String::from("failed to build typelude-rustc-driver")));
    }
    Ok(driver_path())
}

fn driver_path() -> PathBuf {
    let workspace_root = workspace_root();
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root.join("target"));
    target_dir.join("debug").join(format!("typelude-rustc-driver{}", std::env::consts::EXE_SUFFIX))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root should exist")
        .to_path_buf()
}

#[derive(Debug, Serialize)]
struct DiagOutput {
    diagnostics: Vec<typelude_tooling_core::DiagnosticRecord>,
    explanations: Vec<Option<String>>,
}

#[derive(Debug, Default, Serialize)]
struct ProfileArtifacts {
    mir: Vec<PathBuf>,
    dot: Vec<PathBuf>,
    self_profile: Vec<PathBuf>,
}

#[derive(Debug, Serialize)]
struct ProfileOutput {
    metrics: Vec<typelude_tooling_core::MetricRecord>,
    artifacts: ProfileArtifacts,
}

#[derive(Debug, Serialize)]
struct DoctorReport {
    nightly_available: bool,
    driver_exists: bool,
    required_env_vars: Vec<String>,
    supported_flags: Vec<String>,
}

#[derive(Debug, Serialize)]
struct DistributionOutput {
    goal: usize,
    candidate: usize,
    expression: usize,
    semantic: usize,
}

#[derive(Debug, Serialize)]
struct HotNode {
    id: u64,
    out_degree: usize,
}

#[derive(Debug, Serialize)]
struct AnalyzeOutput {
    node_count: usize,
    edge_count: usize,
    root_count: usize,
    max_depth: usize,
    has_cycles: bool,
    distribution: DistributionOutput,
    critical_path: Vec<u64>,
    hot_nodes: Vec<HotNode>,
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use clap::Parser;
    use typelude_tooling_core::{
        CandidateId, EventId, GoalId, SubjectId, SubjectKind, Trace, TraceEvent, TraceEventKind,
        TraceId,
    };

    use super::{Cli, OutputModeArg, run};

    fn unique_path(name: &str) -> std::path::PathBuf {
        let nanos =
            SystemTime::now().duration_since(UNIX_EPOCH).expect("time should advance").as_nanos();
        std::env::temp_dir().join(format!("{name}-{nanos}.txt"))
    }

    fn write_trace_fixture(path: &std::path::Path) {
        let mut trace = Trace::new(TraceId::new(1));
        let mut subject =
            TraceEvent::new(EventId::new(1), TraceEventKind::SubjectDiscovered, "RunWriter");
        subject.subject_id = Some(SubjectId::new(1));
        subject.subject_kind = Some(SubjectKind::Predicate);
        subject.metadata.insert(
            String::from("owner_path"),
            String::from("typelude_vm::core::writer_t::RunWriter"),
        );
        trace.push(subject);

        let mut goal =
            TraceEvent::new(EventId::new(2), TraceEventKind::GoalEntered, "<T as Eval>");
        goal.subject_id = Some(SubjectId::new(1));
        goal.subject_kind = Some(SubjectKind::Predicate);
        goal.goal_id = Some(GoalId::new(1));
        trace.push(goal);

        let mut candidate =
            TraceEvent::new(EventId::new(3), TraceEventKind::CandidateResult, "ImplCandidate");
        candidate.subject_id = Some(SubjectId::new(1));
        candidate.subject_kind = Some(SubjectKind::Predicate);
        candidate.goal_id = Some(GoalId::new(1));
        candidate.candidate_id = Some(CandidateId::new(1));
        candidate.detail = Some(String::from("Ok(())"));
        trace.push(candidate);

        let mut nested =
            TraceEvent::new(EventId::new(4), TraceEventKind::GoalEntered, "<U as Eval>");
        nested.subject_id = Some(SubjectId::new(1));
        nested.subject_kind = Some(SubjectKind::Predicate);
        nested.goal_id = Some(GoalId::new(2));
        nested.parent_goal_id = Some(GoalId::new(1));
        trace.push(nested);

        let mut nested_exit =
            TraceEvent::new(EventId::new(5), TraceEventKind::GoalExited, "<U as Eval>");
        nested_exit.subject_id = Some(SubjectId::new(1));
        nested_exit.subject_kind = Some(SubjectKind::Predicate);
        nested_exit.goal_id = Some(GoalId::new(2));
        nested_exit.parent_goal_id = Some(GoalId::new(1));
        nested_exit.detail = Some(String::from("NoSolution"));
        trace.push(nested_exit);

        let mut goal_exit =
            TraceEvent::new(EventId::new(6), TraceEventKind::GoalExited, "<T as Eval>");
        goal_exit.subject_id = Some(SubjectId::new(1));
        goal_exit.subject_kind = Some(SubjectKind::Predicate);
        goal_exit.goal_id = Some(GoalId::new(1));
        goal_exit.detail = Some(String::from("Ok(())"));
        trace.push(goal_exit);
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
        assert!(output.contains("subject #1"));
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
        assert!(output.contains("result=NoSolution"));
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
