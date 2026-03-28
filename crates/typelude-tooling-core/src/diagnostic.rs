use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{ids::SpanId, span::SourceSpan};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityFailure {
    pub required: Option<String>,
    pub actual: Option<String>,
    pub context: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticKind {
    CompilerDiagnostic,
    CapabilityFailure(CapabilityFailure),
    ToolingNotice,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticRecord {
    pub kind: DiagnosticKind,
    pub level: DiagnosticLevel,
    pub code: Option<String>,
    pub message: String,
    pub primary_span: Option<SourceSpan>,
    pub related_spans: Vec<SourceSpan>,
    pub notes: Vec<String>,
    pub helps: Vec<String>,
    pub span_ids: Vec<SpanId>,
    pub metadata: BTreeMap<String, String>,
}

impl DiagnosticRecord {
    #[must_use]
    pub fn compiler_error(message: impl Into<String>) -> Self {
        Self {
            kind: DiagnosticKind::CompilerDiagnostic,
            level: DiagnosticLevel::Error,
            code: None,
            message: message.into(),
            primary_span: None,
            related_spans: Vec::new(),
            notes: Vec::new(),
            helps: Vec::new(),
            span_ids: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }
}
