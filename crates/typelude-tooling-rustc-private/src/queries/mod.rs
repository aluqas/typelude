mod explicit_predicate;
mod owner;

pub(crate) use explicit_predicate::run_owner_predicates;
pub use explicit_predicate::{SolveExplicitPredicateQuery, solve_explicit_predicate};
pub use owner::{QueryMatchKind, QueryTargetKind, ResolveOwnerQuery};

use crate::{error::AnalysisResult, session::AnalysisSession};

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
