//! Wire-format ingestion for [`crate::Trace`].
//!
//! Central entry for parsing driver/tooling JSONL so schema versioning and
//! migrations can live in one place later.

use crate::{ToolingResult, Trace, TraceId};

/// Parse newline-delimited JSON trace events (as emitted by
/// `typelude-rustc-driver`).
#[must_use = "ingestion may fail on invalid input"]
pub fn trace_from_json_lines(id: TraceId, input: &str) -> ToolingResult<Trace> {
    Trace::from_json_lines(id, input)
}
