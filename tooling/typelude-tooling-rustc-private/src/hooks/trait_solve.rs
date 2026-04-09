use crate::{
    error::AnalysisResult, hooks::Hook, queries::run_owner_predicates, session::AnalysisSession,
    subjects::ResolvedSubject,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct TraitSolveHook;

pub type SweepExplicitPredicatesHook = TraitSolveHook;

impl<'tcx> Hook<'tcx> for TraitSolveHook {
    fn run(&self, session: &mut AnalysisSession<'_, 'tcx>) -> AnalysisResult<()> {
        let crate_items = session.tcx.hir_crate_items(());
        for item_id in crate_items.free_items() {
            if !session.can_emit() {
                session.record_drop();
                break;
            }

            let item = session.tcx.hir_item(item_id);
            let def_id = item_id.owner_id.def_id;
            let owner_subject = match item.kind {
                rustc_hir::ItemKind::Impl(..) => ResolvedSubject::Impl(def_id),
                _ => ResolvedSubject::Item(def_id),
            };
            let owner_matches = session.subject_matches(
                &owner_subject.label(session.tcx),
                &owner_subject.metadata(session.tcx),
            );
            run_owner_predicates(session, owner_subject, owner_matches, def_id)?;

            if let rustc_hir::ItemKind::Impl(impl_data) = item.kind {
                for impl_item_id in impl_data.items {
                    if !session.can_emit() {
                        session.record_drop();
                        break;
                    }
                    let assoc_def_id = impl_item_id.owner_id.def_id;
                    let assoc_subject = ResolvedSubject::AssocItem(assoc_def_id);
                    let assoc_matches = session.subject_matches(
                        &assoc_subject.label(session.tcx),
                        &assoc_subject.metadata(session.tcx),
                    );
                    run_owner_predicates(session, assoc_subject, assoc_matches, assoc_def_id)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{SweepExplicitPredicatesHook, TraitSolveHook};

    #[test]
    fn sweep_alias_matches_trait_solve_type() {
        let _sweep: SweepExplicitPredicatesHook = TraitSolveHook;
    }
}
