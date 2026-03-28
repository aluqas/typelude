use std::collections::BTreeMap;

use typelude_tooling_core::{
    CandidateId, DiagId, EventId, GoalId, HookId, RunId, SubjectId, SubjectKind, TraceEvent,
    TraceEventKind,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CollectStats {
    pub subject_count: usize,
    pub goal_count: usize,
    pub candidate_count: usize,
    pub diagnostic_count: usize,
    pub dropped_count: usize,
}

pub struct EventEmitter {
    run_id: RunId,
    next_event_id: u64,
}

impl EventEmitter {
    #[must_use]
    pub const fn new(run_id: RunId) -> Self {
        Self {
            run_id,
            next_event_id: 0,
        }
    }

    #[must_use]
    pub fn emitted_count(&self) -> usize {
        self.next_event_id as usize
    }

    pub fn emit(&mut self, kind: TraceEventKind, title: impl Into<String>) -> TraceEvent {
        self.next_event_id += 1;
        TraceEvent::new(EventId::new(self.next_event_id), kind, title).with_run_id(self.run_id)
    }

    pub fn write(&mut self, event: &TraceEvent) {
        emit_raw(event);
    }
}

pub fn emit_raw(event: &TraceEvent) {
    if let Ok(json) = serde_json::to_string(event) {
        eprintln!("{json}");
    }
}

pub trait TraceEventExt {
    fn with_run_id(self, run_id: RunId) -> Self;
    fn with_hook_id(self, hook_id: HookId) -> Self;
    fn with_subject(self, subject_id: SubjectId, subject_kind: SubjectKind) -> Self;
    fn with_parent_subject_id(self, parent_subject_id: SubjectId) -> Self;
    fn with_goal_id(self, goal_id: GoalId) -> Self;
    fn with_parent_goal_id(self, parent_goal_id: GoalId) -> Self;
    fn with_candidate_id(self, candidate_id: CandidateId) -> Self;
    fn with_diagnostic_id(self, diagnostic_id: DiagId) -> Self;
    fn with_detail(self, detail: impl Into<String>) -> Self;
    fn with_metadata(self, key: impl Into<String>, value: impl Into<String>) -> Self;
}

impl TraceEventExt for TraceEvent {
    fn with_run_id(mut self, run_id: RunId) -> Self {
        self.run_id = Some(run_id);
        self
    }

    fn with_hook_id(mut self, hook_id: HookId) -> Self {
        self.hook_id = Some(hook_id);
        self
    }

    fn with_subject(mut self, subject_id: SubjectId, subject_kind: SubjectKind) -> Self {
        self.subject_id = Some(subject_id);
        self.subject_kind = Some(subject_kind);
        self
    }

    fn with_parent_subject_id(mut self, parent_subject_id: SubjectId) -> Self {
        self.parent_subject_id = Some(parent_subject_id);
        self
    }

    fn with_goal_id(mut self, goal_id: GoalId) -> Self {
        self.goal_id = Some(goal_id);
        self
    }

    fn with_parent_goal_id(mut self, parent_goal_id: GoalId) -> Self {
        self.parent_goal_id = Some(parent_goal_id);
        self
    }

    fn with_candidate_id(mut self, candidate_id: CandidateId) -> Self {
        self.candidate_id = Some(candidate_id);
        self
    }

    fn with_diagnostic_id(mut self, diagnostic_id: DiagId) -> Self {
        self.diagnostic_id = Some(diagnostic_id);
        self
    }

    fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if self.metadata.is_empty() {
            self.metadata = BTreeMap::new();
        }
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use typelude_tooling_core::{HookId, RunId, SubjectId, SubjectKind, TraceEventKind};

    use super::{EventEmitter, TraceEventExt};

    #[test]
    fn emitter_assigns_event_ids() {
        let mut emitter = EventEmitter::new(RunId::new(1));
        let event = emitter
            .emit(TraceEventKind::RunStarted, "start")
            .with_hook_id(HookId::TraitSolve)
            .with_subject(SubjectId::new(7), SubjectKind::Predicate);
        assert_eq!(event.id.value(), 1);
        assert_eq!(event.run_id, Some(RunId::new(1)));
        assert_eq!(event.hook_id, Some(HookId::TraitSolve));
        assert_eq!(event.subject_id, Some(SubjectId::new(7)));
    }
}
