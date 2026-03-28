mod diagnostics;
mod item_structure;
mod trait_solve;

use typelude_tooling_core::HookId;

use crate::{error::AnalysisResult, session::AnalysisSession};

pub use diagnostics::{DiagnosticsHook, emit_diagnostics_notice};
pub use item_structure::ItemStructureHook;
pub use trait_solve::{SweepExplicitPredicatesHook, TraitSolveHook, solve_explicit_predicate};

pub trait Hook<'tcx> {
    fn run(&self, session: &mut AnalysisSession<'_, 'tcx>) -> AnalysisResult<()>;
}

pub struct HookRegistry;

impl HookRegistry {
    pub fn run<'tcx>(
        hook_id: HookId,
        session: &mut AnalysisSession<'_, 'tcx>,
    ) -> AnalysisResult<()> {
        match hook_id {
            HookId::ItemStructure => ItemStructureHook.run(session),
            HookId::TraitSolve => TraitSolveHook.run(session),
            HookId::Diagnostics => DiagnosticsHook.run(session),
        }
    }
}
