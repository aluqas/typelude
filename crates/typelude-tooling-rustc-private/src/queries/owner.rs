use rustc_hir::ItemKind;
use rustc_span::def_id::LocalDefId;

use crate::{
    error::{AnalysisError, AnalysisResult},
    queries::{Query, QueryContext, run_owner_predicates},
    subjects::{ResolvedSubject, subject_for_owner},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryTargetKind {
    AnyOwner,
    ImplOwner,
    AssocItemOwner,
}

impl QueryTargetKind {
    pub fn from_env(value: Option<&str>) -> Self {
        match value.unwrap_or("owner") {
            "impl" => Self::ImplOwner,
            "assoc_item" => Self::AssocItemOwner,
            _ => Self::AnyOwner,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryMatchKind {
    Substring,
    Suffix,
    Exact,
    DefId,
}

impl QueryMatchKind {
    pub fn from_env(value: Option<&str>) -> Self {
        match value.unwrap_or("suffix") {
            "substring" => Self::Substring,
            "exact" => Self::Exact,
            "def_id" => Self::DefId,
            _ => Self::Suffix,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolveOwnerQuery {
    pub owner: String,
    pub target_kind: QueryTargetKind,
    pub match_kind: QueryMatchKind,
}

impl<'tcx> Query<'tcx> for ResolveOwnerQuery {
    fn run(&self, context: &mut QueryContext<'_, '_, 'tcx>) -> AnalysisResult<()> {
        let session = context.session_mut();
        let crate_items = session.tcx.hir_crate_items(());
        let mut matched = Vec::<String>::new();

        for item_id in crate_items.free_items() {
            if !session.can_emit() {
                session.record_drop();
                break;
            }

            let item = session.tcx.hir_item(item_id);
            let def_id = item_id.owner_id.def_id;
            let owner_path = session.tcx.def_path_str(def_id);
            let owner_subject = subject_for_owner(session.tcx, def_id);

            if owner_matches(def_id, &owner_path, &self.owner, self.match_kind)
                && matches_owner_kind(item.kind, self.target_kind)
            {
                matched.push(owner_path.clone());
                run_owner_predicates(session, owner_subject, true, def_id)?;
            }

            if let ItemKind::Impl(impl_data) = item.kind {
                for impl_item_id in impl_data.items {
                    if !session.can_emit() {
                        session.record_drop();
                        break;
                    }
                    let assoc_def_id = impl_item_id.owner_id.def_id;
                    let assoc_path = session.tcx.def_path_str(assoc_def_id);
                    if owner_matches(assoc_def_id, &assoc_path, &self.owner, self.match_kind)
                        && self.target_kind == QueryTargetKind::AssocItemOwner
                    {
                        matched.push(assoc_path.clone());
                        run_owner_predicates(
                            session,
                            ResolvedSubject::AssocItem(assoc_def_id),
                            true,
                            assoc_def_id,
                        )?;
                    }
                }
            }
        }

        if !matched.is_empty() {
            Ok(())
        } else {
            Err(AnalysisError::new(format!(
                "no owner matched query {match_kind:?}: {}",
                self.owner,
                match_kind = self.match_kind
            )))
        }
    }
}

fn owner_matches(
    def_id: LocalDefId,
    owner_path: &str,
    query: &str,
    match_kind: QueryMatchKind,
) -> bool {
    match match_kind {
        QueryMatchKind::Substring => owner_path.contains(query),
        QueryMatchKind::Suffix => {
            owner_path == query || owner_path.ends_with(&format!("::{query}"))
        },
        QueryMatchKind::Exact => owner_path == query,
        QueryMatchKind::DefId => {
            query.parse::<u32>().ok().is_some_and(|index| def_id.local_def_index.as_u32() == index)
        },
    }
}

fn matches_owner_kind(kind: ItemKind<'_>, target_kind: QueryTargetKind) -> bool {
    match target_kind {
        QueryTargetKind::AnyOwner => true,
        QueryTargetKind::ImplOwner => matches!(kind, ItemKind::Impl(..)),
        QueryTargetKind::AssocItemOwner => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{QueryMatchKind, QueryTargetKind};

    #[test]
    fn parses_query_target_from_env() {
        assert_eq!(QueryTargetKind::from_env(Some("owner")), QueryTargetKind::AnyOwner);
        assert_eq!(QueryTargetKind::from_env(Some("impl")), QueryTargetKind::ImplOwner);
        assert_eq!(QueryTargetKind::from_env(Some("assoc_item")), QueryTargetKind::AssocItemOwner);
    }

    #[test]
    fn parses_query_match_from_env() {
        assert_eq!(QueryMatchKind::from_env(Some("substring")), QueryMatchKind::Substring);
        assert_eq!(QueryMatchKind::from_env(Some("suffix")), QueryMatchKind::Suffix);
        assert_eq!(QueryMatchKind::from_env(Some("exact")), QueryMatchKind::Exact);
        assert_eq!(QueryMatchKind::from_env(Some("def_id")), QueryMatchKind::DefId);
    }
}
