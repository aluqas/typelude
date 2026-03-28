use std::collections::BTreeMap;

use serde::Serialize;
use typelude_tooling_core::{EventId, NodeId, Trace};

use crate::naming::compress_symbol_name;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum SemanticNodeKind {
    Eval,
    Apply,
    If,
    While,
    Map,
    Get,
    PrimitiveOp,
    HelperDispatch,
    VmOp,
    VmStep,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SemanticNode {
    pub id: NodeId,
    pub kind: SemanticNodeKind,
    pub label: String,
    pub source_event_id: Option<EventId>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Default)]
pub struct SemanticMapper;

impl SemanticMapper {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn map_trace(&self, trace: &Trace) -> Vec<SemanticNode> {
        trace
            .events
            .iter()
            .enumerate()
            .map(|(index, event)| SemanticNode {
                id: NodeId::new(index as u64 + 1),
                kind: classify_semantic_kind(&event.title),
                label: compress_symbol_name(&event.title),
                source_event_id: Some(event.id),
                metadata: event.metadata.clone(),
            })
            .collect()
    }
}

fn classify_semantic_kind(label: &str) -> SemanticNodeKind {
    if label.contains("EIf") || label.contains(" If ") || label == "EIf" {
        SemanticNodeKind::If
    } else if label.contains("EWhile") || label.contains("While") {
        SemanticNodeKind::While
    } else if label.contains("EMap") || label.contains("Map") {
        SemanticNodeKind::Map
    } else if label.contains("EGet") || label.contains("Get") {
        SemanticNodeKind::Get
    } else if label.contains("EApp") || label.contains("Apply") {
        SemanticNodeKind::Apply
    } else if label.contains("Helper") {
        SemanticNodeKind::HelperDispatch
    } else if label.contains("Op") {
        SemanticNodeKind::VmOp
    } else if label.contains("step") || label.contains("Step") {
        SemanticNodeKind::VmStep
    } else if label.starts_with('E') {
        SemanticNodeKind::Eval
    } else {
        SemanticNodeKind::PrimitiveOp
    }
}

#[cfg(test)]
mod tests {
    use typelude_tooling_core::{EventId, Trace, TraceEvent, TraceEventKind, TraceId};

    use super::{SemanticMapper, SemanticNodeKind};

    #[test]
    fn maps_key_typelude_nodes() {
        let mut trace = Trace::new(TraceId::new(1));
        for (id, title) in [(1, "EIf"), (2, "EWhile"), (3, "EGet"), (4, "EMap")] {
            trace.push(TraceEvent::new(EventId::new(id), TraceEventKind::GoalEntered, title));
        }

        let nodes = SemanticMapper::new().map_trace(&trace);
        assert_eq!(nodes[0].kind, SemanticNodeKind::If);
        assert_eq!(nodes[1].kind, SemanticNodeKind::While);
        assert_eq!(nodes[2].kind, SemanticNodeKind::Get);
        assert_eq!(nodes[3].kind, SemanticNodeKind::Map);
    }
}
