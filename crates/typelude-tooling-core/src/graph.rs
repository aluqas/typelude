use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    ids::{NodeId, SpanId},
    semantic::SemanticTag,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphNodeKind {
    Goal,
    Candidate,
    Expression,
    Semantic,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GraphEdgeKind {
    Subject,
    Goal,
    NestedGoal,
    Candidate,
    Error,
    Semantic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: NodeId,
    pub kind: GraphNodeKind,
    pub label: String,
    pub span_id: Option<SpanId>,
    pub semantic_tags: Vec<SemanticTag>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: GraphEdgeKind,
    pub label: String,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Graph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}
