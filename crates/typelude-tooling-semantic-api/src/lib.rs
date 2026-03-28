use typelude_tooling_core::{
    CandidateKind, DiagnosticRecord, GoalTreeCandidate, GoalTreeGoal, GraphNode, MetricRecord,
    PredicateRepr, RenderMode, RenderedText, SemanticTag, Trace,
};

pub trait SemanticExtension {
    fn normalize_symbol(&self, symbol: &str) -> String;
    fn classify_predicate(&self, predicate: &PredicateRepr) -> Vec<SemanticTag>;
    fn classify_candidate(&self, candidate: &CandidateKind) -> Vec<SemanticTag>;

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

    fn render_type(&self, input: &str, mode: RenderMode) -> RenderedText;
    fn enrich_diagnostic(&self, diagnostic: &DiagnosticRecord) -> DiagnosticRecord;
    fn explain_diagnostic(&self, diagnostic: &DiagnosticRecord) -> Option<String>;

    fn enrich_graph_node(&self, node: &GraphNode) -> GraphNode {
        node.clone()
    }

    fn enrich_metrics(&self, trace: &Trace) -> Vec<MetricRecord>;
}
