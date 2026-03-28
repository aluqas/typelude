use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    ids::{CandidateId, DiagId, EventId, GoalId, HookId, ItemId, RunId, SpanId, TraceId},
    ToolingResult,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceEventKind {
    RunStarted,
    RunFinished,
    ItemDiscovered,
    GoalDiscovered,
    GoalEntered,
    GoalExited,
    CandidateDiscovered,
    DiagnosticEmitted,
    RelationDeclared,
    Info,
    GoalStarted,
    GoalFinished,
    CandidateTried,
    CandidateResult,
    CandidateChosen,
    CandidateRejected,
    NestedObligation,
    AliasExpanded,
    Normalization,
    RecursionEntered,
    RecursionExited,
    BranchChosen,
    CacheHit,
    CacheMiss,
    ErrorRaised,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceEvent {
    pub id: EventId,
    pub run_id: Option<RunId>,
    pub hook_id: Option<HookId>,
    pub item_id: Option<ItemId>,
    pub goal_id: Option<GoalId>,
    pub parent_goal_id: Option<GoalId>,
    pub candidate_id: Option<CandidateId>,
    pub diagnostic_id: Option<DiagId>,
    pub span_id: Option<SpanId>,
    pub kind: TraceEventKind,
    pub title: String,
    pub detail: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trace {
    pub id: TraceId,
    pub events: Vec<TraceEvent>,
}

impl Trace {
    #[must_use]
    pub fn new(id: TraceId) -> Self {
        Self {
            id,
            events: Vec::new(),
        }
    }

    pub fn push(&mut self, event: TraceEvent) {
        self.events.push(event);
    }

    pub fn from_json_lines(id: TraceId, input: &str) -> ToolingResult<Self> {
        let mut trace = Self::new(id);
        for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
            if let Ok(event) = serde_json::from_str::<TraceEvent>(line) {
                trace.push(event);
            }
        }
        Ok(trace)
    }

    pub fn to_json_lines(&self) -> ToolingResult<String> {
        let mut lines = Vec::with_capacity(self.events.len());
        for event in &self.events {
            lines.push(serde_json::to_string(event)?);
        }
        Ok(lines.join("\n"))
    }
}

impl TraceEvent {
    #[must_use]
    pub fn new(id: EventId, kind: TraceEventKind, title: impl Into<String>) -> Self {
        Self {
            id,
            run_id: None,
            hook_id: None,
            item_id: None,
            goal_id: None,
            parent_goal_id: None,
            candidate_id: None,
            diagnostic_id: None,
            span_id: None,
            kind,
            title: title.into(),
            detail: None,
            metadata: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{EventId, Trace, TraceEvent, TraceEventKind, TraceId};

    #[test]
    fn trace_serializes_stably() {
        let mut trace = Trace::new(TraceId::new(1));
        let mut event = TraceEvent::new(EventId::new(1), TraceEventKind::GoalDiscovered, "goal");
        event.detail = Some(String::from("detail"));
        trace.push(event);

        let json = serde_json::to_value(&trace).expect("trace should serialize");
        assert_eq!(json["id"], 1);
        assert_eq!(json["events"][0]["title"], "goal");
    }

    #[test]
    fn trace_json_lines_roundtrip() {
        let mut trace = Trace::new(TraceId::new(7));
        trace.push(TraceEvent::new(
            EventId::new(1),
            TraceEventKind::RunStarted,
            "run-start",
        ));
        trace.push(TraceEvent::new(
            EventId::new(2),
            TraceEventKind::GoalEntered,
            "goal",
        ));

        let lines = trace.to_json_lines().expect("trace should render as json lines");
        let parsed = Trace::from_json_lines(TraceId::new(7), &lines).expect("trace should parse");

        assert_eq!(parsed.events.len(), 2);
        assert_eq!(parsed.events[0].kind, TraceEventKind::RunStarted);
        assert_eq!(parsed.events[1].kind, TraceEventKind::GoalEntered);
    }
}
