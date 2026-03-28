use std::{collections::BTreeMap, fs, path::Path};

use serde_json::Value;
use typelude_tooling_core::{
    CapabilityFailure, DiagnosticData, DiagnosticKind, DiagnosticLevel, DiagnosticRecord,
    RawCompilerDiagnostic, SourceLocation, SourceOrigin, SourceSpan, SpanId, ToolingResult,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RustcDiagnosticsConfig;

#[derive(Debug, Default)]
pub struct RustcDiagnosticsCollector {
    _config: RustcDiagnosticsConfig,
}

impl RustcDiagnosticsCollector {
    #[must_use]
    pub fn new(config: RustcDiagnosticsConfig) -> Self {
        Self {
            _config: config,
        }
    }

    pub fn collect_from_path(
        &self,
        path: impl AsRef<Path>,
    ) -> ToolingResult<Vec<DiagnosticRecord>> {
        let text = fs::read_to_string(path)?;
        self.collect_from_str(&text)
    }

    pub fn collect_from_str(&self, input: &str) -> ToolingResult<Vec<DiagnosticRecord>> {
        let mut diagnostics = Vec::new();
        for line in input.lines().filter(|line| !line.trim().is_empty()) {
            let value: Value = serde_json::from_str(line)?;
            if value
                .get("$message_type")
                .and_then(Value::as_str)
                .is_some_and(|kind| kind == "diagnostic")
            {
                diagnostics.push(parse_diagnostic(&value));
            }
        }
        Ok(diagnostics)
    }
}

fn parse_diagnostic(value: &Value) -> DiagnosticRecord {
    let code = value
        .get("code")
        .and_then(|inner| inner.get("code"))
        .and_then(Value::as_str)
        .map(String::from);
    let message =
        value.get("message").and_then(Value::as_str).map_or_else(String::new, String::from);
    let level = parse_level(value.get("level").and_then(Value::as_str));
    let spans = value
        .get("spans")
        .and_then(Value::as_array)
        .map(|spans| {
            spans
                .iter()
                .enumerate()
                .map(|(index, span)| parse_span(span, index as u64 + 1))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let primary_span = spans.first().cloned();

    let mut notes = Vec::new();
    let mut helps = Vec::new();
    if let Some(children) = value.get("children").and_then(Value::as_array) {
        for child in children {
            match child.get("level").and_then(Value::as_str) {
                Some("help") => helps.push(extract_message(child)),
                Some("note") => notes.push(extract_message(child)),
                _ => {},
            }
        }
    }

    let kind = if code.as_deref() == Some("E0277") {
        DiagnosticKind::CapabilityFailure(CapabilityFailure {
            required: extract_capability(&message, &notes, &helps),
            actual: None,
            context: Some(message.clone()),
        })
    } else {
        DiagnosticKind::CompilerDiagnostic
    };

    let mut metadata = BTreeMap::new();
    if let Some(rendered) = value.get("rendered").and_then(Value::as_str) {
        metadata.insert(String::from("rendered"), String::from(rendered));
    }

    DiagnosticRecord {
        data: DiagnosticData {
            raw: RawCompilerDiagnostic {
                code,
                message,
                rendered: value
                    .get("rendered")
                    .and_then(Value::as_str)
                    .map(String::from),
            },
            normalized: kind,
        },
        level,
        primary_span,
        related_spans: spans.clone(),
        notes,
        helps,
        span_ids: spans.iter().map(|span| span.id).collect(),
        metadata,
    }
}

fn parse_level(level: Option<&str>) -> DiagnosticLevel {
    match level {
        Some("warning") => DiagnosticLevel::Warning,
        Some("note") => DiagnosticLevel::Note,
        Some("help") => DiagnosticLevel::Help,
        _ => DiagnosticLevel::Error,
    }
}

fn parse_span(value: &Value, id: u64) -> SourceSpan {
    let file = value.get("file_name").and_then(Value::as_str).map(String::from);
    let start_line = value.get("line_start").and_then(Value::as_u64).unwrap_or(0) as u32;
    let start_col = value.get("column_start").and_then(Value::as_u64).unwrap_or(0) as u32;
    let end_line = value.get("line_end").and_then(Value::as_u64).unwrap_or(0) as u32;
    let end_col = value.get("column_end").and_then(Value::as_u64).unwrap_or(0) as u32;

    SourceSpan {
        id: SpanId::new(id),
        origin: SourceOrigin {
            crate_name: None,
            module_path: None,
            file,
        },
        start: Some(SourceLocation {
            line: start_line,
            column: start_col,
        }),
        end: Some(SourceLocation {
            line: end_line,
            column: end_col,
        }),
        label: value.get("label").and_then(Value::as_str).map(String::from),
        raw: None,
    }
}

fn extract_message(value: &Value) -> String {
    value.get("message").and_then(Value::as_str).map_or_else(String::new, String::from)
}

fn extract_capability(message: &str, notes: &[String], helps: &[String]) -> Option<String> {
    extract_backticked_symbol(message)
        .or_else(|| notes.iter().find_map(|note| extract_backticked_symbol(note)))
        .or_else(|| helps.iter().find_map(|help| extract_backticked_symbol(help)))
}

fn extract_backticked_symbol(input: &str) -> Option<String> {
    let (_, rest) = input.split_once('`')?;
    let (symbol, _) = rest.split_once('`')?;
    Some(symbol.to_owned())
}

#[cfg(test)]
mod tests {
    use typelude_tooling_core::DiagnosticKind;

    use super::{RustcDiagnosticsCollector, RustcDiagnosticsConfig};

    const JSON_DIAG: &str = r#"{"$message_type":"diagnostic","message":"expected a typelude Boolish value","code":{"code":"E0277"},"level":"error","spans":[{"file_name":"/tmp/file.rs","line_start":15,"line_end":15,"column_start":18,"column_end":19,"label":"this type does not implement Boolish"}],"children":[{"message":"the trait `Boolish` is not implemented for `X`","level":"help"},{"message":"Boolish is required here","level":"note"}],"rendered":"rendered text"}"#;

    #[test]
    fn parses_rustc_diagnostic_json() {
        let collector = RustcDiagnosticsCollector::new(RustcDiagnosticsConfig);
        let diagnostics = collector.collect_from_str(JSON_DIAG).expect("diagnostics should parse");

        assert_eq!(diagnostics.len(), 1);
        match diagnostics[0].kind() {
            DiagnosticKind::CapabilityFailure(capability) => {
                assert_eq!(capability.required.as_deref(), Some("Boolish"));
            },
            other => panic!("unexpected kind: {other:?}"),
        }
    }
}
