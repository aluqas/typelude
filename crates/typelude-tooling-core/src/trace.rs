use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    ToolingResult,
    ids::{CandidateId, DiagId, EventId, GoalId, HookId, RunId, SpanId, SubjectId, TraceId},
    subject::SubjectKind,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceEventKind {
    RunStarted,
    RunFinished,
    SubjectDiscovered,
    GoalDiscovered,
    GoalEntered,
    GoalExited,
    CandidateDiscovered,
    DiagnosticEmitted,
    RelationDeclared,
    Info,
    CandidateTried,
    CandidateResult,
    ErrorRaised,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceEvent {
    pub id: EventId,
    pub run_id: Option<RunId>,
    pub hook_id: Option<HookId>,
    pub subject_id: Option<SubjectId>,
    pub parent_subject_id: Option<SubjectId>,
    pub subject_kind: Option<SubjectKind>,
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
            if let Some(event) = parse_event_line(line) {
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

fn parse_event_line(line: &str) -> Option<TraceEvent> {
    if let Ok(event) = serde_json::from_str::<TraceEvent>(line) {
        return Some(event);
    }

    let value = serde_json::from_str::<Value>(line).ok()?;
    let message = value.get("message")?;
    serde_json::from_value::<TraceEvent>(message.clone()).ok()
}

impl TraceEvent {
    #[must_use]
    pub fn new(id: EventId, kind: TraceEventKind, title: impl Into<String>) -> Self {
        Self {
            id,
            run_id: None,
            hook_id: None,
            subject_id: None,
            parent_subject_id: None,
            subject_kind: None,
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
        let mut start = TraceEvent::new(EventId::new(1), TraceEventKind::RunStarted, "run-start");
        start.subject_kind = Some(crate::SubjectKind::Legacy);
        trace.push(start);
        let mut goal = TraceEvent::new(EventId::new(2), TraceEventKind::GoalEntered, "goal");
        goal.subject_id = Some(crate::SubjectId::new(9));
        goal.subject_kind = Some(crate::SubjectKind::Predicate);
        trace.push(goal);

        let lines = trace.to_json_lines().expect("trace should render as json lines");
        let parsed = Trace::from_json_lines(TraceId::new(7), &lines).expect("trace should parse");

        assert_eq!(parsed.events.len(), 2);
        assert_eq!(parsed.events[0].kind, TraceEventKind::RunStarted);
        assert_eq!(parsed.events[1].kind, TraceEventKind::GoalEntered);
        assert_eq!(parsed.events[1].subject_id, Some(crate::SubjectId::new(9)));
    }

    #[test]
    fn trace_parses_cargo_compiler_message_wrappers() {
        let line = r#"{"reason":"compiler-message","message":{"id":1,"run_id":null,"hook_id":null,"subject_id":9,"parent_subject_id":null,"subject_kind":"predicate","goal_id":null,"parent_goal_id":null,"candidate_id":null,"diagnostic_id":null,"span_id":null,"kind":"goal_entered","title":"goal","detail":null,"metadata":{}}}"#;
        let parsed =
            Trace::from_json_lines(TraceId::new(9), line).expect("wrapped trace should parse");
        assert_eq!(parsed.events.len(), 1);
        assert_eq!(parsed.events[0].kind, TraceEventKind::GoalEntered);
        assert_eq!(parsed.events[0].subject_id, Some(crate::SubjectId::new(9)));
    }
}
