use typelude_tooling_core::{DiagnosticEmitted, DiagnosticRecord, HookId, TracePayload};

use crate::{error::AnalysisResult, hooks::Hook, session::AnalysisSession};

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
    session.emitter.write_payload(TracePayload::DiagnosticEmitted(DiagnosticEmitted {
        hook_id: Some(HookId::Diagnostics),
        diagnostic_id: diag_id,
        subject_id: None,
        goal_id: None,
        record: DiagnosticRecord::tooling_notice(
            "compiler diagnostics remain available through typelude-tooling-rustc artifacts",
        ),
    }));
}
