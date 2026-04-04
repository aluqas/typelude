use std::{
    fs,
    path::{Path, PathBuf},
};

use typelude_tooling_core::{ToolingResult, Trace, TraceId};

use crate::OutputModeArg;

pub(crate) fn read_trace(path: impl AsRef<Path>) -> ToolingResult<Trace> {
    let input = fs::read_to_string(path)?;
    typelude_tooling_core::ingest::trace_from_json_lines(TraceId::new(1), &input)
}

pub(crate) fn run_trace(input: PathBuf, output: OutputModeArg) -> ToolingResult<String> {
    let trace = read_trace(&input)?;
    match output {
        OutputModeArg::Text => Ok(render_trace_text(&trace)),
        OutputModeArg::Json => Ok(serde_json::to_string_pretty(&trace)?),
    }
}

pub(crate) fn render_trace_text(trace: &Trace) -> String {
    trace
        .events
        .iter()
        .map(|event| {
            let mut prefix = format!("{:?}", event.kind());
            if let Some(goal_id) = event.goal_id() {
                prefix.push_str(&format!(" goal={}", goal_id.value()));
            }
            if let Some(candidate_id) = event.candidate_id() {
                prefix.push_str(&format!(" cand={}", candidate_id.value()));
            }
            if let Some(subject_id) = event.subject_id() {
                prefix.push_str(&format!(" subject={}", subject_id.value()));
            }
            if let Some(parent_subject_id) = event.parent_subject_id() {
                prefix.push_str(&format!(" parent_subject={}", parent_subject_id.value()));
            }
            if let Some(parent_goal_id) = event.parent_goal_id() {
                prefix.push_str(&format!(" parent={}", parent_goal_id.value()));
            }
            if let Some(detail) = event.detail() {
                format!("{prefix} :: {} :: {detail}", event.title())
            } else {
                format!("{prefix} :: {}", event.title())
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
