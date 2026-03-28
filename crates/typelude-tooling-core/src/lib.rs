//! Shared IR and support types for typelude tooling.

pub mod diagnostic;
pub mod error;
pub mod graph;
pub mod ids;
pub mod metrics;
pub mod render;
pub mod semantic;
pub mod solve;
pub mod span;
pub mod subject;
pub mod trace;

pub use diagnostic::{
    CapabilityFailure, DiagnosticData, DiagnosticKind, DiagnosticLevel, DiagnosticRecord,
    RawCompilerDiagnostic,
};
pub use error::{ToolingError, ToolingResult};
pub use graph::{Graph, GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind};
pub use ids::{
    CandidateId, DiagId, EventId, GoalId, HookId, NodeId, RunId, SpanId, SubjectId, TraceId,
    TypeId,
};
pub use metrics::{MetricKind, MetricRecord};
pub use render::{RenderMode, RenderedText};
pub use semantic::SemanticTag;
pub use solve::{
    GoalTree, GoalTreeCandidate, GoalTreeGoal, GoalTreeSubject, SolveSummary, SolveSummaryEntry,
};
pub use span::{SourceLocation, SourceOrigin, SourceSpan};
pub use subject::SubjectKind;
pub use trace::{
    CandidateDiscovered, CandidateKind, CandidateResult, CandidateTried, DiagnosticEmitted,
    ErrorRaised, GoalDiscovered, GoalEntered, GoalExited, GoalResult, InfoEvent, PredicateRepr,
    RelationDeclared, RunFinished, RunStarted, SubjectDiscovered, Trace, TraceEvent,
    TraceEventKind, TracePayload,
};
