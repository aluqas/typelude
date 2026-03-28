mod explicit_predicate;

use crate::{error::AnalysisResult, session::AnalysisSession};

pub use explicit_predicate::SolveExplicitPredicateQuery;

pub struct QueryContext<'a, 's, 'tcx> {
    session: &'a mut AnalysisSession<'s, 'tcx>,
}

impl<'a, 's, 'tcx> QueryContext<'a, 's, 'tcx> {
    #[must_use]
    pub fn new(session: &'a mut AnalysisSession<'s, 'tcx>) -> Self {
        Self {
            session,
        }
    }

    pub fn session_mut(&mut self) -> &mut AnalysisSession<'s, 'tcx> {
        self.session
    }
}

pub trait Query<'tcx> {
    fn run(&self, context: &mut QueryContext<'_, '_, 'tcx>) -> AnalysisResult<()>;
}
