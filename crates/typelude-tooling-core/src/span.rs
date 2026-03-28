use serde::{Deserialize, Serialize};

use crate::ids::SpanId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceOrigin {
    pub crate_name: Option<String>,
    pub module_path: Option<String>,
    pub file: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub id: SpanId,
    pub origin: SourceOrigin,
    pub start: Option<SourceLocation>,
    pub end: Option<SourceLocation>,
    pub label: Option<String>,
    pub raw: Option<String>,
}

impl SourceSpan {
    #[must_use]
    pub fn new(id: SpanId) -> Self {
        Self {
            id,
            origin: SourceOrigin {
                crate_name: None,
                module_path: None,
                file: None,
            },
            start: None,
            end: None,
            label: None,
            raw: None,
        }
    }
}
