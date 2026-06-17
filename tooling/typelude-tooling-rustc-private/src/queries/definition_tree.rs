use std::collections::{BTreeMap, BTreeSet};

use rustc_hir::{GenericParamKind, ImplItemKind, ItemKind};
use rustc_middle::ty::TyCtxt;
use rustc_span::{Span, def_id::LocalDefId};
use typelude_tooling_core::{
    DefinitionEdge, DefinitionEdgeKind, DefinitionGraph, DefinitionGraphEmitted, DefinitionNode,
    DefinitionNodeKind, NodeId, QueryMatchKind, TracePayload, def_index_matches,
    owner_path_matches,
};

use crate::{
    error::{AnalysisError, AnalysisResult},
    queries::{Query, QueryContext},
};

#[derive(Debug, Clone)]
pub struct DefinitionTreeQuery {
    pub owner: String,
    pub match_kind: QueryMatchKind,
    pub max_depth: usize,
}

impl<'tcx> Query<'tcx> for DefinitionTreeQuery {
    fn run(&self, context: &mut QueryContext<'_, '_, 'tcx>) -> AnalysisResult<()> {
        let session = context.session_mut();
        let mut collector = DefinitionCollector::new(session.tcx, self.max_depth);
        collector.index_items();
        let (owner, graph) = collector.collect_owner(&self.owner, self.match_kind)?;
        session.emitter.write_payload(TracePayload::DefinitionGraphEmitted(
            DefinitionGraphEmitted {
                owner,
                graph,
            },
        ));
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct AliasDef {
    name: String,
    def_id: LocalDefId,
    def_path: String,
    generics: Vec<String>,
    source: String,
    rhs: String,
}

#[derive(Debug, Clone)]
struct TraitDef {
    name: String,
    def_path: String,
    source: String,
}

#[derive(Debug, Clone)]
struct AssocTypeDef {
    name: String,
    def_path: String,
    source: String,
    rhs: String,
}

#[derive(Debug, Clone)]
struct ImplDef {
    def_id: LocalDefId,
    def_path: String,
    generics: Vec<String>,
    source: String,
    header: String,
    self_ty: String,
    trait_name: String,
    trait_args: Vec<String>,
    predicates: Vec<String>,
    assoc_types: Vec<AssocTypeDef>,
}

#[derive(Debug, Clone)]
struct ProjectionRef {
    source: String,
    self_ty: String,
    trait_name: String,
    trait_args: Vec<String>,
    assoc_name: String,
    synthetic: bool,
}

#[derive(Debug, Clone)]
struct AliasRef {
    name: String,
    args: Vec<String>,
    source: String,
}

struct DefinitionCollector<'tcx> {
    tcx: TyCtxt<'tcx>,
    max_depth: usize,
    graph: DefinitionGraph,
    next_node_id: u64,
    expanded_keys: BTreeSet<String>,
    aliases: BTreeMap<String, AliasDef>,
    traits: BTreeMap<String, TraitDef>,
    impls: Vec<ImplDef>,
}

impl<'tcx> DefinitionCollector<'tcx> {
    fn new(tcx: TyCtxt<'tcx>, max_depth: usize) -> Self {
        Self {
            tcx,
            max_depth,
            graph: DefinitionGraph::default(),
            next_node_id: 1,
            expanded_keys: BTreeSet::new(),
            aliases: BTreeMap::new(),
            traits: BTreeMap::new(),
            impls: Vec::new(),
        }
    }

    fn index_items(&mut self) {
        let crate_items = self.tcx.hir_crate_items(());
        for item_id in crate_items.free_items() {
            let item = self.tcx.hir_item(item_id);
            let def_id = item_id.owner_id.def_id;
            match item.kind {
                ItemKind::TyAlias(ident, generics, ty) => {
                    let name = ident.name.to_string();
                    self.aliases.insert(
                        name.clone(),
                        AliasDef {
                            name,
                            def_id,
                            def_path: self.tcx.def_path_str(def_id),
                            generics: generic_names(generics),
                            source: self
                                .snippet(item.span)
                                .unwrap_or_else(|| self.tcx.def_path_str(def_id)),
                            rhs: self
                                .snippet(ty.span)
                                .unwrap_or_else(|| format!("{:?}", self.tcx.type_of(def_id))),
                        },
                    );
                },
                ItemKind::Trait {
                    ident,
                    ..
                } => {
                    let name = ident.name.to_string();
                    self.traits.insert(
                        name.clone(),
                        TraitDef {
                            name,
                            def_path: self.tcx.def_path_str(def_id),
                            source: self.snippet(item.span).unwrap_or_default(),
                        },
                    );
                },
                ItemKind::Impl(impl_data) => {
                    if let Some(impl_def) = self.impl_def(def_id, item.span, impl_data) {
                        self.impls.push(impl_def);
                    }
                },
                ItemKind::ExternCrate(..)
                | ItemKind::Use(..)
                | ItemKind::Static(..)
                | ItemKind::Const(..)
                | ItemKind::Fn {
                    ..
                }
                | ItemKind::Macro(..)
                | ItemKind::Mod(..)
                | ItemKind::ForeignMod {
                    ..
                }
                | ItemKind::GlobalAsm {
                    ..
                }
                | ItemKind::Enum(..)
                | ItemKind::Struct(..)
                | ItemKind::Union(..)
                | ItemKind::TraitAlias(..) => {},
            }
        }
    }

    fn collect_owner(
        mut self,
        owner: &str,
        match_kind: QueryMatchKind,
    ) -> AnalysisResult<(String, DefinitionGraph)> {
        let mut matched_owner = None::<String>;
        let mut roots = Vec::new();

        for alias in self.aliases.values().cloned().collect::<Vec<_>>() {
            if !owner_matches(alias.def_id, &alias.def_path, owner, match_kind) {
                continue;
            }
            matched_owner = Some(alias.def_path.clone());
            let root = self.add_alias_node(&alias, &BTreeMap::new());
            self.expand_alias(alias, root, BTreeMap::new(), 0, &mut Vec::new());
            roots.push(root);
        }

        for trait_def in self.traits.values().cloned().collect::<Vec<_>>() {
            if !owner_path_matches(&trait_def.def_path, owner, match_kind) {
                continue;
            }
            matched_owner = Some(trait_def.def_path.clone());
            roots.push(self.add_node(
                DefinitionNodeKind::Trait,
                format!("trait {}", trait_def.name),
                Some(trait_def.def_path),
                Some(trait_def.source),
                BTreeMap::new(),
            ));
        }

        for impl_def in self.impls.clone() {
            if !owner_matches(impl_def.def_id, &impl_def.def_path, owner, match_kind) {
                continue;
            }
            matched_owner = Some(impl_def.def_path.clone());
            let root = self.add_impl_node(&impl_def, BTreeMap::new());
            self.expand_impl(&impl_def, root, &BTreeMap::new(), 0, &mut Vec::new());
            roots.push(root);
        }

        if roots.is_empty() {
            return Err(AnalysisError::new(format!(
                "no owner matched def-tree query {match_kind:?}: {owner}"
            )));
        }

        self.graph.roots = roots;
        Ok((matched_owner.unwrap_or_else(|| owner.to_owned()), self.graph))
    }

    fn impl_def(
        &self,
        def_id: LocalDefId,
        span: Span,
        impl_data: rustc_hir::Impl<'tcx>,
    ) -> Option<ImplDef> {
        let trait_ref = impl_data.of_trait?.trait_ref;
        let trait_source = self.snippet(trait_ref.path.span)?;
        let (trait_name, trait_args) = parse_trait_path(&trait_source);
        let source = self.snippet(span).unwrap_or_else(|| self.tcx.def_path_str(def_id));
        let self_ty = self.snippet(impl_data.self_ty.span).unwrap_or_else(|| String::from("_"));
        let mut assoc_types = Vec::new();
        for impl_item_id in impl_data.items {
            let impl_item = self.tcx.hir_impl_item(*impl_item_id);
            let ImplItemKind::Type(ty) = impl_item.kind else {
                continue;
            };
            assoc_types.push(AssocTypeDef {
                name: impl_item.ident.name.to_string(),
                def_path: self.tcx.def_path_str(impl_item.owner_id.def_id),
                source: self.snippet(impl_item.span).unwrap_or_else(|| {
                    format!(
                        "type {} = {};",
                        impl_item.ident.name,
                        self.snippet(ty.span).unwrap_or_default()
                    )
                }),
                rhs: self.snippet(ty.span).unwrap_or_else(|| format!("{:?}", ty.kind)),
            });
        }
        Some(ImplDef {
            def_id,
            def_path: self.tcx.def_path_str(def_id),
            generics: generic_names(impl_data.generics),
            source,
            header: impl_header(&self.snippet(span).unwrap_or_default(), &trait_source, &self_ty),
            self_ty,
            trait_name,
            trait_args,
            predicates: impl_data
                .generics
                .predicates
                .iter()
                .filter_map(|predicate| self.snippet(predicate.span))
                .filter(|predicate| !predicate.trim_start().starts_with(':'))
                .collect(),
            assoc_types,
        })
    }

    fn expand_alias(
        &mut self,
        alias: AliasDef,
        parent: NodeId,
        env: BTreeMap<String, String>,
        depth: usize,
        stack: &mut Vec<String>,
    ) {
        if self.at_max_depth(parent, depth) {
            return;
        }
        let key = format!("alias:{}:{}", alias.def_path, env_key(&env));
        if self.push_or_cycle(parent, key.clone(), stack) {
            return;
        }
        let source = substitute(&alias.rhs, &env);
        self.expand_source(parent, &source, depth + 1, stack);
        pop_key(stack, &key);
    }

    fn expand_projection(
        &mut self,
        projection: ProjectionRef,
        parent: NodeId,
        depth: usize,
        stack: &mut Vec<String>,
    ) {
        if self.at_max_depth(parent, depth) {
            return;
        }
        let key = format!("projection:{}", compact(&projection.source));
        if self.push_or_cycle(parent, key.clone(), stack) {
            return;
        }

        let matches = self.matching_impls(&projection);
        if matches.is_empty() {
            self.add_projection_leaf(&projection, parent);
            pop_key(stack, &key);
            return;
        }

        for (impl_def, env, confidence) in matches {
            let mut metadata = BTreeMap::new();
            metadata.insert(String::from("match"), confidence);
            metadata.insert(
                String::from("instantiated_trait"),
                format!("{}<{}>", projection.trait_name, projection.trait_args.join(", ")),
            );
            let impl_node = self.add_impl_node(&impl_def, metadata);
            self.add_edge(parent, impl_node, DefinitionEdgeKind::ImplementedBy, "implemented_by");
            self.expand_impl(&impl_def, impl_node, &env, depth + 1, stack);
        }

        pop_key(stack, &key);
    }

    fn expand_impl(
        &mut self,
        impl_def: &ImplDef,
        parent: NodeId,
        env: &BTreeMap<String, String>,
        depth: usize,
        stack: &mut Vec<String>,
    ) {
        if self.at_max_depth(parent, depth) {
            return;
        }
        for predicate in &impl_def.predicates {
            let predicate = substitute(predicate, env);
            let metadata = BTreeMap::from([(String::from("predicate_key"), compact(&predicate))]);
            let node = self.add_node(
                DefinitionNodeKind::WherePredicate,
                predicate.clone(),
                None,
                Some(predicate.clone()),
                metadata,
            );
            self.add_edge(parent, node, DefinitionEdgeKind::HasWherePredicate, "where");
            self.expand_source(node, &predicate, depth + 1, stack);
        }

        for assoc in impl_def.assoc_types.iter().filter(|assoc| assoc.name == "Output") {
            let rhs = substitute(&assoc.rhs, env);
            let source = format!("type {} = {rhs};", assoc.name);
            let metadata = BTreeMap::from([
                (String::from("assoc_key"), compact(&source)),
                (String::from("impl_key"), impl_def.def_path.clone()),
            ]);
            let node = self.add_node(
                DefinitionNodeKind::AssociatedTypeBody,
                source.clone(),
                Some(assoc.def_path.clone()),
                Some(substitute(&assoc.source, env)),
                metadata,
            );
            self.add_edge(parent, node, DefinitionEdgeKind::HasAssociatedTypeBody, "assoc type");
            self.expand_source(node, &rhs, depth + 1, stack);
        }
    }

    fn expand_source(
        &mut self,
        parent: NodeId,
        source: &str,
        depth: usize,
        stack: &mut Vec<String>,
    ) {
        if self.at_max_depth(parent, depth) {
            return;
        }

        let mut seen_projections = BTreeSet::<String>::new();
        for projection in find_projections(source) {
            if !seen_projections.insert(compact(&projection.source)) {
                continue;
            }
            let projection_node = self.add_projection_node(&projection);
            self.add_edge(
                parent,
                projection_node,
                DefinitionEdgeKind::ProjectsTo,
                if projection.synthetic {
                    "where bound"
                } else {
                    "projection"
                },
            );
            self.expand_projection(projection, projection_node, depth + 1, stack);
        }

        for projection in find_where_bound_projections(source) {
            if !seen_projections.insert(compact(&projection.source)) {
                continue;
            }
            let projection_node = self.add_projection_node(&projection);
            self.add_edge(parent, projection_node, DefinitionEdgeKind::ProjectsTo, "where bound");
            self.expand_projection(projection, projection_node, depth + 1, stack);
        }

        let mut seen_aliases = BTreeSet::<String>::new();
        for alias_ref in find_alias_refs(source, &self.aliases) {
            let Some(alias) = self.aliases.get(&alias_ref.name).cloned() else {
                continue;
            };
            let key = compact(&alias_ref.source);
            if !seen_aliases.insert(key) {
                continue;
            }
            let env = alias_env(&alias, &alias_ref.args);
            let alias_node = self.add_alias_node(&alias, &env);
            self.add_edge(parent, alias_node, DefinitionEdgeKind::References, "alias");
            self.expand_alias(alias, alias_node, env, depth + 1, stack);
        }
    }

    fn matching_impls(
        &self,
        projection: &ProjectionRef,
    ) -> Vec<(ImplDef, BTreeMap<String, String>, String)> {
        let mut matches = Vec::new();
        for impl_def in &self.impls {
            if impl_def.trait_name != projection.trait_name {
                continue;
            }
            let Some(assoc) =
                impl_def.assoc_types.iter().find(|assoc| assoc.name == projection.assoc_name)
            else {
                continue;
            };
            let _ = assoc;
            if impl_def.trait_args.len() != projection.trait_args.len() {
                continue;
            }

            let mut env = BTreeMap::new();
            let mut confidence = MatchConfidence::Exact;
            if !bind_pattern(
                &impl_def.self_ty,
                &projection.self_ty,
                &impl_def.generics,
                &mut env,
                &mut confidence,
            ) {
                continue;
            }
            let mut ok = true;
            for (pattern, actual) in impl_def.trait_args.iter().zip(&projection.trait_args) {
                if !bind_pattern(pattern, actual, &impl_def.generics, &mut env, &mut confidence) {
                    ok = false;
                    break;
                }
            }
            if ok {
                matches.push((impl_def.clone(), env, confidence.label().to_owned()));
            }
        }
        matches
    }

    fn add_projection_leaf(&mut self, projection: &ProjectionRef, parent: NodeId) {
        if let Some(trait_def) = self.traits.get(&projection.trait_name).cloned() {
            let node = self.add_node(
                DefinitionNodeKind::Trait,
                format!("trait {}", trait_def.name),
                Some(trait_def.def_path),
                Some(trait_def.source),
                BTreeMap::new(),
            );
            self.add_edge(parent, node, DefinitionEdgeKind::References, "trait");
        } else {
            let mut metadata = BTreeMap::new();
            metadata.insert(String::from("reason"), String::from("no_local_impl"));
            let node = self.add_node(
                DefinitionNodeKind::ExternalLeaf,
                projection.trait_name.clone(),
                None,
                None,
                metadata,
            );
            self.add_edge(parent, node, DefinitionEdgeKind::References, "leaf");
        }
    }

    fn add_alias_node(&mut self, alias: &AliasDef, env: &BTreeMap<String, String>) -> NodeId {
        let generics = if alias.generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", alias.generics.join(", "))
        };
        let mut metadata = BTreeMap::new();
        for (key, value) in env {
            metadata.insert(format!("arg.{key}"), value.clone());
        }
        metadata.insert(String::from("alias_key"), alias.def_path.clone());
        metadata.insert(
            String::from("definition_key"),
            format!("alias:{}:{}", alias.def_path, env_key(env)),
        );
        self.add_node(
            DefinitionNodeKind::TypeAlias,
            format!("type {}{}", alias.name, generics),
            Some(alias.def_path.clone()),
            Some(substitute(&alias.source, env)),
            metadata,
        )
    }

    fn add_projection_node(&mut self, projection: &ProjectionRef) -> NodeId {
        let mut metadata = BTreeMap::new();
        metadata.insert(String::from("self_ty"), projection.self_ty.clone());
        metadata.insert(String::from("trait"), projection.trait_name.clone());
        metadata.insert(String::from("assoc"), projection.assoc_name.clone());
        metadata.insert(String::from("projection_key"), compact(&projection.source));
        metadata.insert(
            String::from("definition_key"),
            format!("projection:{}", compact(&projection.source)),
        );
        if projection.synthetic {
            metadata.insert(String::from("synthetic"), String::from("where_bound"));
        }
        self.add_node(
            DefinitionNodeKind::Projection,
            projection.source.clone(),
            None,
            None,
            metadata,
        )
    }

    fn add_impl_node(&mut self, impl_def: &ImplDef, metadata: BTreeMap<String, String>) -> NodeId {
        let mut metadata = metadata;
        metadata.insert(String::from("impl_key"), impl_def.def_path.clone());
        self.add_node(
            DefinitionNodeKind::ImplBranch,
            impl_def.header.clone(),
            Some(impl_def.def_path.clone()),
            Some(impl_def.source.clone()),
            metadata,
        )
    }

    fn add_node(
        &mut self,
        kind: DefinitionNodeKind,
        label: String,
        def_path: Option<String>,
        source: Option<String>,
        metadata: BTreeMap<String, String>,
    ) -> NodeId {
        let id = NodeId::new(self.next_node_id);
        self.next_node_id += 1;
        self.graph.nodes.push(DefinitionNode {
            id,
            kind,
            label,
            def_path,
            span_id: None,
            source,
            metadata,
        });
        id
    }

    fn add_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        kind: DefinitionEdgeKind,
        label: impl Into<String>,
    ) {
        self.graph.edges.push(DefinitionEdge {
            from,
            to,
            kind,
            label: label.into(),
            metadata: BTreeMap::new(),
        });
    }

    fn at_max_depth(&mut self, parent: NodeId, depth: usize) -> bool {
        if depth < self.max_depth {
            return false;
        }
        let mut metadata = BTreeMap::new();
        metadata.insert(String::from("reason"), String::from("max_depth"));
        let node = self.add_node(
            DefinitionNodeKind::Unresolved,
            format!("max depth reached ({})", self.max_depth),
            None,
            None,
            metadata,
        );
        self.add_edge(parent, node, DefinitionEdgeKind::References, "max_depth");
        true
    }

    fn push_or_cycle(&mut self, parent: NodeId, key: String, stack: &mut Vec<String>) -> bool {
        if stack.iter().any(|seen| seen == &key) {
            let mut metadata = BTreeMap::new();
            metadata.insert(String::from("cycle_key"), key.clone());
            metadata.insert(String::from("reason"), String::from("cycle"));
            let node = self.add_node(DefinitionNodeKind::Cycle, key, None, None, metadata);
            self.add_edge(parent, node, DefinitionEdgeKind::References, "cycle");
            true
        } else if !self.expanded_keys.insert(key.clone()) {
            let mut metadata = BTreeMap::new();
            metadata.insert(String::from("cycle_key"), key.clone());
            metadata.insert(String::from("reason"), String::from("already_expanded"));
            let node = self.add_node(DefinitionNodeKind::Cycle, key, None, None, metadata);
            self.add_edge(parent, node, DefinitionEdgeKind::References, "already expanded");
            true
        } else {
            stack.push(key);
            false
        }
    }

    fn snippet(&self, span: Span) -> Option<String> {
        self.tcx.sess.source_map().span_to_snippet(span).ok()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchConfidence {
    Exact,
    Ambiguous,
}

impl MatchConfidence {
    const fn label(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Ambiguous => "ambiguous",
        }
    }

    fn mark_ambiguous(&mut self) {
        *self = Self::Ambiguous;
    }
}

fn owner_matches(
    def_id: LocalDefId,
    owner_path: &str,
    query: &str,
    match_kind: QueryMatchKind,
) -> bool {
    match match_kind {
        QueryMatchKind::DefId => def_index_matches(def_id.local_def_index.as_u32(), query),
        _ => owner_path_matches(owner_path, query, match_kind),
    }
}

fn generic_names(generics: &rustc_hir::Generics<'_>) -> Vec<String> {
    generics
        .params
        .iter()
        .filter_map(|param| match param.kind {
            GenericParamKind::Type {
                ..
            }
            | GenericParamKind::Const {
                ..
            } => Some(param.name.ident().name.to_string()),
            GenericParamKind::Lifetime {
                ..
            } => None,
        })
        .collect()
}

fn impl_header(source: &str, trait_source: &str, self_ty: &str) -> String {
    if let Some((head, _)) = source.split_once('{') {
        return compact_spaces(head.trim());
    }
    format!("impl {trait_source} for {self_ty}")
}

fn alias_env(alias: &AliasDef, args: &[String]) -> BTreeMap<String, String> {
    alias.generics.iter().zip(args).map(|(param, arg)| (param.clone(), arg.clone())).collect()
}

fn env_key(env: &BTreeMap<String, String>) -> String {
    env.iter()
        .map(|(key, value)| format!("{key}={}", compact(value)))
        .collect::<Vec<_>>()
        .join(",")
}

fn pop_key(stack: &mut Vec<String>, key: &str) {
    if stack.last().is_some_and(|last| last == key) {
        stack.pop();
    }
}

fn substitute(source: &str, env: &BTreeMap<String, String>) -> String {
    if env.is_empty() {
        return source.to_owned();
    }
    let mut out = String::new();
    let mut chars = source.char_indices().peekable();
    while let Some((start, ch)) = chars.next() {
        if is_ident_start(ch) {
            let mut end = start + ch.len_utf8();
            while let Some((next_idx, next_ch)) = chars.peek().copied() {
                if is_ident_continue(next_ch) {
                    chars.next();
                    end = next_idx + next_ch.len_utf8();
                } else {
                    break;
                }
            }
            let ident = &source[start..end];
            if let Some(value) = env.get(ident) {
                out.push_str(value);
            } else {
                out.push_str(ident);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn bind_pattern(
    pattern: &str,
    actual: &str,
    generic_params: &[String],
    env: &mut BTreeMap<String, String>,
    confidence: &mut MatchConfidence,
) -> bool {
    let pattern = substitute(pattern.trim(), env);
    let actual = actual.trim();
    if generic_params.iter().any(|param| param == &pattern) {
        if let Some(bound) = env.get(&pattern) {
            return compact(bound) == compact(actual);
        }
        env.insert(pattern, actual.to_owned());
        return true;
    }

    if compact(&pattern) == compact(actual) {
        return true;
    }

    if is_unknown(actual) && is_branchable_concrete(&pattern) {
        confidence.mark_ambiguous();
        return true;
    }

    if let (Some((p_name, p_args)), Some((a_name, a_args))) =
        (parse_constructor(&pattern), parse_constructor(actual))
    {
        if p_name != a_name || p_args.len() != a_args.len() {
            return false;
        }
        return p_args
            .iter()
            .zip(&a_args)
            .all(|(p_arg, a_arg)| bind_pattern(p_arg, a_arg, generic_params, env, confidence));
    }

    if let (Some(p_items), Some(a_items)) = (parse_tuple(&pattern), parse_tuple(actual)) {
        if p_items.len() != a_items.len() {
            return false;
        }
        return p_items.iter().zip(&a_items).all(|(p_item, a_item)| {
            bind_pattern(p_item, a_item, generic_params, env, confidence)
        });
    }

    false
}

fn is_unknown(value: &str) -> bool {
    let value = value.trim();
    value.starts_with('<')
        || value.contains(" as ")
        || value.contains(">::")
        || value.contains('<')
        || is_single_ident(value)
        || value.contains("tarr!")
}

fn is_branchable_concrete(value: &str) -> bool {
    let value = value.trim();
    value == "True"
        || value == "False"
        || value == "TTerm"
        || value.starts_with("TArr<")
        || value == "Zero"
        || value.starts_with("Succ<")
}

fn find_projections(source: &str) -> Vec<ProjectionRef> {
    let mut projections = Vec::new();
    let indices = source.char_indices().map(|(idx, _)| idx).collect::<Vec<_>>();
    for start in indices {
        if !source[start..].starts_with('<') {
            continue;
        }
        let Some(close) = matching_delimiter(source, start, '<', '>') else {
            continue;
        };
        let Some((assoc, end)) = parse_assoc_suffix(source, close + 1) else {
            continue;
        };
        let inner = &source[start + 1..close];
        let Some((self_ty, trait_part)) = split_top_level_as(inner) else {
            continue;
        };
        let (trait_name, trait_args) = parse_trait_path(trait_part.trim());
        projections.push(ProjectionRef {
            source: source[start..end].trim().to_owned(),
            self_ty: self_ty.trim().to_owned(),
            trait_name,
            trait_args,
            assoc_name: assoc,
            synthetic: false,
        });
    }
    projections
}

fn find_where_bound_projections(source: &str) -> Vec<ProjectionRef> {
    let mut projections = Vec::new();
    for clause in split_top_level(source, ',') {
        let Some((self_ty, bounds)) = split_top_level_colon(&clause) else {
            continue;
        };
        if self_ty.trim().is_empty()
            || self_ty.trim().starts_with("for<")
            || self_ty.trim().starts_with('\'')
        {
            continue;
        }
        for bound in split_top_level(bounds.trim(), '+') {
            let bound = bound.trim();
            if bound.is_empty()
                || bound.starts_with('\'')
                || bound.starts_with('{')
                || bound.contains('=')
            {
                continue;
            }
            let (trait_name, trait_args) = parse_trait_path(bound);
            if trait_name.is_empty() || trait_name == "Sized" {
                continue;
            }
            projections.push(ProjectionRef {
                source: format!("<{} as {}>::Output", self_ty.trim(), bound),
                self_ty: self_ty.trim().to_owned(),
                trait_name,
                trait_args,
                assoc_name: String::from("Output"),
                synthetic: true,
            });
        }
    }
    projections
}

fn find_alias_refs(source: &str, aliases: &BTreeMap<String, AliasDef>) -> Vec<AliasRef> {
    let mut refs = Vec::new();
    let mut chars = source.char_indices().peekable();
    while let Some((start, ch)) = chars.next() {
        if !is_ident_start(ch) {
            continue;
        }
        let mut end = start + ch.len_utf8();
        while let Some((next_idx, next_ch)) = chars.peek().copied() {
            if is_ident_continue(next_ch) {
                chars.next();
                end = next_idx + next_ch.len_utf8();
            } else {
                break;
            }
        }
        let name = &source[start..end];
        let Some(alias) = aliases.get(name) else {
            continue;
        };
        if alias.generics.is_empty() {
            continue;
        }
        let after_ws = skip_ws(source, end);
        if !source[after_ws..].starts_with('<') {
            continue;
        }
        let Some(close) = matching_delimiter(source, after_ws, '<', '>') else {
            continue;
        };
        let args = split_top_level(&source[after_ws + 1..close], ',');
        refs.push(AliasRef {
            name: name.to_owned(),
            args,
            source: source[start..close + 1].to_owned(),
        });
    }
    refs
}

fn parse_trait_path(source: &str) -> (String, Vec<String>) {
    let source = source.trim();
    let Some(open) = top_level_char(source, '<') else {
        return (last_path_segment(source), Vec::new());
    };
    let Some(close) = matching_delimiter(source, open, '<', '>') else {
        return (last_path_segment(source), Vec::new());
    };
    (last_path_segment(source[..open].trim()), split_top_level(&source[open + 1..close], ','))
}

fn parse_constructor(source: &str) -> Option<(String, Vec<String>)> {
    let source = source.trim();
    let open = top_level_char(source, '<')?;
    let close = matching_delimiter(source, open, '<', '>')?;
    if skip_ws(source, close + 1) != source.len() {
        return None;
    }
    Some((source[..open].trim().to_owned(), split_top_level(&source[open + 1..close], ',')))
}

fn parse_tuple(source: &str) -> Option<Vec<String>> {
    let source = source.trim();
    if !source.starts_with('(') || !source.ends_with(')') {
        return None;
    }
    let close = matching_delimiter(source, 0, '(', ')')?;
    if close + 1 != source.len() {
        return None;
    }
    Some(split_top_level(&source[1..close], ','))
}

fn parse_assoc_suffix(source: &str, start: usize) -> Option<(String, usize)> {
    let mut idx = skip_ws(source, start);
    if !source[idx..].starts_with("::") {
        return None;
    }
    idx += 2;
    idx = skip_ws(source, idx);
    let assoc_start = idx;
    while idx < source.len() {
        let ch = source[idx..].chars().next()?;
        if is_ident_continue(ch) {
            idx += ch.len_utf8();
        } else {
            break;
        }
    }
    if assoc_start == idx {
        return None;
    }
    Some((source[assoc_start..idx].to_owned(), idx))
}

fn split_top_level_as(source: &str) -> Option<(&str, &str)> {
    let bytes = source.as_bytes();
    let mut angle = 0_i32;
    let mut paren = 0_i32;
    let mut bracket = 0_i32;
    let mut idx = 0;
    while idx + 4 <= bytes.len() {
        let ch = source[idx..].chars().next()?;
        match ch {
            '<' => angle += 1,
            '>' => angle -= 1,
            '(' => paren += 1,
            ')' => paren -= 1,
            '[' => bracket += 1,
            ']' => bracket -= 1,
            'a' if angle == 0
                && paren == 0
                && bracket == 0
                && source[idx..].starts_with("as ")
                && idx > 0
                && source[..idx].ends_with(' ') =>
            {
                return Some((&source[..idx - 1], &source[idx + 3..]));
            },
            _ => {},
        }
        idx += ch.len_utf8();
    }
    None
}

fn split_top_level_colon(source: &str) -> Option<(&str, &str)> {
    let mut angle = 0_i32;
    let mut paren = 0_i32;
    let mut bracket = 0_i32;
    let mut chars = source.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        match ch {
            '<' => angle += 1,
            '>' => angle -= 1,
            '(' => paren += 1,
            ')' => paren -= 1,
            '[' => bracket += 1,
            ']' => bracket -= 1,
            ':' if angle == 0 && paren == 0 && bracket == 0 => {
                if source[idx..].starts_with("::") || source[..idx].ends_with(':') {
                    continue;
                }
                return Some((&source[..idx], &source[idx + 1..]));
            },
            _ => {},
        }
    }
    None
}

fn split_top_level(source: &str, delimiter: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut angle = 0_i32;
    let mut paren = 0_i32;
    let mut bracket = 0_i32;
    for (idx, ch) in source.char_indices() {
        match ch {
            '<' => angle += 1,
            '>' => angle -= 1,
            '(' => paren += 1,
            ')' => paren -= 1,
            '[' => bracket += 1,
            ']' => bracket -= 1,
            ch if ch == delimiter && angle == 0 && paren == 0 && bracket == 0 => {
                let item = source[start..idx].trim();
                if !item.is_empty() {
                    out.push(item.to_owned());
                }
                start = idx + ch.len_utf8();
            },
            _ => {},
        }
    }
    let item = source[start..].trim();
    if !item.is_empty() {
        out.push(item.to_owned());
    }
    out
}

fn matching_delimiter(source: &str, open: usize, open_ch: char, close_ch: char) -> Option<usize> {
    let mut depth = 0_i32;
    for (idx, ch) in source[open..].char_indices() {
        let idx = open + idx;
        if ch == open_ch {
            depth += 1;
        } else if ch == close_ch {
            depth -= 1;
            if depth == 0 {
                return Some(idx);
            }
        }
    }
    None
}

fn top_level_char(source: &str, target: char) -> Option<usize> {
    let mut angle = 0_i32;
    let mut paren = 0_i32;
    let mut bracket = 0_i32;
    for (idx, ch) in source.char_indices() {
        match ch {
            ch if ch == target && angle == 0 && paren == 0 && bracket == 0 => return Some(idx),
            '<' => angle += 1,
            '>' => angle -= 1,
            '(' => paren += 1,
            ')' => paren -= 1,
            '[' => bracket += 1,
            ']' => bracket -= 1,
            _ => {},
        }
    }
    None
}

fn skip_ws(source: &str, mut idx: usize) -> usize {
    while idx < source.len() {
        let Some(ch) = source[idx..].chars().next() else {
            break;
        };
        if !ch.is_whitespace() {
            break;
        }
        idx += ch.len_utf8();
    }
    idx
}

fn last_path_segment(source: &str) -> String {
    source.split("::").last().unwrap_or(source).trim().trim_start_matches("r#").to_owned()
}

fn compact(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn compact_spaces(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_ident_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn is_single_ident(source: &str) -> bool {
    let mut chars = source.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_ident_start(first) && chars.all(is_ident_continue)
}

#[cfg(test)]
mod tests {
    use super::{find_projections, find_where_bound_projections, split_top_level};

    #[test]
    fn finds_nested_projection_text() {
        let projections =
            find_projections("<<<State as Len>::Output as Sub<N>>::Output as IsZero>::Output");
        assert!(projections.iter().any(|projection| projection.trait_name == "IsZero"));
        assert!(projections.iter().any(|projection| projection.trait_name == "Sub"));
        assert!(projections.iter().any(|projection| projection.trait_name == "Len"));
    }

    #[test]
    fn finds_trait_bounds_as_synthetic_output_projections() {
        let projections =
            find_where_bound_projections("(): Apply<Pred, State>, State: Len + Get<N0>");
        assert!(projections.iter().any(|projection| projection.trait_name == "Apply"));
        assert!(projections.iter().any(|projection| projection.trait_name == "Len"));
        assert!(projections.iter().any(|projection| projection.trait_name == "Get"));
    }

    #[test]
    fn splits_top_level_only() {
        assert_eq!(split_top_level("A<B, C>, (D, E), F", ','), vec!["A<B, C>", "(D, E)", "F"]);
    }
}
