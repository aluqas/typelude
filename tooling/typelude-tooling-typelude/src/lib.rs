//! Typelude-specific semantic adapters and renderers.

pub mod caps;
pub mod diagnostics;
pub mod mapper;
pub mod metrics;
pub mod naming;
pub mod render;

pub use caps::{CapabilityKind, normalize_capability_name};
pub use diagnostics::TypeludeDiagnosticEnricher;
pub use mapper::{SemanticMapper, SemanticNode, SemanticNodeKind};
pub use metrics::TypeludeMetricEnricher;
pub use naming::compress_symbol_name;
pub use render::TypeludeRenderer;
use typelude_tooling_core::{
    CandidateKind, DiagnosticRecord, GoalTreeCandidate, GoalTreeGoal, GraphNode, MetricRecord,
    PredicateRepr, RenderMode, RenderedText, SemanticTag, Trace, semantic_tags_for_candidate,
    semantic_tags_for_predicate,
};
pub use typelude_tooling_core::{
    GraphAnalysis, GraphSummary, KindDistribution, SemanticExpr, TraceGraphBuilder, TypeExpr,
};
use typelude_tooling_semantic_api::SemanticExtension;

#[derive(Debug, Default)]
pub struct TypeludeExtension;

impl SemanticExtension for TypeludeExtension {
    fn normalize_symbol(&self, symbol: &str) -> String {
        compress_symbol_name(symbol)
    }

    fn classify_predicate(&self, predicate: &PredicateRepr) -> Vec<SemanticTag> {
        semantic_tags_for_predicate(predicate)
    }

    fn classify_candidate(&self, candidate: &CandidateKind) -> Vec<SemanticTag> {
        semantic_tags_for_candidate(candidate)
    }

    fn semantic_tags_for_goal(&self, goal: &GoalTreeGoal) -> Vec<SemanticTag> {
        if goal.semantic_tags.is_empty() {
            self.classify_predicate(&goal.predicate)
        } else {
            goal.semantic_tags.clone()
        }
    }

    fn semantic_tags_for_candidate(&self, candidate: &GoalTreeCandidate) -> Vec<SemanticTag> {
        if candidate.semantic_tags.is_empty() {
            self.classify_candidate(&candidate.kind)
        } else {
            candidate.semantic_tags.clone()
        }
    }

    fn render_type(&self, input: &str, mode: RenderMode) -> RenderedText {
        TypeludeRenderer::new().render_type_expression(input, mode)
    }

    fn enrich_diagnostic(&self, diagnostic: &DiagnosticRecord) -> DiagnosticRecord {
        TypeludeDiagnosticEnricher::new().enrich(diagnostic)
    }

    fn explain_diagnostic(&self, diagnostic: &DiagnosticRecord) -> Option<String> {
        TypeludeDiagnosticEnricher::new().explain_failure(diagnostic)
    }

    fn enrich_graph_node(&self, node: &GraphNode) -> GraphNode {
        node.clone()
    }

    fn enrich_metrics(&self, trace: &Trace) -> Vec<MetricRecord> {
        TypeludeMetricEnricher::new().enrich_trace(trace)
    }
}
