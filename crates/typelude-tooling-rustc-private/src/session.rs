use std::collections::BTreeMap;

use rustc_middle::ty::TyCtxt;
use typelude_tooling_core::{
    AnalysisConfig, CandidateId, DiagId, FocusFilter, GoalId, HookId, SubjectDiscovered,
    SubjectFilter, SubjectId, TracePayload,
};

use crate::{
    emit::{CollectStats, EventEmitter},
    subjects::ResolvedSubject,
};

#[derive(Debug, Clone, Copy)]
struct SubjectRecord {
    id: SubjectId,
}

pub struct AnalysisSession<'a, 'tcx> {
    pub tcx: TyCtxt<'tcx>,
    pub emitter: &'a mut EventEmitter,
    pub config: &'a AnalysisConfig,
    pub stats: CollectStats,
    next_subject_id: u64,
    next_goal_id: u64,
    next_candidate_id: u64,
    next_diag_id: u64,
    subjects: BTreeMap<String, SubjectRecord>,
}

impl<'a, 'tcx> AnalysisSession<'a, 'tcx> {
    #[must_use]
    pub fn new(
        tcx: TyCtxt<'tcx>,
        emitter: &'a mut EventEmitter,
        config: &'a AnalysisConfig,
    ) -> Self {
        Self {
            tcx,
            emitter,
            config,
            stats: CollectStats::default(),
            next_subject_id: 1,
            next_goal_id: 1,
            next_candidate_id: 1,
            next_diag_id: 1,
            subjects: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn can_emit(&self) -> bool {
        self.config.max_events == 0 || self.emitter.emitted_count() < self.config.max_events
    }

    #[must_use]
    pub fn focus_matches(&self, value: &str) -> bool {
        FocusFilter::new(self.config.focus.clone()).matches(value)
    }

    #[must_use]
    pub fn subject_matches(&self, label: &str, metadata: &BTreeMap<String, String>) -> bool {
        SubjectFilter::new(self.config.subject_filter.clone()).matches(label, metadata)
    }

    #[must_use]
    pub fn alloc_goal_id(&mut self) -> GoalId {
        let id = GoalId::new(self.next_goal_id);
        self.next_goal_id += 1;
        id
    }

    #[must_use]
    pub fn alloc_candidate_id(&mut self) -> CandidateId {
        let id = CandidateId::new(self.next_candidate_id);
        self.next_candidate_id += 1;
        id
    }

    #[must_use]
    pub fn alloc_diag_id(&mut self) -> DiagId {
        let id = DiagId::new(self.next_diag_id);
        self.next_diag_id += 1;
        id
    }

    pub fn record_drop(&mut self) {
        self.stats.dropped_count += 1;
    }

    #[must_use]
    pub fn max_depth(&self) -> usize {
        self.config.max_depth
    }

    pub fn ensure_subject(
        &mut self,
        hook_id: HookId,
        subject: ResolvedSubject<'tcx>,
    ) -> SubjectId {
        let key = subject.key(self.tcx);
        if let Some(record) = self.subjects.get(&key).copied() {
            return record.id;
        }

        let parent_subject_id =
            subject.parent(self.tcx).map(|parent| self.ensure_subject(hook_id, parent));
        let subject_id = SubjectId::new(self.next_subject_id);
        self.next_subject_id += 1;
        self.subjects.insert(key, SubjectRecord {
            id: subject_id,
        });
        self.stats.subject_count += 1;

        if self.can_emit() {
            self.emitter.write_payload(TracePayload::SubjectDiscovered(SubjectDiscovered {
                hook_id,
                subject_id,
                parent_subject_id,
                subject_kind: subject.kind(),
                label: subject.label(self.tcx),
                metadata: subject.metadata(self.tcx),
            }));
        } else {
            self.record_drop();
        }

        subject_id
    }
}
