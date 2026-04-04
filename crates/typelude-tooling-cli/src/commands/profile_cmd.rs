use std::path::PathBuf;

use serde::Serialize;
use typelude_tooling_core::ToolingResult;
use typelude_tooling_rustc::{
    MirArtifactCollector, MirArtifactConfig, SelfProfileCollector, SelfProfileConfig,
    TimePassesCollector, TypeSizesCollector,
};
use typelude_tooling_semantic_api::SemanticExtension;
use typelude_tooling_typelude::TypeludeExtension;

use crate::OutputModeArg;

use super::trace_io::read_trace;

#[derive(Debug, Default, Serialize)]
pub(crate) struct ProfileArtifacts {
    pub mir: Vec<PathBuf>,
    pub dot: Vec<PathBuf>,
    pub self_profile: Vec<PathBuf>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ProfileOutput {
    pub metrics: Vec<typelude_tooling_core::MetricRecord>,
    pub artifacts: ProfileArtifacts,
}

pub(crate) fn run_profile(
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
        metrics.extend(TypeludeExtension.enrich_metrics(&trace));
    }
    if let Some(path) = time_passes {
        metrics.extend(
            TimePassesCollector::new().collect_from_str(&std::fs::read_to_string(path)?)?,
        );
    }
    if let Some(path) = type_sizes {
        metrics.extend(
            TypeSizesCollector::new().collect_from_str(&std::fs::read_to_string(path)?)?,
        );
    }
    if let Some(root) = self_profile_root {
        let report = SelfProfileCollector::new().collect(&SelfProfileConfig { root })?;
        metrics.extend(report.metrics);
        artifacts.self_profile = report.artifacts;
    }
    if let Some(root) = mir_root {
        let report = MirArtifactCollector::new().collect(&MirArtifactConfig { root })?;
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
    lines.push(format!(
        "self_profile_artifacts={}",
        artifacts.self_profile.len()
    ));
    lines.join("\n")
}
