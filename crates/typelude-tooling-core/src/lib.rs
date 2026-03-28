//! Shared IR and support types for typelude tooling.

pub mod diagnostic;
pub mod error;
pub mod graph;
pub mod ids;
pub mod metrics;
pub mod render;
pub mod span;
pub mod trace;

pub use diagnostic::{CapabilityFailure, DiagnosticKind, DiagnosticLevel, DiagnosticRecord};
pub use error::{ToolingError, ToolingResult};
pub use graph::{Graph, GraphEdge, GraphNode, GraphNodeKind};
pub use ids::{
    CandidateId, DiagId, EventId, GoalId, HookId, ItemId, NodeId, RunId, SpanId, TraceId, TypeId,
};
pub use metrics::{MetricKind, MetricRecord};
pub use render::{RenderMode, RenderedText};
pub use span::{SourceLocation, SourceOrigin, SourceSpan};
pub use trace::{Trace, TraceEvent, TraceEventKind};
