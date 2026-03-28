use crate::{
    error::{AnalysisError, AnalysisResult},
    queries::{Query, QueryContext, run_owner_predicates},
    subjects::{ResolvedSubject, subject_for_owner},
};

#[derive(Debug, Clone)]
pub struct ResolveOwnerQuery {
    pub owner: String,
}

impl<'tcx> Query<'tcx> for ResolveOwnerQuery {
    fn run(&self, context: &mut QueryContext<'_, '_, 'tcx>) -> AnalysisResult<()> {
        let session = context.session_mut();
        let crate_items = session.tcx.hir_crate_items(());
        let mut matched = false;

        for item_id in crate_items.free_items() {
            if !session.can_emit() {
                session.record_drop();
                break;
            }

            let item = session.tcx.hir_item(item_id);
            let def_id = item_id.owner_id.def_id;
            let owner_path = session.tcx.def_path_str(def_id);
            let owner_subject = subject_for_owner(session.tcx, def_id);

            if owner_path.contains(&self.owner) {
                matched = true;
                run_owner_predicates(session, owner_subject, true, def_id)?;
            }

            if let rustc_hir::ItemKind::Impl(impl_data) = item.kind {
                for impl_item_id in impl_data.items {
                    if !session.can_emit() {
                        session.record_drop();
                        break;
                    }
                    let assoc_def_id = impl_item_id.owner_id.def_id;
                    let assoc_path = session.tcx.def_path_str(assoc_def_id);
                    if assoc_path.contains(&self.owner) {
                        matched = true;
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

        if matched {
            Ok(())
        } else {
            Err(AnalysisError::new(format!("no owner matched query substring: {}", self.owner)))
        }
    }
}
