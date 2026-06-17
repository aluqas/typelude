use std::time::{SystemTime, UNIX_EPOCH};

use rustc_middle::ty::TyCtxt;
use typelude_tooling_core::{AnalysisConfig, RunFinished, RunId, RunStarted, TracePayload};

use crate::{
    emit::{CollectStats, EventEmitter},
    error::AnalysisResult,
    hooks::HookRegistry,
    queries::{
        DefinitionTreeQuery, Query, QueryContext, QueryMatchKind, QueryTargetKind,
        ResolveOwnerQuery,
    },
    session::AnalysisSession,
};

pub type CollectConfig = AnalysisConfig;

pub fn run_collect_frontend(
    tcx: TyCtxt<'_>,
    config: &CollectConfig,
) -> AnalysisResult<CollectStats> {
    run_with_session(tcx, config, |session| {
        for hook_id in &config.enabled {
            HookRegistry::run(*hook_id, session)?;
        }
        Ok(())
    })
}

pub fn run_owner_query_frontend(
    tcx: TyCtxt<'_>,
    config: &CollectConfig,
    owner: &str,
    target_kind: QueryTargetKind,
    match_kind: QueryMatchKind,
) -> AnalysisResult<CollectStats> {
    run_with_session(tcx, config, |session| {
        let query = ResolveOwnerQuery {
            owner: owner.to_owned(),
            target_kind,
            match_kind,
        };
        let mut context = QueryContext::new(session);
        query.run(&mut context)
    })
}

pub fn run_definition_tree_frontend(
    tcx: TyCtxt<'_>,
    config: &CollectConfig,
    owner: &str,
    match_kind: QueryMatchKind,
    max_depth: usize,
) -> AnalysisResult<CollectStats> {
    let mut config = config.clone();
    config.max_depth = max_depth;
    run_with_session(tcx, &config, |session| {
        let query = DefinitionTreeQuery {
            owner: owner.to_owned(),
            match_kind,
            max_depth,
        };
        let mut context = QueryContext::new(session);
        query.run(&mut context)
    })
}

fn run_with_session(
    tcx: TyCtxt<'_>,
    config: &CollectConfig,
    run: impl for<'s, 'tcx> FnOnce(&mut AnalysisSession<'s, 'tcx>) -> AnalysisResult<()>,
) -> AnalysisResult<CollectStats> {
    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0);
    let run_id = RunId::new(start ^ u64::from(std::process::id()));
    let crate_name = tcx.crate_name(rustc_span::def_id::LOCAL_CRATE).to_string();
    let rustc_version = rustc_version();

    let mut emitter = EventEmitter::new(run_id);
    emitter.write_payload(TracePayload::RunStarted(RunStarted {
        crate_name: crate_name.clone(),
        rustc_version,
        subject_filter: config.subject_filter.clone(),
        max_events: config.max_events,
        max_depth: config.max_depth,
    }));

    let mut session = AnalysisSession::new(tcx, &mut emitter, config);
    run(&mut session)?;

    session.emitter.write_payload(TracePayload::RunFinished(RunFinished {
        crate_name,
        goal_count: session.stats.goal_count,
        candidate_count: session.stats.candidate_count,
        subject_count: session.stats.subject_count,
        diagnostic_count: session.stats.diagnostic_count,
        dropped_count: session.stats.dropped_count,
    }));

    Ok(session.stats)
}
fn rustc_version() -> String {
    option_env!("CFG_VERSION").map(str::to_owned).unwrap_or_else(|| String::from("unknown"))
}
