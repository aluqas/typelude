//! Shared IR and support types for typelude tooling.

pub mod analysis_config;
pub mod diagnostic;
pub mod error;
pub mod expr;
pub mod filters;
pub mod graph;
pub mod graph_analysis;
pub mod ids;
pub mod ingest;
pub mod lowering;
pub mod metrics;
pub mod query;
pub mod render;
pub mod semantic;
pub mod solve;
pub mod solve_analysis;
pub mod solve_filter;
pub mod solve_provenance;
pub mod span;
pub mod subject;
pub mod trace;
pub mod trace_graph;

pub use analysis_config::AnalysisConfig;
pub use diagnostic::{
    CapabilityFailure, DiagnosticData, DiagnosticKind, DiagnosticLevel, DiagnosticRecord,
    RawCompilerDiagnostic,
};
pub use error::{ToolingError, ToolingResult};
pub use expr::{SemanticExpr, TypeExpr};
pub use filters::{FocusFilter, SubjectFilter};
pub use graph::{Graph, GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind};
pub use graph_analysis::{GraphAnalysis, GraphSummary, KindDistribution};
pub use ids::{
    CandidateId, DiagId, EventId, GoalId, HookId, NodeId, RunId, SpanId, SubjectId, TraceId,
    TypeId,
};
pub use lowering::{
    lower_candidate_kind, lower_goal_result, lower_predicate_repr, semantic_tags_for_candidate,
    semantic_tags_for_predicate,
};
pub use metrics::{MetricKind, MetricRecord};
pub use query::{
    OwnerQuerySpec, QueryMatchKind, QueryTargetKind, def_index_matches, owner_path_matches,
};
pub use render::{RenderMode, RenderedText};
pub use semantic::SemanticTag;
pub use solve::{
    GoalTree, GoalTreeCandidate, GoalTreeGoal, GoalTreeSubject, SolveSummary, SolveSummaryEntry,
};
pub use solve_analysis::{
    LabelCount, LabelDelta, RootHotspot, RootHotspotDelta, SolveAnalysis, SolveAnalysisDiff,
    SolveSummaryDelta, build_solve_analysis, candidate_family, diff_analysis, predicate_family,
    unsupported_error_count,
};
pub use solve_filter::{SolveFilters, SolveResultFilter, filter_goal_tree};
pub use solve_provenance::{
    SolveCollectionBasis, SolveProvenance, SolveTreeSemantics, SolveViewKind,
};
pub use span::{SourceLocation, SourceOrigin, SourceSpan};
pub use subject::SubjectKind;
pub use trace::{
    CandidateDiscovered, CandidateKind, CandidateResult, CandidateTried, DiagnosticEmitted,
    ErrorRaised, GoalDiscovered, GoalEntered, GoalExited, GoalResult, InfoEvent, PredicateRepr,
    RelationDeclared, RunFinished, RunStarted, SubjectDiscovered, Trace, TraceEvent,
    TraceEventKind, TracePayload,
};
pub use trace_graph::TraceGraphBuilder;
