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
pub struct RawCompilerDiagnostic {
    pub code: Option<String>,
    pub message: String,
    pub rendered: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticData {
    pub raw: RawCompilerDiagnostic,
    pub normalized: DiagnosticKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticRecord {
    pub data: DiagnosticData,
    pub level: DiagnosticLevel,
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
            data: DiagnosticData {
                raw: RawCompilerDiagnostic {
                    code: None,
                    message: message.into(),
                    rendered: None,
                },
                normalized: DiagnosticKind::CompilerDiagnostic,
            },
            level: DiagnosticLevel::Error,
            primary_span: None,
            related_spans: Vec::new(),
            notes: Vec::new(),
            helps: Vec::new(),
            span_ids: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn tooling_notice(message: impl Into<String>) -> Self {
        Self {
            data: DiagnosticData {
                raw: RawCompilerDiagnostic {
                    code: None,
                    message: message.into(),
                    rendered: None,
                },
                normalized: DiagnosticKind::ToolingNotice,
            },
            level: DiagnosticLevel::Note,
            primary_span: None,
            related_spans: Vec::new(),
            notes: Vec::new(),
            helps: Vec::new(),
            span_ids: Vec::new(),
            metadata: BTreeMap::new(),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> &DiagnosticKind {
        &self.data.normalized
    }

    #[must_use]
    pub fn message(&self) -> String {
        self.data.raw.message.clone()
    }

    #[must_use]
    pub fn code(&self) -> Option<&str> {
        self.data.raw.code.as_deref()
    }

    #[must_use]
    pub const fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }
}
