use serde::{Deserialize, Serialize};

use crate::{QueryMatchKind, QueryTargetKind, SolveFilters};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolveViewKind {
    Tree,
    Summary,
    Analysis,
}

impl SolveViewKind {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Tree => "solve_tree",
            Self::Summary => "solve_summary",
            Self::Analysis => "solve_analysis",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolveTreeSemantics {
    RustcProofTree,
}

impl SolveTreeSemantics {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::RustcProofTree => "rustc_proof_tree",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SolveCollectionBasis {
    TraceInput,
    OwnerExplicitPredicates,
}

impl SolveCollectionBasis {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::TraceInput => "trace_input",
            Self::OwnerExplicitPredicates => "owner_explicit_predicates",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolveProvenance {
    pub view_kind: SolveViewKind,
    pub tree_semantics: SolveTreeSemantics,
    pub collection_basis: SolveCollectionBasis,
    pub trace_id: Option<u64>,
    pub query_kind: Option<QueryTargetKind>,
    pub query_owner: Option<String>,
    pub query_match: Option<QueryMatchKind>,
    pub input_path: Option<String>,
    pub manifest_path: Option<String>,
    pub filters: SolveFilters,
}

impl Default for SolveProvenance {
    fn default() -> Self {
        Self {
            view_kind: SolveViewKind::Tree,
            tree_semantics: SolveTreeSemantics::RustcProofTree,
            collection_basis: SolveCollectionBasis::TraceInput,
            trace_id: None,
            query_kind: None,
            query_owner: None,
            query_match: None,
            input_path: None,
            manifest_path: None,
            filters: SolveFilters::default(),
        }
    }
}

impl SolveProvenance {
    #[must_use]
    pub fn header_lines(&self, fallback_trace_id: Option<u64>) -> Vec<String> {
        let mut lines = vec![
            format!("view_kind={}", self.view_kind.label()),
            format!("tree_semantics={}", self.tree_semantics.label()),
            format!("collection_basis={}", self.collection_basis.label()),
            String::from("subject_semantics=collection_unit"),
        ];
        if let Some(trace_id) = self.trace_id.or(fallback_trace_id) {
            lines.push(format!("trace_id={trace_id}"));
        }
        if let Some(query_kind) = self.query_kind {
            lines.push(format!("query_kind={}", query_kind.label()));
        }
        if let Some(query_owner) = &self.query_owner {
            lines.push(format!("query_owner={query_owner}"));
        }
        if let Some(query_match) = self.query_match {
            lines.push(format!("query_match={}", query_match.label()));
        }
        if let Some(input_path) = &self.input_path {
            lines.push(format!("input_path={input_path}"));
        }
        if let Some(manifest_path) = &self.manifest_path {
            lines.push(format!("manifest_path={manifest_path}"));
        }
        lines.push(format!(
            "subject_filter={}",
            self.filters.subject.as_deref().unwrap_or("<none>")
        ));
        lines.push(format!(
            "result_filter={}",
            self.filters.result.map_or("<none>", |result| result.label())
        ));
        lines.push(format!(
            "candidate_kind_filter={}",
            self.filters.candidate_kind.as_deref().unwrap_or("<none>")
        ));
        lines.push(format!(
            "max_depth_filter={}",
            self.filters
                .max_depth
                .map_or_else(|| String::from("<none>"), |value| value.to_string())
        ));
        lines.push(String::new());
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::{SolveCollectionBasis, SolveProvenance, SolveTreeSemantics, SolveViewKind};
    use crate::{QueryMatchKind, QueryTargetKind, SolveFilters, SolveResultFilter};

    #[test]
    fn renders_stable_header_lines() {
        let lines = SolveProvenance {
            view_kind: SolveViewKind::Analysis,
            tree_semantics: SolveTreeSemantics::RustcProofTree,
            collection_basis: SolveCollectionBasis::OwnerExplicitPredicates,
            trace_id: Some(1),
            query_kind: Some(QueryTargetKind::ImplOwner),
            query_owner: Some(String::from("array::Get")),
            query_match: Some(QueryMatchKind::Exact),
            manifest_path: Some(String::from("crates/typelude-col/Cargo.toml")),
            filters: SolveFilters {
                result: Some(SolveResultFilter::Unsupported),
                ..SolveFilters::default()
            },
            ..SolveProvenance::default()
        }
        .header_lines(None);

        assert!(lines.contains(&String::from("view_kind=solve_analysis")));
        assert!(lines.contains(&String::from("query_kind=impl")));
        assert!(lines.contains(&String::from("query_match=exact")));
        assert!(lines.contains(&String::from("result_filter=unsupported")));
    }
}
