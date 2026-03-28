use typelude_tooling_core::HookId;

use crate::{
    error::AnalysisResult, hooks::Hook, session::AnalysisSession, subjects::ResolvedSubject,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct ItemStructureHook;

impl<'tcx> Hook<'tcx> for ItemStructureHook {
    fn run(&self, session: &mut AnalysisSession<'_, 'tcx>) -> AnalysisResult<()> {
        let crate_items = session.tcx.hir_crate_items(());
        for item_id in crate_items.free_items() {
            if !session.can_emit() {
                session.record_drop();
                break;
            }

            let item = session.tcx.hir_item(item_id);
            let def_id = item_id.owner_id.def_id;
            let subject = match item.kind {
                rustc_hir::ItemKind::Impl(..) => ResolvedSubject::Impl(def_id),
                _ => ResolvedSubject::Item(def_id),
            };
            if session.subject_matches(&subject.label(session.tcx), &subject.metadata(session.tcx))
            {
                session.ensure_subject(HookId::ItemStructure, subject);
            }

            if let rustc_hir::ItemKind::Impl(impl_data) = item.kind {
                for impl_item_id in impl_data.items {
                    if !session.can_emit() {
                        session.record_drop();
                        break;
                    }
                    let subject = ResolvedSubject::AssocItem(impl_item_id.owner_id.def_id);
                    if session.subject_matches(
                        &subject.label(session.tcx),
                        &subject.metadata(session.tcx),
                    ) {
                        session.ensure_subject(HookId::ItemStructure, subject);
                    }
                }
            }
        }
        Ok(())
    }
}
