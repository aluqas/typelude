use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricKind {
    StepCount,
    ObligationCount,
    CandidateCount,
    BranchCount,
    RecursionDepth,
    TypeSizeBytes,
    ReEvaluationCount,
    WallTimeMillis,
    ArtifactCount,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricRecord {
    pub kind: MetricKind,
    pub name: String,
    pub value: f64,
    pub unit: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

impl MetricRecord {
    #[must_use]
    pub fn new(kind: MetricKind, name: impl Into<String>, value: f64) -> Self {
        Self {
            kind,
            name: name.into(),
            value,
            unit: None,
            metadata: BTreeMap::new(),
        }
    }
}
