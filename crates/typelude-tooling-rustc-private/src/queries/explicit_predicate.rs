use typelude_tooling_core::SubjectId;

use crate::{
    error::AnalysisResult,
    queries::{Query, QueryContext},
    subjects::ResolvedSubject,
};

#[derive(Debug, Clone, Copy)]
pub struct SolveExplicitPredicateQuery<'tcx> {
    pub subject: ResolvedSubject<'tcx>,
    pub subject_id: SubjectId,
    pub runner: fn(
        &mut crate::session::AnalysisSession<'_, 'tcx>,
        ResolvedSubject<'tcx>,
        SubjectId,
    ) -> AnalysisResult<()>,
}

impl<'tcx> Query<'tcx> for SolveExplicitPredicateQuery<'tcx> {
    fn run(&self, context: &mut QueryContext<'_, '_, 'tcx>) -> AnalysisResult<()> {
        (self.runner)(context.session_mut(), self.subject, self.subject_id)
    }
}
