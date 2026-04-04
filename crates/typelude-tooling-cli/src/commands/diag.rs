use std::path::PathBuf;

use serde::Serialize;
use typelude_tooling_core::{ToolingResult, TraceEventKind};
use typelude_tooling_rustc::{RustcDiagnosticsCollector, RustcDiagnosticsConfig};
use typelude_tooling_semantic_api::SemanticExtension;
use typelude_tooling_typelude::TypeludeExtension;

use crate::OutputModeArg;

use super::trace_io::read_trace;

#[derive(Debug, Serialize)]
pub(crate) struct DiagOutput {
    pub diagnostics: Vec<typelude_tooling_core::DiagnosticRecord>,
    pub explanations: Vec<Option<String>>,
}

pub(crate) fn run_diag(
    input: PathBuf,
    trace_input: Option<PathBuf>,
    output: OutputModeArg,
) -> ToolingResult<String> {
    let extension = TypeludeExtension;
    let mut diagnostics = RustcDiagnosticsCollector::new(RustcDiagnosticsConfig)
        .collect_from_path(input)?
        .into_iter()
        .map(|diagnostic| extension.enrich_diagnostic(&diagnostic))
        .collect::<Vec<_>>();
    if let Some(trace_input) = trace_input {
        let trace = read_trace(&trace_input)?;
        let diagnostic_count = trace
            .events
            .iter()
            .filter(|event| event.kind() == TraceEventKind::DiagnosticEmitted)
            .count();
        if diagnostic_count > 0 {
            for diagnostic in &mut diagnostics {
                diagnostic.metadata.insert(
                    String::from("trace_diagnostics"),
                    diagnostic_count.to_string(),
                );
            }
        }
    }
    let explanations = diagnostics
        .iter()
        .map(|diagnostic| extension.explain_diagnostic(diagnostic))
        .collect::<Vec<_>>();

    match output {
        OutputModeArg::Text => Ok(render_diagnostics_text(&diagnostics, &explanations)),
        OutputModeArg::Json => Ok(serde_json::to_string_pretty(&DiagOutput {
            diagnostics,
            explanations,
        })?),
    }
}

fn render_diagnostics_text(
    diagnostics: &[typelude_tooling_core::DiagnosticRecord],
    explanations: &[Option<String>],
) -> String {
    diagnostics
        .iter()
        .zip(explanations.iter())
        .map(|(diagnostic, explanation)| {
            let code = diagnostic.code().unwrap_or("no-code");
            let mut line = format!("{code}: {}", diagnostic.message());
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
