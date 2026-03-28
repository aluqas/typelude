use std::collections::BTreeMap;

use typelude_tooling_core::{MetricKind, MetricRecord, Trace, TraceEventKind};

use crate::mapper::{SemanticNode, SemanticNodeKind};

#[derive(Debug, Default)]
pub struct TypeludeMetricEnricher;

impl TypeludeMetricEnricher {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Derive global summary metrics from a [`Trace`].
    ///
    /// Returns step count, obligation count, goal nesting depth, and
    /// re-evaluation count.
    #[must_use]
    pub fn enrich_trace(&self, trace: &Trace) -> Vec<MetricRecord> {
        let step_count = trace.events.len() as f64;

        let obligation_count = trace
            .events
            .iter()
            .filter(|e| {
                matches!(
                    e.kind,
                    TraceEventKind::GoalDiscovered
                        | TraceEventKind::GoalEntered
                        | TraceEventKind::GoalExited
                )
            })
            .count() as f64;

        let recursion_depth = max_goal_nesting_depth(trace);
        let re_eval_count = repeated_titles(trace) as f64;

        vec![
            MetricRecord::new(MetricKind::StepCount, "semantic_step_count", step_count),
            MetricRecord::new(MetricKind::ObligationCount, "obligation_count", obligation_count),
            MetricRecord::new(MetricKind::RecursionDepth, "recursion_depth_max", recursion_depth),
            MetricRecord::new(MetricKind::ReEvaluationCount, "re_evaluation_count", re_eval_count),
        ]
    }

    /// Derive per-node-kind metrics from a slice of [`SemanticNode`]s.
    ///
    /// Returns one [`MetricRecord`] per distinct [`SemanticNodeKind`] with
    /// the count of nodes of that kind in the trace.
    #[must_use]
    pub fn enrich_per_node(&self, nodes: &[SemanticNode]) -> Vec<MetricRecord> {
        let mut kind_counts: BTreeMap<&'static str, u64> = BTreeMap::new();

        for node in nodes {
            let key = semantic_node_kind_name(&node.kind);
            *kind_counts.entry(key).or_default() += 1;
        }

        kind_counts
            .into_iter()
            .map(|(kind, count)| {
                MetricRecord::new(
                    MetricKind::StepCount,
                    format!("node.{kind}.count"),
                    count as f64,
                )
            })
            .collect()
    }
}

fn semantic_node_kind_name(kind: &SemanticNodeKind) -> &'static str {
    match kind {
        SemanticNodeKind::Eval => "Eval",
        SemanticNodeKind::Apply => "Apply",
        SemanticNodeKind::If => "If",
        SemanticNodeKind::While => "While",
        SemanticNodeKind::Map => "Map",
        SemanticNodeKind::Get => "Get",
        SemanticNodeKind::PrimitiveOp => "PrimitiveOp",
        SemanticNodeKind::HelperDispatch => "HelperDispatch",
        SemanticNodeKind::VmOp => "VmOp",
        SemanticNodeKind::VmStep => "VmStep",
    }
}

fn max_goal_nesting_depth(trace: &Trace) -> f64 {
    let mut max = 0_u64;
    for event in &trace.events {
        if matches!(event.kind, TraceEventKind::GoalDiscovered | TraceEventKind::GoalEntered) {
            let depth = u64::from(event.parent_goal_id.is_some());
            max = max.max(depth);
        }
    }
    max as f64
}

fn repeated_titles(trace: &Trace) -> usize {
    let mut counts = BTreeMap::<&str, usize>::new();
    for event in &trace.events {
        *counts.entry(event.title.as_str()).or_default() += 1;
    }
    counts.values().filter(|count| **count > 1).map(|count| count - 1).sum()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use typelude_tooling_core::{EventId, NodeId, Trace, TraceEvent, TraceEventKind, TraceId};

    use super::TypeludeMetricEnricher;
    use crate::mapper::{SemanticNode, SemanticNodeKind};

    fn make_event(id: u64, kind: TraceEventKind, title: &str) -> TraceEvent {
        TraceEvent::new(EventId::new(id), kind, title)
    }

    fn make_node(id: u64, kind: SemanticNodeKind) -> SemanticNode {
        SemanticNode {
            id: NodeId::new(id),
            kind,
            label: String::new(),
            source_event_id: None,
            metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn derives_semantic_metrics() {
        let mut trace = Trace::new(TraceId::new(1));
        trace.push(make_event(1, TraceEventKind::GoalEntered, "EIf"));
        let mut nested = make_event(2, TraceEventKind::GoalEntered, "EWhile");
        nested.parent_goal_id = Some(typelude_tooling_core::GoalId::new(1));
        trace.push(nested);
        trace.push(make_event(3, TraceEventKind::GoalExited, "EWhile"));
        trace.push(make_event(4, TraceEventKind::GoalExited, "EIf"));
        trace.push(make_event(5, TraceEventKind::GoalEntered, "EIf"));

        let metrics = TypeludeMetricEnricher::new().enrich_trace(&trace);
        assert_eq!(metrics[0].value, 5.0); // step_count
        assert_eq!(metrics[2].value, 1.0); // recursion_depth_max
        assert_eq!(metrics[3].value, 3.0); // re_evaluation_count
    }

    // ① per-node metrics
    #[test]
    fn derives_per_node_metrics() {
        let nodes = vec![
            make_node(1, SemanticNodeKind::If),
            make_node(2, SemanticNodeKind::If),
            make_node(3, SemanticNodeKind::While),
            make_node(4, SemanticNodeKind::Get),
        ];

        let metrics = TypeludeMetricEnricher::new().enrich_per_node(&nodes);

        let if_count = metrics.iter().find(|m| m.name == "node.If.count").unwrap();
        let while_count = metrics.iter().find(|m| m.name == "node.While.count").unwrap();
        let get_count = metrics.iter().find(|m| m.name == "node.Get.count").unwrap();

        assert_eq!(if_count.value, 2.0);
        assert_eq!(while_count.value, 1.0);
        assert_eq!(get_count.value, 1.0);
    }
}
