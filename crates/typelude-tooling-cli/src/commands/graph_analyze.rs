use std::path::{Path, PathBuf};

use serde::Serialize;
use typelude_tooling_core::{
    GraphAnalysis, ToolingError, ToolingResult, TraceGraphBuilder, TypeExpr,
};

use super::trace_io::read_trace;
use crate::OutputModeArg;

#[derive(Debug, Serialize)]
pub(crate) struct DistributionOutput {
    pub goal: usize,
    pub candidate: usize,
    pub expression: usize,
    pub semantic: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct HotNode {
    pub id: u64,
    pub out_degree: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct AnalyzeOutput {
    pub node_count: usize,
    pub edge_count: usize,
    pub root_count: usize,
    pub max_depth: usize,
    pub has_cycles: bool,
    pub distribution: DistributionOutput,
    pub critical_path: Vec<u64>,
    pub hot_nodes: Vec<HotNode>,
}

pub(crate) fn run_graph(
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

pub(crate) fn run_analyze(
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
