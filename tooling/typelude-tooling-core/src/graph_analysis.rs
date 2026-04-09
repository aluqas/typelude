use std::collections::{BTreeMap, BTreeSet};

use crate::{Graph, GraphNodeKind, NodeId};

/// Graph analysis over an obligation or semantic expression graph.
pub struct GraphAnalysis<'a> {
    graph: &'a Graph,
}

/// Per-kind node count summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KindDistribution {
    pub goal: usize,
    pub candidate: usize,
    pub expression: usize,
    pub semantic: usize,
}

/// High-level summary of a graph.
#[derive(Debug, Clone)]
pub struct GraphSummary {
    pub node_count: usize,
    pub edge_count: usize,
    pub root_count: usize,
    pub max_depth: usize,
    pub has_cycles: bool,
    pub distribution: KindDistribution,
}

impl<'a> GraphAnalysis<'a> {
    #[must_use]
    pub const fn new(graph: &'a Graph) -> Self {
        Self {
            graph,
        }
    }

    #[must_use]
    pub fn roots(&self) -> Vec<NodeId> {
        let targets: BTreeSet<NodeId> = self.graph.edges.iter().map(|edge| edge.to).collect();
        self.graph
            .nodes
            .iter()
            .filter(|node| !targets.contains(&node.id))
            .map(|node| node.id)
            .collect()
    }

    #[must_use]
    pub fn max_depth(&self) -> usize {
        let roots = self.roots();
        if roots.is_empty() {
            return 0;
        }
        roots.iter().map(|&root| self.depth_from(root, &mut Vec::new())).max().unwrap_or(0)
    }

    #[must_use]
    pub fn critical_path(&self) -> Vec<NodeId> {
        let roots = self.roots();
        if roots.is_empty() {
            return Vec::new();
        }
        roots
            .iter()
            .map(|&root| {
                let mut path = Vec::new();
                self.longest_path_from(root, &mut path, &mut Vec::new());
                path
            })
            .max_by_key(Vec::len)
            .unwrap_or_default()
    }

    #[must_use]
    pub fn hot_nodes(&self, top_n: usize) -> Vec<(NodeId, usize)> {
        let mut counts = BTreeMap::<NodeId, usize>::new();
        for edge in &self.graph.edges {
            *counts.entry(edge.from).or_insert(0) += 1;
        }
        let mut entries: Vec<(NodeId, usize)> = counts.into_iter().collect();
        entries.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
        entries.truncate(top_n);
        entries
    }

    #[must_use]
    pub fn has_cycles(&self) -> bool {
        let mut visited = BTreeSet::new();
        for root in self.roots() {
            if self.cycle_dfs(root, &mut visited, &mut BTreeSet::new()) {
                return true;
            }
        }
        false
    }

    #[must_use]
    pub fn kind_distribution(&self) -> KindDistribution {
        let mut dist = KindDistribution {
            goal: 0,
            candidate: 0,
            expression: 0,
            semantic: 0,
        };
        for node in &self.graph.nodes {
            match node.kind {
                GraphNodeKind::Goal => dist.goal += 1,
                GraphNodeKind::Candidate => dist.candidate += 1,
                GraphNodeKind::Expression => dist.expression += 1,
                GraphNodeKind::Semantic => dist.semantic += 1,
            }
        }
        dist
    }

    #[must_use]
    pub fn summarize(&self) -> GraphSummary {
        GraphSummary {
            node_count: self.graph.nodes.len(),
            edge_count: self.graph.edges.len(),
            root_count: self.roots().len(),
            max_depth: self.max_depth(),
            has_cycles: self.has_cycles(),
            distribution: self.kind_distribution(),
        }
    }

    fn children_of(&self, id: NodeId) -> Vec<NodeId> {
        self.graph.edges.iter().filter(|edge| edge.from == id).map(|edge| edge.to).collect()
    }

    fn depth_from(&self, id: NodeId, stack: &mut Vec<NodeId>) -> usize {
        if stack.contains(&id) {
            return 0;
        }
        stack.push(id);
        let children = self.children_of(id);
        let depth = if children.is_empty() {
            0
        } else {
            children.iter().map(|&child| 1 + self.depth_from(child, stack)).max().unwrap_or(0)
        };
        stack.pop();
        depth
    }

    fn longest_path_from(&self, id: NodeId, best: &mut Vec<NodeId>, current: &mut Vec<NodeId>) {
        if current.contains(&id) {
            return;
        }
        current.push(id);
        let children = self.children_of(id);
        if children.is_empty() {
            if current.len() > best.len() {
                *best = current.clone();
            }
        } else {
            for child in children {
                self.longest_path_from(child, best, current);
            }
        }
        current.pop();
    }

    fn cycle_dfs(
        &self,
        id: NodeId,
        visited: &mut BTreeSet<NodeId>,
        in_stack: &mut BTreeSet<NodeId>,
    ) -> bool {
        if in_stack.contains(&id) {
            return true;
        }
        if visited.contains(&id) {
            return false;
        }
        visited.insert(id);
        in_stack.insert(id);
        for child in self.children_of(id) {
            if self.cycle_dfs(child, visited, in_stack) {
                return true;
            }
        }
        in_stack.remove(&id);
        false
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::GraphAnalysis;
    use crate::{
        EventId, GoalDiscovered, GoalId, HookId, PredicateRepr, SubjectDiscovered, SubjectId,
        SubjectKind, Trace, TraceEvent, TraceId, TracePayload, trace_graph::TraceGraphBuilder,
    };

    fn linear_trace() -> Trace {
        let mut trace = Trace::new(TraceId::new(1));
        trace.push(TraceEvent::new(
            EventId::new(1),
            TracePayload::SubjectDiscovered(SubjectDiscovered {
                hook_id: HookId::TraitSolve,
                subject_id: SubjectId::new(1),
                parent_subject_id: None,
                subject_kind: SubjectKind::Predicate,
                label: String::from("subject"),
                metadata: BTreeMap::new(),
            }),
        ));
        trace.push(TraceEvent::new(
            EventId::new(2),
            TracePayload::GoalDiscovered(GoalDiscovered {
                hook_id: HookId::TraitSolve,
                subject_id: SubjectId::new(1),
                goal_id: GoalId::new(1),
                parent_goal_id: None,
                predicate: PredicateRepr::DebugText(String::from("outer")),
                candidate_count: 0,
                semantic_tags: Vec::new(),
            }),
        ));
        trace.push(TraceEvent::new(
            EventId::new(3),
            TracePayload::GoalDiscovered(GoalDiscovered {
                hook_id: HookId::TraitSolve,
                subject_id: SubjectId::new(1),
                goal_id: GoalId::new(2),
                parent_goal_id: Some(GoalId::new(1)),
                predicate: PredicateRepr::DebugText(String::from("inner")),
                candidate_count: 0,
                semantic_tags: Vec::new(),
            }),
        ));
        trace
    }

    #[test]
    fn roots_returns_top_level_node() {
        let graph = TraceGraphBuilder::new().build(&linear_trace());
        let roots = GraphAnalysis::new(&graph).roots();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0], graph.nodes[0].id);
    }

    #[test]
    fn max_depth_two_levels() {
        let graph = TraceGraphBuilder::new().build(&linear_trace());
        assert_eq!(GraphAnalysis::new(&graph).max_depth(), 2);
    }

    #[test]
    fn critical_path_contains_both_nodes() {
        let graph = TraceGraphBuilder::new().build(&linear_trace());
        assert_eq!(GraphAnalysis::new(&graph).critical_path().len(), 3);
    }

    #[test]
    fn hot_nodes_returns_parent() {
        let graph = TraceGraphBuilder::new().build(&linear_trace());
        let hot = GraphAnalysis::new(&graph).hot_nodes(1);
        assert_eq!(hot.len(), 1);
        assert_eq!(hot[0].0, graph.nodes[0].id);
        assert_eq!(hot[0].1, 1);
    }

    #[test]
    fn has_cycles_false_for_dag() {
        let graph = TraceGraphBuilder::new().build(&linear_trace());
        assert!(!GraphAnalysis::new(&graph).has_cycles());
    }

    #[test]
    fn kind_distribution_counts_goal_nodes() {
        let graph = TraceGraphBuilder::new().build(&linear_trace());
        let dist = GraphAnalysis::new(&graph).kind_distribution();
        assert_eq!(dist.goal, 2);
        assert_eq!(dist.expression, 0);
    }

    #[test]
    fn summarize_matches_individual_methods() {
        let graph = TraceGraphBuilder::new().build(&linear_trace());
        let summary = GraphAnalysis::new(&graph).summarize();
        assert_eq!(summary.node_count, 3);
        assert_eq!(summary.edge_count, 2);
        assert_eq!(summary.root_count, 1);
        assert_eq!(summary.max_depth, 2);
        assert!(!summary.has_cycles);
    }
}
