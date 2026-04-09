use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    ToolingResult,
    diagnostic::DiagnosticRecord,
    ids::{CandidateId, DiagId, EventId, GoalId, HookId, RunId, SpanId, SubjectId, TraceId},
    semantic::SemanticTag,
    subject::SubjectKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceEventKind {
    RunStarted,
    RunFinished,
    SubjectDiscovered,
    GoalDiscovered,
    GoalEntered,
    GoalExited,
    CandidateDiscovered,
    CandidateTried,
    CandidateResult,
    DiagnosticEmitted,
    RelationDeclared,
    Info,
    ErrorRaised,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredicateRepr {
    DebugText(String),
    TraitPredicate {
        trait_path: String,
        self_ty: String,
        args: Vec<String>,
    },
    AliasRelate {
        lhs: String,
        rhs: String,
    },
    NormalizesTo {
        alias: String,
        target: String,
    },
    Unknown,
}

impl PredicateRepr {
    #[must_use]
    pub fn debug_text(&self) -> String {
        match self {
            Self::DebugText(text) => text.clone(),
            Self::TraitPredicate {
                trait_path,
                self_ty,
                args,
            } => {
                if args.is_empty() {
                    format!("TraitPredicate({self_ty} -> {trait_path})")
                } else {
                    format!("TraitPredicate({self_ty} -> {trait_path}({}))", args.join(", "))
                }
            },
            Self::AliasRelate {
                lhs,
                rhs,
            } => format!("AliasRelate({lhs}, {rhs})"),
            Self::NormalizesTo {
                alias,
                target,
            } => format!("NormalizesTo({alias}, {target})"),
            Self::Unknown => String::from("Unknown"),
        }
    }

    #[must_use]
    pub fn family(&self) -> &'static str {
        match self {
            Self::DebugText(..) => "DebugText",
            Self::TraitPredicate {
                ..
            } => "TraitPredicate",
            Self::AliasRelate {
                ..
            } => "AliasRelate",
            Self::NormalizesTo {
                ..
            } => "NormalizesTo",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateKind {
    ParamEnv,
    Impl,
    Builtin,
    AliasRelate,
    Normalize,
    Unknown(String),
}

impl CandidateKind {
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::ParamEnv => String::from("ParamEnv"),
            Self::Impl => String::from("Impl"),
            Self::Builtin => String::from("Builtin"),
            Self::AliasRelate => String::from("AliasRelate"),
            Self::Normalize => String::from("Normalize"),
            Self::Unknown(text) => text.clone(),
        }
    }

    #[must_use]
    pub fn family(&self) -> &'static str {
        match self {
            Self::ParamEnv => "ParamEnv",
            Self::Impl => "Impl",
            Self::Builtin => "Builtin",
            Self::AliasRelate => "AliasRelate",
            Self::Normalize => "Normalize",
            Self::Unknown(..) => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalResult {
    Success,
    NoSolution,
    Ambiguous,
    Unsupported,
    Error,
}

impl GoalResult {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::NoSolution => "no_solution",
            Self::Ambiguous => "ambiguous",
            Self::Unsupported => "unsupported",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunStarted {
    pub crate_name: String,
    pub rustc_version: String,
    pub subject_filter: Option<String>,
    pub max_events: usize,
    pub max_depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunFinished {
    pub crate_name: String,
    pub goal_count: usize,
    pub candidate_count: usize,
    pub subject_count: usize,
    pub diagnostic_count: usize,
    pub dropped_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubjectDiscovered {
    pub hook_id: HookId,
    pub subject_id: SubjectId,
    pub parent_subject_id: Option<SubjectId>,
    pub subject_kind: SubjectKind,
    pub label: String,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalDiscovered {
    pub hook_id: HookId,
    pub subject_id: SubjectId,
    pub goal_id: GoalId,
    pub parent_goal_id: Option<GoalId>,
    pub predicate: PredicateRepr,
    pub candidate_count: usize,
    pub semantic_tags: Vec<SemanticTag>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalEntered {
    pub hook_id: HookId,
    pub subject_id: SubjectId,
    pub goal_id: GoalId,
    pub parent_goal_id: Option<GoalId>,
    pub predicate: PredicateRepr,
    pub semantic_tags: Vec<SemanticTag>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoalExited {
    pub hook_id: HookId,
    pub subject_id: SubjectId,
    pub goal_id: GoalId,
    pub parent_goal_id: Option<GoalId>,
    pub predicate: PredicateRepr,
    pub result: GoalResult,
    pub semantic_tags: Vec<SemanticTag>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateDiscovered {
    pub hook_id: HookId,
    pub subject_id: SubjectId,
    pub goal_id: GoalId,
    pub candidate_id: CandidateId,
    pub candidate_kind: CandidateKind,
    pub semantic_tags: Vec<SemanticTag>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateTried {
    pub hook_id: HookId,
    pub subject_id: SubjectId,
    pub goal_id: GoalId,
    pub candidate_id: CandidateId,
    pub candidate_kind: CandidateKind,
    pub semantic_tags: Vec<SemanticTag>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateResult {
    pub hook_id: HookId,
    pub subject_id: SubjectId,
    pub goal_id: GoalId,
    pub candidate_id: CandidateId,
    pub candidate_kind: CandidateKind,
    pub result: GoalResult,
    pub semantic_tags: Vec<SemanticTag>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticEmitted {
    pub hook_id: Option<HookId>,
    pub diagnostic_id: DiagId,
    pub subject_id: Option<SubjectId>,
    pub goal_id: Option<GoalId>,
    pub record: DiagnosticRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationDeclared {
    pub from: String,
    pub to: String,
    pub relation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InfoEvent {
    pub hook_id: Option<HookId>,
    pub message: String,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorRaised {
    pub hook_id: Option<HookId>,
    pub subject_id: Option<SubjectId>,
    pub goal_id: Option<GoalId>,
    pub result: GoalResult,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum TracePayload {
    RunStarted(RunStarted),
    RunFinished(RunFinished),
    SubjectDiscovered(SubjectDiscovered),
    GoalDiscovered(GoalDiscovered),
    GoalEntered(GoalEntered),
    GoalExited(GoalExited),
    CandidateDiscovered(CandidateDiscovered),
    CandidateTried(CandidateTried),
    CandidateResult(CandidateResult),
    DiagnosticEmitted(DiagnosticEmitted),
    RelationDeclared(RelationDeclared),
    Info(InfoEvent),
    ErrorRaised(ErrorRaised),
}

impl TracePayload {
    #[must_use]
    pub const fn kind(&self) -> TraceEventKind {
        match self {
            Self::RunStarted(..) => TraceEventKind::RunStarted,
            Self::RunFinished(..) => TraceEventKind::RunFinished,
            Self::SubjectDiscovered(..) => TraceEventKind::SubjectDiscovered,
            Self::GoalDiscovered(..) => TraceEventKind::GoalDiscovered,
            Self::GoalEntered(..) => TraceEventKind::GoalEntered,
            Self::GoalExited(..) => TraceEventKind::GoalExited,
            Self::CandidateDiscovered(..) => TraceEventKind::CandidateDiscovered,
            Self::CandidateTried(..) => TraceEventKind::CandidateTried,
            Self::CandidateResult(..) => TraceEventKind::CandidateResult,
            Self::DiagnosticEmitted(..) => TraceEventKind::DiagnosticEmitted,
            Self::RelationDeclared(..) => TraceEventKind::RelationDeclared,
            Self::Info(..) => TraceEventKind::Info,
            Self::ErrorRaised(..) => TraceEventKind::ErrorRaised,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceEvent {
    pub id: EventId,
    pub run_id: Option<RunId>,
    pub span_id: Option<SpanId>,
    pub payload: TracePayload,
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
    pub fn new(id: EventId, payload: TracePayload) -> Self {
        Self {
            id,
            run_id: None,
            span_id: None,
            payload,
        }
    }

    #[must_use]
    pub const fn kind(&self) -> TraceEventKind {
        self.payload.kind()
    }

    #[must_use]
    pub fn subject_id(&self) -> Option<SubjectId> {
        match &self.payload {
            TracePayload::SubjectDiscovered(data) => Some(data.subject_id),
            TracePayload::GoalDiscovered(data) => Some(data.subject_id),
            TracePayload::GoalEntered(data) => Some(data.subject_id),
            TracePayload::GoalExited(data) => Some(data.subject_id),
            TracePayload::CandidateDiscovered(data) => Some(data.subject_id),
            TracePayload::CandidateTried(data) => Some(data.subject_id),
            TracePayload::CandidateResult(data) => Some(data.subject_id),
            TracePayload::DiagnosticEmitted(data) => data.subject_id,
            TracePayload::ErrorRaised(data) => data.subject_id,
            TracePayload::RunStarted(..)
            | TracePayload::RunFinished(..)
            | TracePayload::RelationDeclared(..)
            | TracePayload::Info(..) => None,
        }
    }

    #[must_use]
    pub fn goal_id(&self) -> Option<GoalId> {
        match &self.payload {
            TracePayload::GoalDiscovered(data) => Some(data.goal_id),
            TracePayload::GoalEntered(data) => Some(data.goal_id),
            TracePayload::GoalExited(data) => Some(data.goal_id),
            TracePayload::CandidateDiscovered(data) => Some(data.goal_id),
            TracePayload::CandidateTried(data) => Some(data.goal_id),
            TracePayload::CandidateResult(data) => Some(data.goal_id),
            TracePayload::DiagnosticEmitted(data) => data.goal_id,
            TracePayload::ErrorRaised(data) => data.goal_id,
            TracePayload::RunStarted(..)
            | TracePayload::RunFinished(..)
            | TracePayload::SubjectDiscovered(..)
            | TracePayload::RelationDeclared(..)
            | TracePayload::Info(..) => None,
        }
    }

    #[must_use]
    pub fn parent_goal_id(&self) -> Option<GoalId> {
        match &self.payload {
            TracePayload::GoalDiscovered(data) => data.parent_goal_id,
            TracePayload::GoalEntered(data) => data.parent_goal_id,
            TracePayload::GoalExited(data) => data.parent_goal_id,
            _ => None,
        }
    }

    #[must_use]
    pub fn parent_subject_id(&self) -> Option<SubjectId> {
        match &self.payload {
            TracePayload::SubjectDiscovered(data) => data.parent_subject_id,
            _ => None,
        }
    }

    #[must_use]
    pub fn candidate_id(&self) -> Option<CandidateId> {
        match &self.payload {
            TracePayload::CandidateDiscovered(data) => Some(data.candidate_id),
            TracePayload::CandidateTried(data) => Some(data.candidate_id),
            TracePayload::CandidateResult(data) => Some(data.candidate_id),
            _ => None,
        }
    }

    #[must_use]
    pub fn diagnostic_id(&self) -> Option<DiagId> {
        match &self.payload {
            TracePayload::DiagnosticEmitted(data) => Some(data.diagnostic_id),
            _ => None,
        }
    }

    #[must_use]
    pub fn hook_id(&self) -> Option<HookId> {
        match &self.payload {
            TracePayload::SubjectDiscovered(data) => Some(data.hook_id),
            TracePayload::GoalDiscovered(data) => Some(data.hook_id),
            TracePayload::GoalEntered(data) => Some(data.hook_id),
            TracePayload::GoalExited(data) => Some(data.hook_id),
            TracePayload::CandidateDiscovered(data) => Some(data.hook_id),
            TracePayload::CandidateTried(data) => Some(data.hook_id),
            TracePayload::CandidateResult(data) => Some(data.hook_id),
            TracePayload::DiagnosticEmitted(data) => data.hook_id,
            TracePayload::Info(data) => data.hook_id,
            TracePayload::ErrorRaised(data) => data.hook_id,
            TracePayload::RunStarted(..)
            | TracePayload::RunFinished(..)
            | TracePayload::RelationDeclared(..) => None,
        }
    }

    #[must_use]
    pub fn title(&self) -> String {
        match &self.payload {
            TracePayload::RunStarted(data) => data.crate_name.clone(),
            TracePayload::RunFinished(data) => data.crate_name.clone(),
            TracePayload::SubjectDiscovered(data) => data.label.clone(),
            TracePayload::GoalDiscovered(data) => data.predicate.debug_text(),
            TracePayload::GoalEntered(data) => data.predicate.debug_text(),
            TracePayload::GoalExited(data) => data.predicate.debug_text(),
            TracePayload::CandidateDiscovered(data) => data.candidate_kind.label(),
            TracePayload::CandidateTried(data) => data.candidate_kind.label(),
            TracePayload::CandidateResult(data) => data.candidate_kind.label(),
            TracePayload::DiagnosticEmitted(data) => data.record.message(),
            TracePayload::RelationDeclared(data) => data.relation.clone(),
            TracePayload::Info(data) => data.message.clone(),
            TracePayload::ErrorRaised(data) => data.message.clone(),
        }
    }

    #[must_use]
    pub fn detail(&self) -> Option<String> {
        match &self.payload {
            TracePayload::GoalExited(data) => Some(String::from(data.result.label())),
            TracePayload::CandidateResult(data) => Some(String::from(data.result.label())),
            TracePayload::ErrorRaised(data) => Some(String::from(data.result.label())),
            _ => None,
        }
    }

    #[must_use]
    pub fn metadata(&self) -> BTreeMap<String, String> {
        match &self.payload {
            TracePayload::RunStarted(data) => {
                let mut metadata = BTreeMap::new();
                metadata.insert(String::from("rustc_version"), data.rustc_version.clone());
                metadata.insert(String::from("max_events"), data.max_events.to_string());
                metadata.insert(String::from("max_depth"), data.max_depth.to_string());
                if let Some(subject_filter) = &data.subject_filter {
                    metadata.insert(String::from("subject_filter"), subject_filter.clone());
                }
                metadata
            },
            TracePayload::RunFinished(data) => {
                let mut metadata = BTreeMap::new();
                metadata.insert(String::from("goal_count"), data.goal_count.to_string());
                metadata.insert(String::from("candidate_count"), data.candidate_count.to_string());
                metadata.insert(String::from("subject_count"), data.subject_count.to_string());
                metadata
                    .insert(String::from("diagnostic_count"), data.diagnostic_count.to_string());
                metadata.insert(String::from("dropped_count"), data.dropped_count.to_string());
                metadata
            },
            TracePayload::SubjectDiscovered(data) => data.metadata.clone(),
            TracePayload::GoalDiscovered(data) => {
                let mut metadata = BTreeMap::new();
                metadata.insert(String::from("candidate_count"), data.candidate_count.to_string());
                metadata
            },
            TracePayload::GoalEntered(data) => {
                let mut metadata = BTreeMap::new();
                if !data.semantic_tags.is_empty() {
                    metadata.insert(
                        String::from("semantic_tags"),
                        data.semantic_tags
                            .iter()
                            .map(|tag| tag.label())
                            .collect::<Vec<_>>()
                            .join(","),
                    );
                }
                metadata
            },
            TracePayload::GoalExited(data) => {
                let mut metadata = BTreeMap::new();
                metadata.insert(String::from("result"), String::from(data.result.label()));
                metadata
            },
            TracePayload::CandidateDiscovered(data) => data.metadata.clone(),
            TracePayload::CandidateTried(data) => data.metadata.clone(),
            TracePayload::CandidateResult(data) => data.metadata.clone(),
            TracePayload::DiagnosticEmitted(data) => data.record.metadata().clone(),
            TracePayload::RelationDeclared(..) => BTreeMap::new(),
            TracePayload::Info(data) => data.metadata.clone(),
            TracePayload::ErrorRaised(data) => {
                let mut metadata = BTreeMap::new();
                metadata.insert(String::from("result"), String::from(data.result.label()));
                metadata
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        EventId, GoalId, GoalResult, HookId, PredicateRepr, RunStarted, SubjectDiscovered, Trace,
        TraceEvent, TraceId, TracePayload,
    };

    #[test]
    fn trace_serializes_stably() {
        let mut trace = Trace::new(TraceId::new(1));
        trace.push(TraceEvent::new(
            EventId::new(1),
            TracePayload::RunStarted(RunStarted {
                crate_name: String::from("typelude"),
                rustc_version: String::from("nightly"),
                subject_filter: None,
                max_events: 10,
                max_depth: 3,
            }),
        ));

        let json = serde_json::to_value(&trace).expect("trace should serialize");
        assert_eq!(json["id"], 1);
        assert_eq!(json["events"][0]["payload"]["kind"], "run_started");
    }

    #[test]
    fn trace_json_lines_roundtrip() {
        let mut trace = Trace::new(TraceId::new(7));
        trace.push(TraceEvent::new(
            EventId::new(1),
            TracePayload::SubjectDiscovered(SubjectDiscovered {
                hook_id: HookId::TraitSolve,
                subject_id: crate::SubjectId::new(9),
                parent_subject_id: None,
                subject_kind: crate::SubjectKind::Predicate,
                label: String::from("predicate"),
                metadata: BTreeMap::new(),
            }),
        ));
        trace.push(TraceEvent::new(
            EventId::new(2),
            TracePayload::GoalExited(crate::GoalExited {
                hook_id: HookId::TraitSolve,
                subject_id: crate::SubjectId::new(9),
                goal_id: GoalId::new(2),
                parent_goal_id: None,
                predicate: PredicateRepr::DebugText(String::from("goal")),
                result: GoalResult::Success,
                semantic_tags: Vec::new(),
            }),
        ));

        let lines = trace.to_json_lines().expect("trace should render as json lines");
        let parsed = Trace::from_json_lines(TraceId::new(7), &lines).expect("trace should parse");

        assert_eq!(parsed.events.len(), 2);
        assert_eq!(parsed.events[0].kind(), crate::TraceEventKind::SubjectDiscovered);
        assert_eq!(parsed.events[1].kind(), crate::TraceEventKind::GoalExited);
    }
}
