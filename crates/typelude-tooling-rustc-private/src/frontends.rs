use std::time::{SystemTime, UNIX_EPOCH};

use rustc_middle::ty::TyCtxt;
use typelude_tooling_core::{HookId, RunId, TraceEventKind};

use crate::{
    emit::{CollectStats, EventEmitter, TraceEventExt},
    error::AnalysisResult,
    hooks::HookRegistry,
    queries::{Query, QueryContext, QueryMatchKind, QueryTargetKind, ResolveOwnerQuery},
    session::{AnalysisConfig, AnalysisSession},
};

pub type CollectConfig = AnalysisConfig;

impl Default for CollectConfig {
    fn default() -> Self {
        Self {
            enabled: default_hooks(),
            focus: std::env::var("TYPELUDE_TOOLING_FOCUS").ok(),
            subject_filter: std::env::var("TYPELUDE_TOOLING_SUBJECT_FILTER").ok(),
            max_events: std::env::var("TYPELUDE_TOOLING_MAX_EVENTS")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(50_000),
            max_depth: std::env::var("TYPELUDE_TOOLING_MAX_DEPTH")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(32),
        }
    }
}

fn default_hooks() -> Vec<HookId> {
    if let Some(raw) = std::env::var("TYPELUDE_TOOLING_HOOKS").ok() {
        let parsed = raw
            .split(',')
            .filter_map(|segment| match segment.trim() {
                "trait_solve" => Some(HookId::TraitSolve),
                "diagnostics" => Some(HookId::Diagnostics),
                "item_structure" => Some(HookId::ItemStructure),
                _ => None,
            })
            .collect::<Vec<_>>();
        if !parsed.is_empty() {
            return parsed;
        }
    }
    vec![HookId::ItemStructure, HookId::TraitSolve, HookId::Diagnostics]
}

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
    let start_event = emitter
        .emit(TraceEventKind::RunStarted, crate_name.clone())
        .with_detail("rustc_private analysis session started")
        .with_metadata("rustc_version", rustc_version)
        .with_metadata(
            "subject_filter",
            config.subject_filter.clone().unwrap_or_else(|| String::from("<none>")),
        )
        .with_metadata("max_events", config.max_events.to_string())
        .with_metadata("max_depth", config.max_depth.to_string());
    emitter.write(&start_event);

    let mut session = AnalysisSession::new(tcx, &mut emitter, config);
    run(&mut session)?;

    let end_event = session
        .emitter
        .emit(TraceEventKind::RunFinished, crate_name)
        .with_detail("rustc_private analysis session finished")
        .with_metadata("goal_count", session.stats.goal_count.to_string())
        .with_metadata("candidate_count", session.stats.candidate_count.to_string())
        .with_metadata("subject_count", session.stats.subject_count.to_string())
        .with_metadata("diagnostic_count", session.stats.diagnostic_count.to_string())
        .with_metadata("dropped_count", session.stats.dropped_count.to_string());
    session.emitter.write(&end_event);

    Ok(session.stats)
}
fn rustc_version() -> String {
    option_env!("CFG_VERSION").map(str::to_owned).unwrap_or_else(|| String::from("unknown"))
}

#[cfg(test)]
mod tests {
    use typelude_tooling_core::HookId;

    use super::default_hooks;

    #[test]
    fn default_config_has_all_primary_hooks() {
        let hooks = default_hooks();
        assert!(hooks.contains(&HookId::TraitSolve));
        assert!(hooks.contains(&HookId::ItemStructure));
        assert!(hooks.contains(&HookId::Diagnostics));
    }
}
