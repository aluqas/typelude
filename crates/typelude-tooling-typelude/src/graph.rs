use std::collections::{BTreeMap, BTreeSet, VecDeque};

use typelude_tooling_core::{
    CandidateId, GoalId, Graph, GraphEdge, GraphEdgeKind, GraphNode, GraphNodeKind, NodeId,
    SubjectId, Trace, TracePayload,
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
        let mut seen_edges = BTreeSet::<(NodeId, NodeId, GraphEdgeKind, String)>::new();

        for event in &trace.events {
            match &event.payload {
                TracePayload::SubjectDiscovered(data) => {
                    let node_id =
                        ensure_subject_node(&mut graph, &mut subject_nodes, data.subject_id, event);
                    if let Some(parent_subject_id) = data.parent_subject_id {
                        let parent_id = ensure_subject_placeholder(
                            &mut graph,
                            &mut subject_nodes,
                            parent_subject_id,
                        );
                        push_edge(
                            &mut graph,
                            &mut seen_edges,
                            parent_id,
                            node_id,
                            GraphEdgeKind::Subject,
                            "subject",
                        );
                    }
                },
                TracePayload::GoalDiscovered(data) => {
                    let node_id = *goal_nodes.entry(data.goal_id).or_insert_with(|| {
                        let node_id = goal_node_id(data.goal_id);
                        graph.nodes.push(GraphNode {
                            id: node_id,
                            kind: GraphNodeKind::Goal,
                            label: data.predicate.debug_text(),
                            span_id: event.span_id,
                            semantic_tags: data.semantic_tags.clone(),
                            metadata: BTreeMap::from([(
                                String::from("candidate_count"),
                                data.candidate_count.to_string(),
                            )]),
                        });
                        node_id
                    });

                    if let Some(parent_goal_id) = data.parent_goal_id {
                        push_edge(
                            &mut graph,
                            &mut seen_edges,
                            goal_node_id(parent_goal_id),
                            node_id,
                            GraphEdgeKind::NestedGoal,
                            "nested",
                        );
                    } else {
                        let subject_node_id =
                            ensure_subject_placeholder(&mut graph, &mut subject_nodes, data.subject_id);
                        push_edge(
                            &mut graph,
                            &mut seen_edges,
                            subject_node_id,
                            node_id,
                            GraphEdgeKind::Goal,
                            "goal",
                        );
                    }
                },
                TracePayload::CandidateDiscovered(data) => {
                    let node_id = *candidate_nodes.entry(data.candidate_id).or_insert_with(|| {
                        let node_id = candidate_node_id(data.candidate_id);
                        graph.nodes.push(GraphNode {
                            id: node_id,
                            kind: GraphNodeKind::Candidate,
                            label: data.candidate_kind.label(),
                            span_id: event.span_id,
                            semantic_tags: data.semantic_tags.clone(),
                            metadata: data.metadata.clone(),
                        });
                        node_id
                    });
                    push_edge(
                        &mut graph,
                        &mut seen_edges,
                        goal_node_id(data.goal_id),
                        node_id,
                        GraphEdgeKind::Candidate,
                        "candidate",
                    );
                },
                TracePayload::ErrorRaised(data) => {
                    let node_id = event_node_id(event.id.value());
                    graph.nodes.push(GraphNode {
                        id: node_id,
                        kind: GraphNodeKind::Expression,
                        label: data.message.clone(),
                        span_id: event.span_id,
                        semantic_tags: vec![],
                        metadata: BTreeMap::from([(
                            String::from("result"),
                            String::from(data.result.label()),
                        )]),
                    });
                    if let Some(goal_id) = data.goal_id {
                        push_edge(
                            &mut graph,
                            &mut seen_edges,
                            goal_node_id(goal_id),
                            node_id,
                            GraphEdgeKind::Error,
                            "error",
                        );
                    } else if let Some(subject_id) = data.subject_id {
                        let subject_node_id =
                            ensure_subject_placeholder(&mut graph, &mut subject_nodes, subject_id);
                        push_edge(
                            &mut graph,
                            &mut seen_edges,
                            subject_node_id,
                            node_id,
                            GraphEdgeKind::Error,
                            "error",
                        );
                    }
                },
                _ => {},
            }
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
    event: &typelude_tooling_core::TraceEvent,
) -> NodeId {
    *subject_nodes.entry(subject_id).or_insert_with(|| {
        let node_id = subject_node_id(subject_id);
        graph.nodes.push(GraphNode {
            id: node_id,
            kind: GraphNodeKind::Semantic,
            label: event.title(),
            span_id: event.span_id,
            semantic_tags: vec![],
            metadata: event.metadata(),
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
            semantic_tags: vec![],
            metadata: BTreeMap::new(),
        });
        node_id
    })
}

fn push_edge(
    graph: &mut Graph,
    seen_edges: &mut BTreeSet<(NodeId, NodeId, GraphEdgeKind, String)>,
    from: NodeId,
    to: NodeId,
    kind: GraphEdgeKind,
    label: &str,
) {
    let key = (from, to, kind.clone(), String::from(label));
    if seen_edges.insert(key) {
        graph.edges.push(GraphEdge {
            from,
            to,
            kind,
            label: String::from(label),
            metadata: BTreeMap::new(),
        });
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
