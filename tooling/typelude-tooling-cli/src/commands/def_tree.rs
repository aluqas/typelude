use std::{
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use typelude_tooling_core::{
    DefinitionCompactMode, DefinitionGraph, DefinitionRenderOptions, DefinitionSolveAnalysis,
    DefinitionSolveLinks, GoalTree, NodeId, QueryMatchKind, QueryTargetKind, ToolingError,
    ToolingResult, Trace, TraceId, TracePayload, build_definition_solve_analysis,
    build_definition_solve_links, render_definition_solve_analysis_text, unsupported_error_count,
};

use super::collect::collect_trace;
use crate::{
    DefTreeCompactArg, DefTreeFormatArg, HookArg, OutputModeArg, OwnerMatchArg,
    process::cargo::ensure_driver,
};

pub(crate) struct DefTreeOptions {
    pub owner: String,
    pub owner_match: OwnerMatchArg,
    pub input: Option<PathBuf>,
    pub edition: String,
    pub package: Option<String>,
    pub manifest_path: Option<PathBuf>,
    pub toolchain: String,
    pub format: DefTreeFormatArg,
    pub output: OutputModeArg,
    pub max_depth: usize,
    pub scope: String,
    pub rebuild_driver: bool,
    pub compact: DefTreeCompactArg,
    pub focus: Vec<String>,
    pub node: Option<u64>,
    pub show_sources: bool,
    pub hide_sources: bool,
    pub with_solve: bool,
    pub solve_owner: Option<String>,
    pub solve_owner_match: Option<OwnerMatchArg>,
    pub analysis: bool,
}

pub(crate) fn run_def_tree(options: DefTreeOptions) -> ToolingResult<String> {
    if options.scope != "local" {
        return Err(ToolingError::Unsupported(format!(
            "def-tree currently supports --scope local only, got {}",
            options.scope
        )));
    }

    let query_match = query_match_kind(options.owner_match);
    let render_options = definition_render_options(&options);
    let trace = if let Some(input) = &options.input {
        collect_single_file_def_tree(
            input,
            &options.edition,
            &options.toolchain,
            &options.owner,
            query_match,
            options.max_depth,
            options.rebuild_driver,
        )?
    } else {
        collect_cargo_def_tree(
            options.package.clone(),
            options.manifest_path.clone(),
            &options.toolchain,
            &options.owner,
            query_match,
            options.max_depth,
            options.rebuild_driver,
        )?
    };
    let graph = graph_from_trace(&trace)?;
    let integration = if options.with_solve || options.analysis {
        Some(definition_solve_integration(&options, &graph)?)
    } else {
        None
    };
    render_definition_graph(
        &graph,
        options.format,
        options.output,
        &render_options,
        integration.as_ref(),
    )
}

fn collect_cargo_def_tree(
    package: Option<String>,
    manifest_path: Option<PathBuf>,
    toolchain: &str,
    owner: &str,
    owner_match: QueryMatchKind,
    max_depth: usize,
    rebuild_driver: bool,
) -> ToolingResult<Trace> {
    let driver = ensure_driver(toolchain, rebuild_driver)?;
    let mut command = Command::new("cargo");
    command.arg(format!("+{toolchain}")).arg("check").arg("--quiet");
    if let Some(package) = package {
        command.args(["-p", &package]);
    }
    if let Some(manifest_path) = manifest_path {
        command.arg("--manifest-path").arg(manifest_path);
    }
    command.env("RUSTC_WRAPPER", driver);
    command.env("TYPELUDE_TOOLING_DEF_TREE_OWNER", owner);
    command.env("TYPELUDE_TOOLING_DEF_TREE_MATCH", owner_match.label());
    command.env("TYPELUDE_TOOLING_DEF_TREE_MAX_DEPTH", max_depth.to_string());
    command.env("TYPELUDE_TOOLING_SUMMARY_ONLY", "0");

    command_trace(command, "cargo check")
}

fn collect_single_file_def_tree(
    input: &Path,
    edition: &str,
    toolchain: &str,
    owner: &str,
    owner_match: QueryMatchKind,
    max_depth: usize,
    rebuild_driver: bool,
) -> ToolingResult<Trace> {
    let driver = ensure_driver(toolchain, rebuild_driver)?;
    let mut output = std::env::temp_dir();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    output.push(format!("typelude-def-tree-{stamp}.rmeta"));

    let mut command = Command::new(driver);
    command
        .arg(input)
        .arg("--edition")
        .arg(edition)
        .arg("--crate-name")
        .arg("typelude_def_tree_input")
        .arg("--emit")
        .arg("metadata")
        .arg("-o")
        .arg(&output);
    command.env("TYPELUDE_TOOLING_DRIVER_MODE", "direct");
    command.env("TYPELUDE_TOOLING_DEF_TREE_OWNER", owner);
    command.env("TYPELUDE_TOOLING_DEF_TREE_MATCH", owner_match.label());
    command.env("TYPELUDE_TOOLING_DEF_TREE_MAX_DEPTH", max_depth.to_string());
    command.env("TYPELUDE_TOOLING_SUMMARY_ONLY", "0");

    let trace = command_trace(command, &format!("rustc {}", input.display()));
    let _ = std::fs::remove_file(output);
    trace
}

fn collect_solve_trace_for_def_tree(options: &DefTreeOptions) -> ToolingResult<Trace> {
    let owner = options.solve_owner.as_deref().unwrap_or(&options.owner);
    let query_match = query_match_kind(options.solve_owner_match.unwrap_or(options.owner_match));
    if let Some(input) = &options.input {
        collect_single_file_solve_trace(
            input,
            &options.edition,
            &options.toolchain,
            owner,
            query_match,
            options.max_depth,
            options.rebuild_driver,
        )
    } else {
        collect_trace(
            "check",
            options.package.clone(),
            options.manifest_path.clone(),
            &options.toolchain,
            &[HookArg::TraitSolve],
            None,
            options.rebuild_driver,
            Some(owner),
            Some(QueryTargetKind::AnyOwner),
            Some(query_match),
        )
    }
}

fn collect_single_file_solve_trace(
    input: &Path,
    edition: &str,
    toolchain: &str,
    owner: &str,
    owner_match: QueryMatchKind,
    max_depth: usize,
    rebuild_driver: bool,
) -> ToolingResult<Trace> {
    let driver = ensure_driver(toolchain, rebuild_driver)?;
    let mut output = std::env::temp_dir();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    output.push(format!("typelude-solve-tree-{stamp}.rmeta"));

    let mut command = Command::new(driver);
    command
        .arg(input)
        .arg("--edition")
        .arg(edition)
        .arg("--crate-name")
        .arg("typelude_solve_input")
        .arg("--emit")
        .arg("metadata")
        .arg("-o")
        .arg(&output);
    command.env("TYPELUDE_TOOLING_DRIVER_MODE", "direct");
    command.env("TYPELUDE_TOOLING_QUERY_OWNER", owner);
    command.env("TYPELUDE_TOOLING_QUERY_KIND", QueryTargetKind::AnyOwner.label());
    command.env("TYPELUDE_TOOLING_QUERY_MATCH", owner_match.label());
    command.env("TYPELUDE_TOOLING_HOOKS", "trait_solve");
    command.env("TYPELUDE_TOOLING_MAX_DEPTH", max_depth.to_string());
    command.env("TYPELUDE_TOOLING_SUMMARY_ONLY", "0");

    let trace = command_trace(command, &format!("rustc {}", input.display()));
    let _ = std::fs::remove_file(output);
    trace
}

fn command_trace(mut command: Command, label: &str) -> ToolingResult<Trace> {
    let result = command.output()?;
    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);
    let combined = format!("{stdout}\n{stderr}");
    let trace = typelude_tooling_core::ingest::trace_from_json_lines(TraceId::new(1), &combined)?;
    if !result.status.success() {
        return Err(ToolingError::Command(format!(
            "{label} failed with status {}\n{}",
            result.status,
            trim_command_output(&combined)
        )));
    }
    if trace.events.is_empty() {
        return Err(ToolingError::Parse(String::from(
            "no def-tree trace events were collected from rustc_private output",
        )));
    }
    Ok(trace)
}

pub(crate) fn graph_from_trace(trace: &Trace) -> ToolingResult<DefinitionGraph> {
    trace
        .events
        .iter()
        .find_map(|event| {
            if let TracePayload::DefinitionGraphEmitted(payload) = &event.payload {
                Some(payload.graph.clone())
            } else {
                None
            }
        })
        .ok_or_else(|| {
            ToolingError::Parse(String::from("trace did not contain a definition graph event"))
        })
}

pub(crate) fn render_definition_graph(
    graph: &DefinitionGraph,
    format: DefTreeFormatArg,
    output: OutputModeArg,
    options: &DefinitionRenderOptions,
    integration: Option<&DefinitionSolveIntegration>,
) -> ToolingResult<String> {
    let view = graph.view(options)?;
    if output == OutputModeArg::Json || format == DefTreeFormatArg::Json {
        if let Some(integration) = integration {
            let links = filter_links_for_graph(&integration.links, &view);
            return Ok(serde_json::to_string_pretty(&DefinitionTreeJsonOutput {
                graph: view,
                links: Some(links),
                analysis: integration.analysis.clone(),
            })?);
        }
        return Ok(serde_json::to_string_pretty(&view)?);
    }
    let rendered = match format {
        DefTreeFormatArg::Tree | DefTreeFormatArg::Json => {
            let tree = view.to_tree_text_with_options(options)?;
            if let Some(integration) =
                integration.and_then(|integration| integration.analysis.as_ref())
            {
                format!("{}\n\n{tree}", render_definition_solve_analysis_text(integration))
            } else {
                tree
            }
        },
        DefTreeFormatArg::Dot => view.to_dot(),
        DefTreeFormatArg::Mermaid => view.to_mermaid(),
    };
    Ok(rendered)
}

fn filter_links_for_graph(
    links: &DefinitionSolveLinks,
    graph: &DefinitionGraph,
) -> DefinitionSolveLinks {
    let node_ids =
        graph.nodes.iter().map(|node| node.id).collect::<std::collections::BTreeSet<_>>();
    DefinitionSolveLinks {
        links: links
            .links
            .iter()
            .filter(|link| node_ids.contains(&link.def_node_id))
            .cloned()
            .collect(),
    }
}

pub(crate) fn definition_render_options(options: &DefTreeOptions) -> DefinitionRenderOptions {
    DefinitionRenderOptions {
        compact: definition_compact_mode(options.compact),
        focus: options.focus.clone(),
        root_node: options.node.map(NodeId::new),
        show_sources: if options.hide_sources {
            false
        } else if options.show_sources {
            true
        } else {
            options.compact != DefTreeCompactArg::Aggressive
        },
    }
}

pub(crate) fn definition_compact_mode(value: DefTreeCompactArg) -> DefinitionCompactMode {
    match value {
        DefTreeCompactArg::Off => DefinitionCompactMode::Off,
        DefTreeCompactArg::Basic => DefinitionCompactMode::Basic,
        DefTreeCompactArg::Aggressive => DefinitionCompactMode::Aggressive,
    }
}

pub(crate) fn collect_definition_graph(
    owner: &str,
    owner_match: OwnerMatchArg,
    input: Option<&Path>,
    edition: &str,
    package: Option<String>,
    manifest_path: Option<PathBuf>,
    toolchain: &str,
    max_depth: usize,
    rebuild_driver: bool,
) -> ToolingResult<DefinitionGraph> {
    let query_match = query_match_kind(owner_match);
    let trace = if let Some(input) = input {
        collect_single_file_def_tree(
            input,
            edition,
            toolchain,
            owner,
            query_match,
            max_depth,
            rebuild_driver,
        )?
    } else {
        collect_cargo_def_tree(
            package,
            manifest_path,
            toolchain,
            owner,
            query_match,
            max_depth,
            rebuild_driver,
        )?
    };
    graph_from_trace(&trace)
}

fn definition_solve_integration(
    options: &DefTreeOptions,
    graph: &DefinitionGraph,
) -> ToolingResult<DefinitionSolveIntegration> {
    let solve_trace = collect_solve_trace_for_def_tree(options)?;
    let tree = GoalTree::from_trace(&solve_trace)?;
    let links = build_definition_solve_links(graph, &tree);
    let analysis = if options.analysis {
        Some(build_definition_solve_analysis(
            graph,
            &tree,
            &links,
            unsupported_error_count(&solve_trace, &tree),
            10,
        ))
    } else {
        None
    };
    Ok(DefinitionSolveIntegration {
        links,
        analysis,
    })
}

fn query_match_kind(value: OwnerMatchArg) -> QueryMatchKind {
    match value {
        OwnerMatchArg::Substring => QueryMatchKind::Substring,
        OwnerMatchArg::Suffix => QueryMatchKind::Suffix,
        OwnerMatchArg::Exact => QueryMatchKind::Exact,
        OwnerMatchArg::DefId => QueryMatchKind::DefId,
    }
}

fn trim_command_output(output: &str) -> String {
    const LIMIT: usize = 4000;
    let output = output.trim();
    if output.len() <= LIMIT {
        output.to_owned()
    } else {
        format!("{}...", &output[..LIMIT])
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DefinitionSolveIntegration {
    pub links: DefinitionSolveLinks,
    pub analysis: Option<DefinitionSolveAnalysis>,
}

#[derive(Debug, Serialize)]
struct DefinitionTreeJsonOutput {
    graph: DefinitionGraph,
    links: Option<DefinitionSolveLinks>,
    analysis: Option<DefinitionSolveAnalysis>,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use typelude_tooling_core::{
        DefinitionEdge, DefinitionEdgeKind, DefinitionGraph, DefinitionNode, DefinitionNodeKind,
        NodeId,
    };

    use super::{
        DefTreeFormatArg, OutputModeArg, definition_compact_mode, render_definition_graph,
    };
    use crate::DefTreeCompactArg;

    #[test]
    fn renders_tree_and_json() {
        let graph = DefinitionGraph {
            roots: vec![NodeId::new(1)],
            nodes: vec![DefinitionNode {
                id: NodeId::new(1),
                kind: DefinitionNodeKind::TypeAlias,
                label: String::from("type Fib<N>"),
                def_path: None,
                span_id: None,
                source: None,
                metadata: BTreeMap::new(),
            }],
            edges: vec![DefinitionEdge {
                from: NodeId::new(1),
                to: NodeId::new(1),
                kind: DefinitionEdgeKind::References,
                label: String::from("cycle"),
                metadata: BTreeMap::new(),
            }],
        };
        assert!(
            render_definition_graph(
                &graph,
                DefTreeFormatArg::Tree,
                OutputModeArg::Text,
                &typelude_tooling_core::DefinitionRenderOptions::default(),
                None,
            )
            .expect("tree render should succeed")
            .contains("definition_tree")
        );
        assert!(
            render_definition_graph(
                &graph,
                DefTreeFormatArg::Tree,
                OutputModeArg::Json,
                &typelude_tooling_core::DefinitionRenderOptions::default(),
                None,
            )
            .expect("json render should succeed")
            .contains("\"nodes\"")
        );
    }

    #[test]
    fn converts_def_tree_compact_mode() {
        assert_eq!(
            definition_compact_mode(DefTreeCompactArg::Aggressive),
            typelude_tooling_core::DefinitionCompactMode::Aggressive
        );
    }
}
