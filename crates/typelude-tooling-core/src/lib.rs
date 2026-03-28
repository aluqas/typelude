//! Shared IR and support types for typelude tooling.

pub mod diagnostic;
pub mod error;
pub mod graph;
pub mod ids;
pub mod metrics;
pub mod render;
pub mod solve;
pub mod span;
pub mod subject;
pub mod trace;

pub use diagnostic::{CapabilityFailure, DiagnosticKind, DiagnosticLevel, DiagnosticRecord};
pub use error::{ToolingError, ToolingResult};
pub use graph::{Graph, GraphEdge, GraphNode, GraphNodeKind};
pub use ids::{
    CandidateId, DiagId, EventId, GoalId, HookId, NodeId, RunId, SpanId, SubjectId, TraceId,
    TypeId,
};
pub use metrics::{MetricKind, MetricRecord};
pub use render::{RenderMode, RenderedText};
pub use solve::{
    GoalTree, GoalTreeCandidate, GoalTreeGoal, GoalTreeSubject, SolveSummary, SolveSummaryEntry,
};
pub use span::{SourceLocation, SourceOrigin, SourceSpan};
pub use subject::SubjectKind;
pub use trace::{Trace, TraceEvent, TraceEventKind};
