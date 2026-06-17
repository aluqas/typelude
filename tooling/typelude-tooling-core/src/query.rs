use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryTargetKind {
    AnyOwner,
    ImplOwner,
    AssocItemOwner,
}

impl QueryTargetKind {
    #[must_use]
    pub fn from_label(value: Option<&str>) -> Self {
        match value.unwrap_or("owner") {
            "impl" => Self::ImplOwner,
            "assoc_item" => Self::AssocItemOwner,
            _ => Self::AnyOwner,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::AnyOwner => "owner",
            Self::ImplOwner => "impl",
            Self::AssocItemOwner => "assoc_item",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryMatchKind {
    Substring,
    Suffix,
    Exact,
    DefId,
}

impl QueryMatchKind {
    #[must_use]
    pub fn from_label(value: Option<&str>) -> Self {
        match value.unwrap_or("exact") {
            "substring" => Self::Substring,
            "exact" => Self::Exact,
            "def_id" => Self::DefId,
            _ => Self::Suffix,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Substring => "substring",
            Self::Suffix => "suffix",
            Self::Exact => "exact",
            Self::DefId => "def_id",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerQuerySpec {
    pub owner: String,
    pub target_kind: QueryTargetKind,
    pub match_kind: QueryMatchKind,
}

impl OwnerQuerySpec {
    #[must_use]
    pub fn from_env() -> Option<Self> {
        Some(Self {
            owner: std::env::var("TYPELUDE_TOOLING_QUERY_OWNER").ok()?,
            target_kind: QueryTargetKind::from_label(
                std::env::var("TYPELUDE_TOOLING_QUERY_KIND").ok().as_deref(),
            ),
            match_kind: QueryMatchKind::from_label(
                std::env::var("TYPELUDE_TOOLING_QUERY_MATCH").ok().as_deref(),
            ),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionTreeQuerySpec {
    pub owner: String,
    pub match_kind: QueryMatchKind,
    pub max_depth: usize,
}

impl DefinitionTreeQuerySpec {
    #[must_use]
    pub fn from_env() -> Option<Self> {
        Some(Self {
            owner: std::env::var("TYPELUDE_TOOLING_DEF_TREE_OWNER").ok()?,
            match_kind: QueryMatchKind::from_label(
                std::env::var("TYPELUDE_TOOLING_DEF_TREE_MATCH").ok().as_deref(),
            ),
            max_depth: std::env::var("TYPELUDE_TOOLING_DEF_TREE_MAX_DEPTH")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(32),
        })
    }
}

#[must_use]
pub fn owner_path_matches(owner_path: &str, query: &str, match_kind: QueryMatchKind) -> bool {
    match match_kind {
        QueryMatchKind::Substring => owner_path.contains(query),
        QueryMatchKind::Suffix => {
            owner_path == query || owner_path.ends_with(&format!("::{query}"))
        },
        QueryMatchKind::Exact => owner_path == query,
        QueryMatchKind::DefId => false,
    }
}

#[must_use]
pub fn def_index_matches(local_def_index: u32, query: &str) -> bool {
    query.parse::<u32>().ok().is_some_and(|index| index == local_def_index)
}

#[cfg(test)]
mod tests {
    use super::{QueryMatchKind, QueryTargetKind, def_index_matches, owner_path_matches};

    #[test]
    fn parses_query_target_from_labels() {
        assert_eq!(QueryTargetKind::from_label(Some("owner")), QueryTargetKind::AnyOwner);
        assert_eq!(QueryTargetKind::from_label(Some("impl")), QueryTargetKind::ImplOwner);
        assert_eq!(
            QueryTargetKind::from_label(Some("assoc_item")),
            QueryTargetKind::AssocItemOwner
        );
    }

    #[test]
    fn parses_query_match_from_labels() {
        assert_eq!(QueryMatchKind::from_label(Some("substring")), QueryMatchKind::Substring);
        assert_eq!(QueryMatchKind::from_label(Some("suffix")), QueryMatchKind::Suffix);
        assert_eq!(QueryMatchKind::from_label(Some("exact")), QueryMatchKind::Exact);
        assert_eq!(QueryMatchKind::from_label(Some("def_id")), QueryMatchKind::DefId);
    }

    #[test]
    fn matches_owner_path_using_string_policies() {
        let path = "foo::bar::Baz";
        assert!(owner_path_matches(path, "bar::Baz", QueryMatchKind::Suffix));
        assert!(owner_path_matches(path, "foo::bar::Baz", QueryMatchKind::Exact));
        assert!(owner_path_matches(path, "bar", QueryMatchKind::Substring));
        assert!(!owner_path_matches(path, "123", QueryMatchKind::DefId));
    }

    #[test]
    fn matches_def_index_textually() {
        assert!(def_index_matches(12, "12"));
        assert!(!def_index_matches(12, "13"));
    }
}
