//! Subcommand implementations and dispatch from [`crate::Cli`].

mod cargo_profile_cmd;
mod collect;
mod diag;
mod doctor_cmd;
mod graph_analyze;
mod profile_cmd;
mod solve;
mod trace_io;

pub(crate) use collect::QueryKindArg;
use typelude_tooling_core::{SolveFilters, SolveResultFilter};

use crate::{
    Cli, Commands, OutputModeArg,
    solve_view::{SolveRenderOptions, SolveResultArg},
};

pub fn dispatch(cli: Cli) -> typelude_tooling_core::ToolingResult<String> {
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
        } => collect::run_collect(
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
        } => trace_io::run_trace(input, output),
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
        } => solve::run_solve_tree(
            input,
            output,
            SolveRenderOptions {
                compact,
                show_raw_kind,
                show_full_predicate,
            },
            SolveFilters {
                result: result.map(solve_result_filter),
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
        } => solve::run_solve_summary(
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
                result: result.map(solve_result_filter),
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
        } => solve::run_solve_query(
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
                result: result.map(solve_result_filter),
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
        } => solve::run_solve_query(
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
                result: result.map(solve_result_filter),
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
        } => solve::run_solve_query(
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
                result: result.map(solve_result_filter),
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
        } => solve::run_solve_diff(
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
        } => diag::run_diag(input, trace_input, output),
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
        } => profile_cmd::run_profile(
            trace_input,
            time_passes,
            type_sizes,
            self_profile_root,
            mir_root,
            output,
        ),
        Commands::CargoProfile {
            chrome_profiler_json,
            summarize_json,
            self_profile_prefix,
            top,
            output,
        } => cargo_profile_cmd::run_cargo_profile(
            chrome_profiler_json,
            summarize_json,
            self_profile_prefix,
            top,
            output,
        ),
        Commands::Doctor {
            output,
        } => doctor_cmd::run_doctor(output),
        Commands::Graph {
            input,
            expr,
            filter,
            filter_depth,
            format,
        } => graph_analyze::run_graph(input, expr, filter.as_deref(), filter_depth, &format),
        Commands::Analyze {
            input,
            expr,
            filter,
            filter_depth,
            output,
        } => graph_analyze::run_analyze(input, expr, filter.as_deref(), filter_depth, output),
    }
}

fn solve_result_filter(value: SolveResultArg) -> SolveResultFilter {
    value.into()
}

fn run_render(value: &str, output: OutputModeArg) -> typelude_tooling_core::ToolingResult<String> {
    use typelude_tooling_core::RenderMode;
    use typelude_tooling_semantic_api::SemanticExtension;
    use typelude_tooling_typelude::TypeludeExtension;

    let mode = match output {
        OutputModeArg::Text => RenderMode::Text,
        OutputModeArg::Json => RenderMode::Json,
    };
    let rendered = TypeludeExtension.render_type(value, mode);
    match output {
        OutputModeArg::Text => Ok(rendered.text),
        OutputModeArg::Json => Ok(serde_json::to_string_pretty(&rendered)?),
    }
}
