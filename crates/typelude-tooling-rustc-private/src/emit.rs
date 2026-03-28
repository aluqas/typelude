use typelude_tooling_core::{RunId, TraceEvent, TracePayload};

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

    pub fn emit(&mut self, payload: TracePayload) -> TraceEvent {
        self.next_event_id += 1;
        let mut event = TraceEvent::new(typelude_tooling_core::EventId::new(self.next_event_id), payload);
        event.run_id = Some(self.run_id);
        event
    }

    pub fn write(&mut self, event: &TraceEvent) {
        emit_raw(event);
    }

    pub fn write_payload(&mut self, payload: TracePayload) {
        let event = self.emit(payload);
        self.write(&event);
    }
}

pub fn emit_raw(event: &TraceEvent) {
    if let Ok(json) = serde_json::to_string(event) {
        eprintln!("{json}");
    }
}
