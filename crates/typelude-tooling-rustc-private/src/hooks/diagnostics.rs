use typelude_tooling_core::{HookId, TraceEventKind};

use crate::{emit::TraceEventExt, error::AnalysisResult, hooks::Hook, session::AnalysisSession};

#[derive(Debug, Default, Clone, Copy)]
pub struct DiagnosticsHook;

impl<'tcx> Hook<'tcx> for DiagnosticsHook {
    fn run(&self, session: &mut AnalysisSession<'_, 'tcx>) -> AnalysisResult<()> {
        emit_diagnostics_notice(session);
        Ok(())
    }
}

pub fn emit_diagnostics_notice(session: &mut AnalysisSession<'_, '_>) {
    if !session.can_emit() {
        session.record_drop();
        return;
    }
    let diag_id = session.alloc_diag_id();
    session.stats.diagnostic_count += 1;
    let event = session
        .emitter
        .emit(TraceEventKind::Info, "diagnostics frontend active")
        .with_hook_id(HookId::Diagnostics)
        .with_diagnostic_id(diag_id)
        .with_detail(
            "compiler diagnostics remain available through typelude-tooling-rustc artifacts",
        );
    session.emitter.write(&event);
}
