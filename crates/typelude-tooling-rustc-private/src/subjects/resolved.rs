use std::collections::BTreeMap;

use rustc_hir::def::DefKind;
use rustc_middle::ty::{Clause, TyCtxt};
use rustc_span::def_id::LocalDefId;
use typelude_tooling_core::SubjectKind;

#[derive(Debug, Clone, Copy)]
pub enum ResolvedSubject<'tcx> {
    Item(LocalDefId),
    Impl(LocalDefId),
    AssocItem(LocalDefId),
    ExplicitPredicate {
        owner: LocalDefId,
        index: usize,
        clause: Clause<'tcx>,
    },
}

impl<'tcx> ResolvedSubject<'tcx> {
    #[must_use]
    pub const fn kind(self) -> SubjectKind {
        match self {
            Self::Item(..) => SubjectKind::Item,
            Self::Impl(..) => SubjectKind::Impl,
            Self::AssocItem(..) => SubjectKind::AssocItem,
            Self::ExplicitPredicate {
                ..
            } => SubjectKind::Predicate,
        }
    }

    #[must_use]
    pub fn label(self, tcx: TyCtxt<'tcx>) -> String {
        match self {
            Self::Item(def_id) | Self::Impl(def_id) | Self::AssocItem(def_id) => {
                tcx.def_path_str(def_id)
            },
            Self::ExplicitPredicate {
                clause,
                ..
            } => format!("{clause:?}"),
        }
    }

    #[must_use]
    pub fn key(self, tcx: TyCtxt<'tcx>) -> String {
        match self {
            Self::Item(def_id) => format!("item:{}", tcx.def_path_str(def_id)),
            Self::Impl(def_id) => format!("impl:{}", tcx.def_path_str(def_id)),
            Self::AssocItem(def_id) => format!("assoc:{}", tcx.def_path_str(def_id)),
            Self::ExplicitPredicate {
                owner,
                index,
                clause,
            } => format!("predicate:{}:{index}:{clause:?}", tcx.def_path_str(owner)),
        }
    }

    #[must_use]
    pub fn metadata(self, tcx: TyCtxt<'tcx>) -> BTreeMap<String, String> {
        let mut metadata = BTreeMap::new();
        match self {
            Self::Item(def_id) | Self::Impl(def_id) | Self::AssocItem(def_id) => {
                metadata.insert(String::from("path"), tcx.def_path_str(def_id));
                metadata.insert(String::from("kind"), format!("{:?}", self.kind()));
            },
            Self::ExplicitPredicate {
                owner,
                index,
                clause,
            } => {
                metadata.insert(String::from("owner_path"), tcx.def_path_str(owner));
                metadata.insert(String::from("predicate_index"), index.to_string());
                metadata.insert(String::from("clause"), format!("{clause:?}"));
            },
        }
        metadata
    }

    #[must_use]
    pub fn parent(self, tcx: TyCtxt<'tcx>) -> Option<Self> {
        match self {
            Self::ExplicitPredicate {
                owner,
                ..
            } => Some(subject_for_owner(tcx, owner)),
            Self::Item(..) | Self::Impl(..) | Self::AssocItem(..) => None,
        }
    }
}

#[must_use]
pub fn subject_for_owner<'tcx>(tcx: TyCtxt<'tcx>, def_id: LocalDefId) -> ResolvedSubject<'tcx> {
    match tcx.def_kind(def_id) {
        DefKind::Impl {
            ..
        } => ResolvedSubject::Impl(def_id),
        DefKind::AssocFn
        | DefKind::AssocConst {
            ..
        }
        | DefKind::AssocTy => ResolvedSubject::AssocItem(def_id),
        _ => ResolvedSubject::Item(def_id),
    }
}
