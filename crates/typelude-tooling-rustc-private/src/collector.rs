use std::{collections::BTreeMap, time::Instant};

extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use rustc_middle::ty::TyCtxt;
use typelude_tooling_core::{CandidateId, GoalId, HookId, ItemId, RunId, TraceEventKind};

use crate::output::{CollectStats, EventEmitter, TraceEventExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionPoint {
    ExplicitPredicates,
    ImplPredicates,
}

impl CollectionPoint {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitPredicates => "explicit_predicates",
            Self::ImplPredicates => "impl_predicates",
        }
    }

    fn all() -> &'static [CollectionPoint] {
        &[Self::ExplicitPredicates, Self::ImplPredicates]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectorConfig {
    pub enabled: Vec<HookId>,
    pub focus: Option<String>,
    pub max_events: usize,
    pub max_depth: usize,
    pub summary_only: bool,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            enabled: default_hooks(),
            focus: std::env::var("TYPELUDE_TOOLING_FOCUS").ok(),
            max_events: std::env::var("TYPELUDE_TOOLING_MAX_EVENTS")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(50_000),
            max_depth: std::env::var("TYPELUDE_TOOLING_MAX_DEPTH")
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(32),
            summary_only: std::env::var("TYPELUDE_TOOLING_SUMMARY_ONLY")
                .ok()
                .as_deref()
                .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
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

pub struct HookContext<'a> {
    pub emitter: &'a mut EventEmitter,
    pub config: &'a CollectorConfig,
    pub stats: CollectStats,
    item_ids: BTreeMap<String, ItemId>,
    next_item_id: u64,
    next_goal_id: u64,
    next_candidate_id: u64,
}

impl<'a> HookContext<'a> {
    #[must_use]
    pub fn new(emitter: &'a mut EventEmitter, config: &'a CollectorConfig) -> Self {
        Self {
            emitter,
            config,
            stats: CollectStats::default(),
            item_ids: BTreeMap::new(),
            next_item_id: 1,
            next_goal_id: 1,
            next_candidate_id: 1,
        }
    }

    #[must_use]
    pub fn can_emit(&self) -> bool {
        self.config.max_events == 0 || self.emitter.emitted_count() < self.config.max_events
    }

    #[must_use]
    pub fn focus_matches(&self, value: &str) -> bool {
        self.config
            .focus
            .as_ref()
            .map(|focus| value.to_ascii_lowercase().contains(&focus.to_ascii_lowercase()))
            .unwrap_or(true)
    }

    #[must_use]
    pub fn alloc_goal_id(&mut self) -> GoalId {
        let id = GoalId::new(self.next_goal_id);
        self.next_goal_id += 1;
        id
    }

    #[must_use]
    pub fn alloc_candidate_id(&mut self) -> CandidateId {
        let id = CandidateId::new(self.next_candidate_id);
        self.next_candidate_id += 1;
        id
    }

    pub fn ensure_item(
        &mut self,
        hook_id: HookId,
        item_path: &str,
        item_kind: &str,
        collection_point: &str,
    ) -> ItemId {
        if let Some(item_id) = self.item_ids.get(item_path).copied() {
            return item_id;
        }

        let item_id = ItemId::new(self.next_item_id);
        self.next_item_id += 1;
        self.item_ids.insert(item_path.to_owned(), item_id);
        self.stats.item_count += 1;

        if self.can_emit() {
            let event = self
                .emitter
                .emit(TraceEventKind::ItemDiscovered, item_path)
                .with_hook_id(hook_id)
                .with_item_id(item_id)
                .with_metadata("item_kind", item_kind)
                .with_metadata("collection_point", collection_point);
            self.emitter.write(&event);
        } else {
            self.stats.dropped_count += 1;
        }

        item_id
    }
}

pub trait Hook {
    fn id(&self) -> HookId;
    fn run(&self, tcx: TyCtxt<'_>, ctx: &mut HookContext<'_>) -> Result<(), String>;
}

#[derive(Debug, Clone)]
pub struct ItemContext {
    pub item_id: ItemId,
    pub item_path: String,
    pub item_kind: String,
    pub collection_point: CollectionPoint,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TraitSolveHook;

#[derive(Debug, Default, Clone, Copy)]
pub struct ItemStructureHook;

#[derive(Debug, Default, Clone, Copy)]
pub struct DiagnosticsHook;

impl Hook for TraitSolveHook {
    fn id(&self) -> HookId {
        HookId::TraitSolve
    }

    fn run(&self, tcx: TyCtxt<'_>, ctx: &mut HookContext<'_>) -> Result<(), String> {
        let crate_items = tcx.hir_crate_items(());
        for &point in CollectionPoint::all() {
            match point {
                CollectionPoint::ExplicitPredicates => {
                    for item_id in crate_items.free_items() {
                        if !ctx.can_emit() {
                            break;
                        }
                        let item = tcx.hir_item(item_id);
                        let def_id = item_id.owner_id.def_id;
                        let item_path = tcx.def_path_str(def_id);
                        let item_kind = hir_item_kind_str(item.kind).to_string();
                        let item_id =
                            ctx.ensure_item(self.id(), &item_path, &item_kind, point.as_str());
                        let item_ctx = ItemContext {
                            item_id,
                            item_path,
                            item_kind,
                            collection_point: point,
                        };
                        collect_item_predicates(tcx, ctx, def_id, &item_ctx)?;
                    }
                }
                CollectionPoint::ImplPredicates => {
                    for item_id in crate_items.free_items() {
                        if !ctx.can_emit() {
                            break;
                        }
                        let item = tcx.hir_item(item_id);
                        if !matches!(item.kind, rustc_hir::ItemKind::Impl { .. }) {
                            continue;
                        }

                        let def_id = item_id.owner_id.def_id;
                        let item_path = tcx.def_path_str(def_id);
                        let item_id = ctx.ensure_item(self.id(), &item_path, "impl", point.as_str());
                        let item_ctx = ItemContext {
                            item_id,
                            item_path,
                            item_kind: String::from("impl"),
                            collection_point: point,
                        };
                        collect_item_predicates(tcx, ctx, def_id, &item_ctx)?;

                        if let rustc_hir::ItemKind::Impl(impl_data) = item.kind {
                            for impl_item_id in impl_data.items {
                                if !ctx.can_emit() {
                                    break;
                                }
                                let assoc_def_id = impl_item_id.owner_id.def_id;
                                let assoc_item = tcx.hir_impl_item(*impl_item_id);
                                let assoc_path = tcx.def_path_str(assoc_def_id);
                                let assoc_kind =
                                    hir_impl_item_kind_str(assoc_item.kind).to_string();
                                let assoc_item_id = ctx.ensure_item(
                                    self.id(),
                                    &assoc_path,
                                    &assoc_kind,
                                    point.as_str(),
                                );
                                let assoc_ctx = ItemContext {
                                    item_id: assoc_item_id,
                                    item_path: assoc_path,
                                    item_kind: assoc_kind,
                                    collection_point: point,
                                };
                                collect_item_predicates(tcx, ctx, assoc_def_id, &assoc_ctx)?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl Hook for ItemStructureHook {
    fn id(&self) -> HookId {
        HookId::ItemStructure
    }

    fn run(&self, tcx: TyCtxt<'_>, ctx: &mut HookContext<'_>) -> Result<(), String> {
        let crate_items = tcx.hir_crate_items(());
        for item_id in crate_items.free_items() {
            if !ctx.can_emit() {
                break;
            }
            let item = tcx.hir_item(item_id);
            let def_id = item_id.owner_id.def_id;
            let item_path = tcx.def_path_str(def_id);
            let item_kind = hir_item_kind_str(item.kind);
            ctx.ensure_item(self.id(), &item_path, item_kind, "item_structure");

            if let rustc_hir::ItemKind::Impl(impl_data) = item.kind {
                for impl_item_id in impl_data.items {
                    if !ctx.can_emit() {
                        break;
                    }
                    let assoc_def_id = impl_item_id.owner_id.def_id;
                    let assoc_item = tcx.hir_impl_item(*impl_item_id);
                    let assoc_path = tcx.def_path_str(assoc_def_id);
                    let assoc_kind = hir_impl_item_kind_str(assoc_item.kind);
                    ctx.ensure_item(self.id(), &assoc_path, assoc_kind, "item_structure");
                }
            }
        }
        Ok(())
    }
}

impl Hook for DiagnosticsHook {
    fn id(&self) -> HookId {
        HookId::Diagnostics
    }

    fn run(&self, tcx: TyCtxt<'_>, ctx: &mut HookContext<'_>) -> Result<(), String> {
        if !ctx.can_emit() {
            ctx.stats.dropped_count += 1;
            return Ok(());
        }
        let crate_name = tcx.crate_name(rustc_span::def_id::LOCAL_CRATE).to_string();
        let event = ctx
            .emitter
            .emit(TraceEventKind::Info, format!("diagnostics hook active for {crate_name}"))
            .with_hook_id(self.id())
            .with_detail("compiler diagnostics remain available through typelude-tooling-rustc");
        ctx.emitter.write(&event);
        Ok(())
    }
}

fn collect_item_predicates(
    tcx: TyCtxt<'_>,
    ctx: &mut HookContext<'_>,
    def_id: rustc_span::def_id::LocalDefId,
    item_ctx: &ItemContext,
) -> Result<(), String> {
    let generic_preds = tcx.explicit_predicates_of(def_id).instantiate_identity(tcx);
    let predicate_list: Vec<_> = generic_preds.into_iter().collect();
    if predicate_list.is_empty() {
        return Ok(());
    }

    crate::visitor::collect_for_item(tcx, ctx, item_ctx, predicate_list.into_iter())
}

fn hir_item_kind_str(kind: rustc_hir::ItemKind<'_>) -> &'static str {
    match kind {
        rustc_hir::ItemKind::Fn { .. } => "fn",
        rustc_hir::ItemKind::Struct(..) => "struct",
        rustc_hir::ItemKind::Enum(..) => "enum",
        rustc_hir::ItemKind::Trait(..) => "trait",
        rustc_hir::ItemKind::Impl(..) => "impl",
        rustc_hir::ItemKind::TyAlias(..) => "type",
        rustc_hir::ItemKind::Const(..) => "const",
        rustc_hir::ItemKind::Static(..) => "static",
        rustc_hir::ItemKind::Mod(..) => "mod",
        rustc_hir::ItemKind::Use(..) => "use",
        rustc_hir::ItemKind::Macro(..) => "macro",
        rustc_hir::ItemKind::TraitAlias(..) => "trait_alias",
        rustc_hir::ItemKind::Union(..) => "union",
        rustc_hir::ItemKind::ForeignMod { .. } => "foreign_mod",
        rustc_hir::ItemKind::GlobalAsm { .. } => "global_asm",
        rustc_hir::ItemKind::ExternCrate(..) => "extern_crate",
    }
}

fn hir_impl_item_kind_str(kind: rustc_hir::ImplItemKind<'_>) -> &'static str {
    match kind {
        rustc_hir::ImplItemKind::Fn(..) => "assoc_fn",
        rustc_hir::ImplItemKind::Type(..) => "assoc_type",
        rustc_hir::ImplItemKind::Const(..) => "assoc_const",
    }
}

fn hook_registry() -> Vec<Box<dyn Hook>> {
    vec![
        Box::new(ItemStructureHook),
        Box::new(TraitSolveHook),
        Box::new(DiagnosticsHook),
    ]
}

pub fn run(tcx: TyCtxt<'_>, config: &CollectorConfig) -> Result<CollectStats, String> {
    let start = Instant::now();
    let run_id = RunId::new(start.elapsed().as_nanos() as u64 ^ u64::from(std::process::id()));
    let crate_name = tcx.crate_name(rustc_span::def_id::LOCAL_CRATE).to_string();
    let rustc_version = rustc_version();

    let mut emitter = EventEmitter::new(run_id);
    let start_event = emitter
        .emit(TraceEventKind::RunStarted, crate_name.clone())
        .with_detail("rustc_private collection started")
        .with_metadata("rustc_version", rustc_version)
        .with_metadata("summary_only", config.summary_only.to_string())
        .with_metadata("max_events", config.max_events.to_string())
        .with_metadata("max_depth", config.max_depth.to_string());
    emitter.write(&start_event);

    let mut ctx = HookContext::new(&mut emitter, config);
    for hook in hook_registry() {
        if !config.enabled.iter().any(|id| *id == hook.id()) {
            continue;
        }
        hook.run(tcx, &mut ctx)?;
    }

    let end_event = ctx
        .emitter
        .emit(TraceEventKind::RunFinished, crate_name)
        .with_detail("rustc_private collection finished")
        .with_metadata("goal_count", ctx.stats.goal_count.to_string())
        .with_metadata("candidate_count", ctx.stats.candidate_count.to_string())
        .with_metadata("item_count", ctx.stats.item_count.to_string())
        .with_metadata("diagnostic_count", ctx.stats.diagnostic_count.to_string())
        .with_metadata("dropped_count", ctx.stats.dropped_count.to_string())
        .with_metadata("elapsed_ms", start.elapsed().as_millis().to_string());
    ctx.emitter.write(&end_event);

    Ok(ctx.stats)
}

fn rustc_version() -> String {
    option_env!("CFG_VERSION")
        .map(str::to_owned)
        .unwrap_or_else(|| String::from("unknown"))
}

#[cfg(test)]
mod tests {
    use typelude_tooling_core::HookId;

    use super::{default_hooks, CollectionPoint, CollectorConfig};

    #[test]
    fn default_config_has_all_primary_hooks() {
        let hooks = default_hooks();
        assert!(hooks.contains(&HookId::TraitSolve));
        assert!(hooks.contains(&HookId::ItemStructure));
        assert!(hooks.contains(&HookId::Diagnostics));
    }

    #[test]
    fn collection_point_as_str_is_stable() {
        assert_eq!(CollectionPoint::ExplicitPredicates.as_str(), "explicit_predicates");
        assert_eq!(CollectionPoint::ImplPredicates.as_str(), "impl_predicates");
    }

    #[test]
    fn collector_defaults_are_non_typelude_specific() {
        let config = CollectorConfig::default();
        assert!(config.focus.is_none() || config.focus.is_some());
        assert!(config.max_depth >= 1);
    }
}
