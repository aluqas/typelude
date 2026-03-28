use std::collections::{BTreeMap, BTreeSet, VecDeque};

use typelude_tooling_core::{
    CandidateId, GoalId, Graph, GraphEdge, GraphNode, GraphNodeKind, NodeId, SubjectId, Trace,
    TraceEvent, TraceEventKind,
};

const SUBJECT_NODE_BASE: u64 = 1_000_000_000;
const GOAL_NODE_BASE: u64 = 2_000_000_000;
const CANDIDATE_NODE_BASE: u64 = 3_000_000_000;
const EVENT_NODE_BASE: u64 = 4_000_000_000;

#[derive(Debug, Default)]
pub struct TraceGraphBuilder;

impl TraceGraphBuilder {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn build(&self, trace: &Trace) -> Graph {
        let mut graph = Graph::default();
        let mut subject_nodes = BTreeMap::<SubjectId, NodeId>::new();
        let mut goal_nodes = BTreeMap::<GoalId, NodeId>::new();
        let mut candidate_nodes = BTreeMap::<CandidateId, NodeId>::new();

        for event in &trace.events {
            match event.kind {
                TraceEventKind::SubjectDiscovered => {
                    let Some(subject_id) = event.subject_id else {
                        continue;
                    };
                    let node_id =
                        ensure_subject_node(&mut graph, &mut subject_nodes, subject_id, event);
                    if let Some(parent_subject_id) = event.parent_subject_id {
                        let parent_id = ensure_subject_placeholder(
                            &mut graph,
                            &mut subject_nodes,
                            parent_subject_id,
                        );
                        graph.edges.push(make_edge(parent_id, node_id, "subject"));
                    }
                },
                TraceEventKind::GoalDiscovered
                | TraceEventKind::GoalEntered
                | TraceEventKind::GoalStarted => {
                    let Some(goal_id) = event.goal_id else {
                        continue;
                    };
                    let node_id = *goal_nodes.entry(goal_id).or_insert_with(|| {
                        let node_id = goal_node_id(goal_id);
                        graph.nodes.push(GraphNode {
                            id: node_id,
                            kind: GraphNodeKind::Goal,
                            label: event.title.clone(),
                            span_id: event.span_id,
                            metadata: event.metadata.clone(),
                        });
                        node_id
                    });

                    if let Some(parent_goal_id) = event.parent_goal_id {
                        graph.edges.push(make_edge(
                            goal_node_id(parent_goal_id),
                            node_id,
                            "nested",
                        ));
                    } else if let Some(subject_id) = event.subject_id {
                        let subject_node_id =
                            ensure_subject_placeholder(&mut graph, &mut subject_nodes, subject_id);
                        graph.edges.push(make_edge(subject_node_id, node_id, "goal"));
                    }
                },
                TraceEventKind::NestedObligation => {
                    let Some(parent_goal_id) = event.parent_goal_id.or(event.goal_id) else {
                        continue;
                    };
                    let node_id = event_node_id(event.id.value());
                    graph.nodes.push(GraphNode {
                        id: node_id,
                        kind: GraphNodeKind::Goal,
                        label: event.title.clone(),
                        span_id: event.span_id,
                        metadata: event.metadata.clone(),
                    });
                    graph.edges.push(make_edge(
                        goal_node_id(parent_goal_id),
                        node_id,
                        "obligation",
                    ));
                },
                TraceEventKind::CandidateDiscovered
                | TraceEventKind::CandidateTried
                | TraceEventKind::CandidateResult
                | TraceEventKind::CandidateChosen
                | TraceEventKind::CandidateRejected => {
                    let Some(goal_id) = event.goal_id else {
                        continue;
                    };
                    let Some(candidate_id) = event.candidate_id else {
                        continue;
                    };
                    let node_id = *candidate_nodes.entry(candidate_id).or_insert_with(|| {
                        let node_id = candidate_node_id(candidate_id);
                        graph.nodes.push(GraphNode {
                            id: node_id,
                            kind: GraphNodeKind::Candidate,
                            label: event.title.clone(),
                            span_id: event.span_id,
                            metadata: event.metadata.clone(),
                        });
                        node_id
                    });
                    graph.edges.push(make_edge(
                        goal_node_id(goal_id),
                        node_id,
                        candidate_edge_label(&event.kind),
                    ));
                },
                TraceEventKind::BranchChosen
                | TraceEventKind::AliasExpanded
                | TraceEventKind::Normalization
                | TraceEventKind::ErrorRaised => {
                    let node_id = event_node_id(event.id.value());
                    graph.nodes.push(GraphNode {
                        id: node_id,
                        kind: GraphNodeKind::Expression,
                        label: event.title.clone(),
                        span_id: event.span_id,
                        metadata: event.metadata.clone(),
                    });
                    if let Some(parent_goal_id) = event.goal_id.or(event.parent_goal_id) {
                        graph.edges.push(make_edge(
                            goal_node_id(parent_goal_id),
                            node_id,
                            expression_edge_label(&event.kind),
                        ));
                    } else if let Some(subject_id) = event.subject_id {
                        let subject_node_id =
                            ensure_subject_placeholder(&mut graph, &mut subject_nodes, subject_id);
                        graph.edges.push(make_edge(
                            subject_node_id,
                            node_id,
                            expression_edge_label(&event.kind),
                        ));
                    }
                },
                _ => {},
            }
        }

        graph
    }

    #[must_use]
    pub fn build_legacy_from_depth(&self, trace: &Trace) -> Graph {
        let mut graph = Graph::default();
        let mut stack: Vec<(usize, NodeId)> = Vec::new();

        for event in &trace.events {
            let Some(depth) =
                event.metadata.get("depth").and_then(|value| value.parse::<usize>().ok())
            else {
                continue;
            };

            while stack.last().is_some_and(|(stack_depth, _)| *stack_depth >= depth) {
                stack.pop();
            }

            let node_id = NodeId::new(event.id.value());
            graph.nodes.push(GraphNode {
                id: node_id,
                kind: legacy_node_kind(&event.kind),
                label: event.title.clone(),
                span_id: event.span_id,
                metadata: event.metadata.clone(),
            });

            if let Some((_, parent_id)) = stack.last().copied() {
                graph.edges.push(make_edge(
                    parent_id,
                    node_id,
                    expression_edge_label(&event.kind),
                ));
            }

            stack.push((depth, node_id));
        }

        graph
    }

    #[must_use]
    pub fn subgraph(&self, graph: &Graph, pattern: &str, max_depth: usize) -> Graph {
        let pattern_lower = pattern.to_lowercase();
        let seeds: BTreeSet<NodeId> = graph
            .nodes
            .iter()
            .filter(|node| node.label.to_lowercase().contains(&pattern_lower))
            .map(|node| node.id)
            .collect();
        if seeds.is_empty() {
            return Graph::default();
        }

        let mut parents = BTreeMap::<NodeId, Vec<NodeId>>::new();
        let mut children = BTreeMap::<NodeId, Vec<NodeId>>::new();
        for edge in &graph.edges {
            parents.entry(edge.to).or_default().push(edge.from);
            children.entry(edge.from).or_default().push(edge.to);
        }

        let mut keep = seeds.clone();
        let limit = if max_depth == 0 {
            usize::MAX
        } else {
            max_depth
        };
        let mut queue: VecDeque<(NodeId, usize)> =
            seeds.into_iter().map(|node_id| (node_id, 0)).collect();

        while let Some((node_id, distance)) = queue.pop_front() {
            if distance >= limit {
                continue;
            }
            for parent in parents.get(&node_id).into_iter().flatten().copied() {
                if keep.insert(parent) {
                    queue.push_back((parent, distance + 1));
                }
            }
            for child in children.get(&node_id).into_iter().flatten().copied() {
                if keep.insert(child) {
                    queue.push_back((child, distance + 1));
                }
            }
        }

        Graph {
            nodes: graph.nodes.iter().filter(|node| keep.contains(&node.id)).cloned().collect(),
            edges: graph
                .edges
                .iter()
                .filter(|edge| keep.contains(&edge.from) && keep.contains(&edge.to))
                .cloned()
                .collect(),
        }
    }

    #[must_use]
    pub fn to_mermaid(&self, graph: &Graph) -> String {
        let mut out = String::from("flowchart TD\n");
        for node in &graph.nodes {
            let id = node.id.value();
            let label = node.label.replace('"', "'");
            let shape = match node.kind {
                GraphNodeKind::Goal => format!("n{id}([\"{label}\"])"),
                GraphNodeKind::Candidate => format!("n{id}[\"{label}\"]"),
                GraphNodeKind::Expression => format!("n{id}{{\"{label}\"}}"),
                GraphNodeKind::Semantic => format!("n{id}((\"{label}\"))"),
            };
            out.push_str(&format!("  {shape}\n"));
        }
        for edge in &graph.edges {
            out.push_str(&format!(
                "  n{} -->|{}| n{}\n",
                edge.from.value(),
                edge.label.replace('"', "'"),
                edge.to.value()
            ));
        }
        out
    }

    #[must_use]
    pub fn to_dot(&self, graph: &Graph) -> String {
        let mut out = String::from("digraph obligation {\n  rankdir=TB;\n");
        for node in &graph.nodes {
            let shape = match node.kind {
                GraphNodeKind::Goal => "ellipse",
                GraphNodeKind::Candidate => "box",
                GraphNodeKind::Expression => "diamond",
                GraphNodeKind::Semantic => "hexagon",
            };
            out.push_str(&format!(
                "  n{} [label=\"{}\" shape={}];\n",
                node.id.value(),
                node.label.replace('"', "\\\""),
                shape
            ));
        }
        for edge in &graph.edges {
            out.push_str(&format!(
                "  n{} -> n{} [label=\"{}\"];\n",
                edge.from.value(),
                edge.to.value(),
                edge.label.replace('"', "\\\"")
            ));
        }
        out.push('}');
        out
    }
}

fn ensure_subject_node(
    graph: &mut Graph,
    subject_nodes: &mut BTreeMap<SubjectId, NodeId>,
    subject_id: SubjectId,
    event: &TraceEvent,
) -> NodeId {
    *subject_nodes.entry(subject_id).or_insert_with(|| {
        let node_id = subject_node_id(subject_id);
        graph.nodes.push(GraphNode {
            id: node_id,
            kind: GraphNodeKind::Semantic,
            label: event.title.clone(),
            span_id: event.span_id,
            metadata: event.metadata.clone(),
        });
        node_id
    })
}

fn ensure_subject_placeholder(
    graph: &mut Graph,
    subject_nodes: &mut BTreeMap<SubjectId, NodeId>,
    subject_id: SubjectId,
) -> NodeId {
    *subject_nodes.entry(subject_id).or_insert_with(|| {
        let node_id = subject_node_id(subject_id);
        graph.nodes.push(GraphNode {
            id: node_id,
            kind: GraphNodeKind::Semantic,
            label: format!("subject:{}", subject_id.value()),
            span_id: None,
            metadata: BTreeMap::new(),
        });
        node_id
    })
}

fn make_edge(from: NodeId, to: NodeId, label: &str) -> GraphEdge {
    GraphEdge {
        from,
        to,
        label: String::from(label),
        metadata: BTreeMap::new(),
    }
}

const fn subject_node_id(subject_id: SubjectId) -> NodeId {
    NodeId::new(SUBJECT_NODE_BASE + subject_id.value())
}

const fn goal_node_id(goal_id: GoalId) -> NodeId {
    NodeId::new(GOAL_NODE_BASE + goal_id.value())
}

const fn candidate_node_id(candidate_id: CandidateId) -> NodeId {
    NodeId::new(CANDIDATE_NODE_BASE + candidate_id.value())
}

const fn event_node_id(event_id: u64) -> NodeId {
    NodeId::new(EVENT_NODE_BASE + event_id)
}

fn candidate_edge_label(kind: &TraceEventKind) -> &'static str {
    match kind {
        TraceEventKind::CandidateDiscovered => "candidate",
        TraceEventKind::CandidateResult => "result",
        TraceEventKind::CandidateChosen => "chosen",
        TraceEventKind::CandidateRejected => "rejected",
        _ => "tried",
    }
}

fn expression_edge_label(kind: &TraceEventKind) -> &'static str {
    match kind {
        TraceEventKind::AliasExpanded => "alias",
        TraceEventKind::Normalization => "normalize",
        TraceEventKind::BranchChosen => "branch",
        TraceEventKind::ErrorRaised => "error",
        TraceEventKind::CandidateDiscovered => "candidate",
        TraceEventKind::CandidateResult => "result",
        TraceEventKind::CandidateChosen => "chosen",
        TraceEventKind::CandidateRejected => "rejected",
        _ => "nested",
    }
}

fn legacy_node_kind(kind: &TraceEventKind) -> GraphNodeKind {
    match kind {
        TraceEventKind::CandidateDiscovered
        | TraceEventKind::CandidateTried
        | TraceEventKind::CandidateResult
        | TraceEventKind::CandidateChosen
        | TraceEventKind::CandidateRejected => GraphNodeKind::Candidate,
        TraceEventKind::AliasExpanded
        | TraceEventKind::Normalization
        | TraceEventKind::BranchChosen => GraphNodeKind::Expression,
        _ => GraphNodeKind::Goal,
    }
}

#[cfg(test)]
mod tests {
    use typelude_tooling_core::{
        CandidateId, EventId, GoalId, SubjectId, SubjectKind, Trace, TraceEvent, TraceEventKind,
        TraceId,
    };

    use super::TraceGraphBuilder;

    fn make_event(id: u64, kind: TraceEventKind, title: &str) -> TraceEvent {
        TraceEvent::new(EventId::new(id), kind, title)
    }

    #[test]
    fn builds_nested_goal_graph() {
        let mut trace = Trace::new(TraceId::new(1));
        let mut outer_subject = make_event(1, TraceEventKind::SubjectDiscovered, "predicate");
        outer_subject.subject_id = Some(SubjectId::new(1));
        outer_subject.subject_kind = Some(SubjectKind::Predicate);
        let mut outer = make_event(2, TraceEventKind::GoalEntered, "outer");
        outer.goal_id = Some(GoalId::new(1));
        outer.subject_id = Some(SubjectId::new(1));
        let mut inner = make_event(3, TraceEventKind::GoalEntered, "inner");
        inner.goal_id = Some(GoalId::new(2));
        inner.parent_goal_id = Some(GoalId::new(1));
        inner.subject_id = Some(SubjectId::new(1));
        trace.push(outer_subject);
        trace.push(outer);
        trace.push(inner);

        let graph = TraceGraphBuilder::new().build(&trace);
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
        assert!(graph.edges.iter().any(|edge| edge.label == "goal"));
        assert!(graph.edges.iter().any(|edge| edge.label == "nested"));
    }

    #[test]
    fn candidate_nodes_are_wired_to_parent() {
        let mut trace = Trace::new(TraceId::new(1));
        let mut subject = make_event(1, TraceEventKind::SubjectDiscovered, "predicate");
        subject.subject_id = Some(SubjectId::new(1));
        subject.subject_kind = Some(SubjectKind::Predicate);
        let mut goal = make_event(2, TraceEventKind::GoalEntered, "goal");
        goal.goal_id = Some(GoalId::new(1));
        goal.subject_id = Some(SubjectId::new(1));
        let mut candidate = make_event(3, TraceEventKind::CandidateDiscovered, "impl A");
        candidate.goal_id = Some(GoalId::new(1));
        candidate.candidate_id = Some(CandidateId::new(1));
        candidate.subject_id = Some(SubjectId::new(1));
        trace.push(subject);
        trace.push(goal);
        trace.push(candidate);

        let graph = TraceGraphBuilder::new().build(&trace);
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
        assert!(graph.edges.iter().any(|edge| edge.label == "candidate"));
    }

    #[test]
    fn to_mermaid_renders_valid_structure() {
        let mut trace = Trace::new(TraceId::new(1));
        let mut subject = make_event(1, TraceEventKind::SubjectDiscovered, "predicate");
        subject.subject_id = Some(SubjectId::new(1));
        subject.subject_kind = Some(SubjectKind::Predicate);
        let mut event = make_event(2, TraceEventKind::GoalEntered, "EIf");
        event.goal_id = Some(GoalId::new(1));
        event.subject_id = Some(SubjectId::new(1));
        trace.push(subject);
        trace.push(event);

        let graph = TraceGraphBuilder::new().build(&trace);
        let mermaid = TraceGraphBuilder::new().to_mermaid(&graph);
        assert!(mermaid.starts_with("flowchart TD\n"));
        assert!(mermaid.contains("n1000000001"));
    }
}
