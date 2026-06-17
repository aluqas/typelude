use std::collections::BTreeMap;

use serde::Serialize;
use typelude_tooling_core::{
    CandidateKind, EventId, NodeId, PredicateRepr, SemanticTag, Trace, TracePayload,
};

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
            .filter_map(|event| match &event.payload {
                TracePayload::RunStarted(..)
                | TracePayload::RunFinished(..)
                | TracePayload::DefinitionGraphEmitted(..)
                | TracePayload::Info(..) => None,
                TracePayload::SubjectDiscovered(data) => Some(SemanticNode {
                    id: NodeId::new(event.id.value()),
                    kind: SemanticNodeKind::Eval,
                    label: compress_symbol_name(&data.label),
                    source_event_id: Some(event.id),
                    metadata: data.metadata.clone(),
                }),
                TracePayload::GoalDiscovered(data) => Some(SemanticNode {
                    id: NodeId::new(event.id.value()),
                    kind: classify_semantic_kind_from_tags(&data.semantic_tags, &data.predicate),
                    label: compress_symbol_name(&data.predicate.debug_text()),
                    source_event_id: Some(event.id),
                    metadata: BTreeMap::new(),
                }),
                TracePayload::GoalEntered(data) => Some(SemanticNode {
                    id: NodeId::new(event.id.value()),
                    kind: classify_semantic_kind_from_tags(&data.semantic_tags, &data.predicate),
                    label: compress_symbol_name(&data.predicate.debug_text()),
                    source_event_id: Some(event.id),
                    metadata: BTreeMap::new(),
                }),
                TracePayload::GoalExited(data) => Some(SemanticNode {
                    id: NodeId::new(event.id.value()),
                    kind: classify_semantic_kind_from_tags(&data.semantic_tags, &data.predicate),
                    label: compress_symbol_name(&data.predicate.debug_text()),
                    source_event_id: Some(event.id),
                    metadata: BTreeMap::new(),
                }),
                TracePayload::CandidateDiscovered(data) => Some(SemanticNode {
                    id: NodeId::new(event.id.value()),
                    kind: classify_candidate_kind(&data.candidate_kind),
                    label: compress_symbol_name(&data.candidate_kind.label()),
                    source_event_id: Some(event.id),
                    metadata: data.metadata.clone(),
                }),
                TracePayload::CandidateTried(data) => Some(SemanticNode {
                    id: NodeId::new(event.id.value()),
                    kind: classify_candidate_kind(&data.candidate_kind),
                    label: compress_symbol_name(&data.candidate_kind.label()),
                    source_event_id: Some(event.id),
                    metadata: data.metadata.clone(),
                }),
                TracePayload::CandidateResult(data) => Some(SemanticNode {
                    id: NodeId::new(event.id.value()),
                    kind: classify_candidate_kind(&data.candidate_kind),
                    label: compress_symbol_name(&data.candidate_kind.label()),
                    source_event_id: Some(event.id),
                    metadata: data.metadata.clone(),
                }),
                TracePayload::DiagnosticEmitted(data) => Some(SemanticNode {
                    id: NodeId::new(event.id.value()),
                    kind: SemanticNodeKind::HelperDispatch,
                    label: compress_symbol_name(&data.record.message()),
                    source_event_id: Some(event.id),
                    metadata: data.record.metadata().clone(),
                }),
                TracePayload::RelationDeclared(data) => Some(SemanticNode {
                    id: NodeId::new(event.id.value()),
                    kind: SemanticNodeKind::PrimitiveOp,
                    label: compress_symbol_name(&data.relation),
                    source_event_id: Some(event.id),
                    metadata: BTreeMap::new(),
                }),
                TracePayload::ErrorRaised(data) => Some(SemanticNode {
                    id: NodeId::new(event.id.value()),
                    kind: SemanticNodeKind::PrimitiveOp,
                    label: compress_symbol_name(&data.message),
                    source_event_id: Some(event.id),
                    metadata: BTreeMap::new(),
                }),
            })
            .collect()
    }
}

fn classify_semantic_kind_from_tags(
    tags: &[SemanticTag],
    predicate: &PredicateRepr,
) -> SemanticNodeKind {
    if tags.contains(&SemanticTag::BranchLike) {
        SemanticNodeKind::If
    } else if tags.contains(&SemanticTag::LoopLike) {
        SemanticNodeKind::While
    } else if tags.contains(&SemanticTag::MapLike) {
        SemanticNodeKind::Map
    } else if tags.contains(&SemanticTag::LookupLike) {
        SemanticNodeKind::Get
    } else if tags.contains(&SemanticTag::ApplyLike) {
        SemanticNodeKind::Apply
    } else if tags.contains(&SemanticTag::HelperDispatchLike) {
        SemanticNodeKind::HelperDispatch
    } else if tags.contains(&SemanticTag::VmOpLike) {
        SemanticNodeKind::VmOp
    } else if predicate.debug_text().contains("Step") {
        SemanticNodeKind::VmStep
    } else if predicate.family() == "DebugText" {
        SemanticNodeKind::PrimitiveOp
    } else {
        SemanticNodeKind::Eval
    }
}

fn classify_candidate_kind(candidate: &CandidateKind) -> SemanticNodeKind {
    match candidate {
        CandidateKind::AliasRelate => SemanticNodeKind::HelperDispatch,
        CandidateKind::Normalize => SemanticNodeKind::Eval,
        CandidateKind::Unknown(text) if text.contains("Op") => SemanticNodeKind::VmOp,
        CandidateKind::Unknown(text) if text.contains("Step") => SemanticNodeKind::VmStep,
        CandidateKind::Unknown(_) => SemanticNodeKind::PrimitiveOp,
        CandidateKind::ParamEnv | CandidateKind::Impl | CandidateKind::Builtin => {
            SemanticNodeKind::Eval
        },
    }
}
