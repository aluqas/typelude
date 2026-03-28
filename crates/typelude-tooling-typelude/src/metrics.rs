use std::collections::BTreeMap;

use typelude_tooling_core::{GoalTree, MetricKind, MetricRecord, Trace};

use crate::mapper::{SemanticNode, SemanticNodeKind};

#[derive(Debug, Default)]
pub struct TypeludeMetricEnricher;

impl TypeludeMetricEnricher {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn enrich_trace(&self, trace: &Trace) -> Vec<MetricRecord> {
        let tree = GoalTree::from_trace(trace).unwrap_or(GoalTree {
            trace_id: trace.id,
            subjects: Vec::new(),
        });
        let mut goal_count = 0_u64;
        let mut candidate_count = 0_u64;
        let mut max_depth = 0_u64;
        let mut predicate_counts = BTreeMap::<String, usize>::new();

        for subject in &tree.subjects {
            for root in &subject.roots {
                collect_goal_metrics(
                    root,
                    &mut goal_count,
                    &mut candidate_count,
                    &mut max_depth,
                    &mut predicate_counts,
                );
            }
        }

        let re_eval_count = predicate_counts
            .values()
            .filter(|count| **count > 1)
            .map(|count| count - 1)
            .sum::<usize>() as f64;

        vec![
            MetricRecord::new(MetricKind::StepCount, "semantic_step_count", trace.events.len() as f64),
            MetricRecord::new(MetricKind::ObligationCount, "obligation_count", goal_count as f64),
            MetricRecord::new(
                MetricKind::CandidateCount,
                "candidate_count",
                candidate_count as f64,
            ),
            MetricRecord::new(
                MetricKind::RecursionDepth,
                "recursion_depth_max",
                max_depth as f64,
            ),
            MetricRecord::new(
                MetricKind::ReEvaluationCount,
                "re_evaluation_count",
                re_eval_count,
            ),
        ]
    }

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

fn collect_goal_metrics(
    goal: &typelude_tooling_core::GoalTreeGoal,
    goal_count: &mut u64,
    candidate_count: &mut u64,
    max_depth: &mut u64,
    predicate_counts: &mut BTreeMap<String, usize>,
) {
    *goal_count += 1;
    *candidate_count += goal.candidates.len() as u64;
    *max_depth = (*max_depth).max(goal.depth as u64);
    *predicate_counts.entry(goal.predicate.debug_text()).or_default() += 1;
    for child in &goal.children {
        collect_goal_metrics(child, goal_count, candidate_count, max_depth, predicate_counts);
    }
}
