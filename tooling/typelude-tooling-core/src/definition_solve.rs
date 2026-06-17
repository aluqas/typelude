use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    CandidateId, DefinitionEdgeKind, DefinitionGraph, DefinitionNode, DefinitionNodeKind,
    DefinitionStats, GoalId, GoalTree, GoalTreeCandidate, GoalTreeGoal, GoalTreeSubject,
    PredicateRepr, SolveSummary, build_solve_analysis,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefinitionSolveLinkConfidence {
    FallbackText,
    NormalizedText,
    ExactPath,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionSolveLink {
    pub def_node_id: crate::NodeId,
    pub goal_ids: Vec<GoalId>,
    pub candidate_ids: Vec<CandidateId>,
    pub link_key: String,
    pub confidence: DefinitionSolveLinkConfidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DefinitionSolveLinks {
    pub links: Vec<DefinitionSolveLink>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefinitionSolveAnalysis {
    pub definition_stats: DefinitionStats,
    pub solve_summary: SolveSummary,
    pub node_metrics: Vec<DefinitionSolveNodeAnalysis>,
    pub branching_hotspots: Vec<DefinitionSolveNodeAnalysis>,
    pub solve_load_hotspots: Vec<DefinitionSolveNodeAnalysis>,
    pub projection_nodes_with_no_static_impl_branch: Vec<DefinitionSolveNodeAnalysis>,
    pub unlinked_goal_count: usize,
    pub unlinked_candidate_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefinitionSolveNodeAnalysis {
    pub def_node_id: crate::NodeId,
    pub label: String,
    pub kind: DefinitionNodeKind,
    pub static_branch_count: usize,
    pub linked_goal_count: usize,
    pub linked_candidate_count: usize,
    pub candidate_attempts_per_static_branch: f64,
}

#[must_use]
pub fn build_definition_solve_links(
    graph: &DefinitionGraph,
    tree: &GoalTree,
) -> DefinitionSolveLinks {
    let solve_index = SolveLinkIndex::from_tree(tree);
    let mut links = Vec::new();

    for node in &graph.nodes {
        let keys = definition_link_keys(node);
        if keys.is_empty() {
            continue;
        }

        let mut goal_ids = BTreeSet::<GoalId>::new();
        let mut candidate_ids = BTreeSet::<CandidateId>::new();
        let mut best_confidence = DefinitionSolveLinkConfidence::FallbackText;
        let mut link_key = keys.first().map_or_else(String::new, |key| key.raw.clone());

        for goal in &solve_index.goals {
            if let Some((confidence, key)) = match_link_keys(node, &keys, &goal.keys) {
                goal_ids.insert(goal.id);
                if confidence > best_confidence {
                    best_confidence = confidence;
                    link_key = key;
                }
            }
        }

        for candidate in &solve_index.candidates {
            if let Some((confidence, key)) = match_link_keys(node, &keys, &candidate.keys) {
                candidate_ids.insert(candidate.id);
                if confidence > best_confidence {
                    best_confidence = confidence;
                    link_key = key;
                }
            }
        }

        if !goal_ids.is_empty() || !candidate_ids.is_empty() {
            links.push(DefinitionSolveLink {
                def_node_id: node.id,
                goal_ids: goal_ids.into_iter().collect(),
                candidate_ids: candidate_ids.into_iter().collect(),
                link_key,
                confidence: best_confidence,
            });
        }
    }

    DefinitionSolveLinks {
        links,
    }
}

#[must_use]
pub fn build_definition_solve_analysis(
    graph: &DefinitionGraph,
    tree: &GoalTree,
    links: &DefinitionSolveLinks,
    unsupported_count: usize,
    top_n: usize,
) -> DefinitionSolveAnalysis {
    let solve_summary = build_solve_analysis(tree, unsupported_count, top_n).summary;
    let solve_index = SolveLinkIndex::from_tree(tree);
    let linked_goals =
        links.links.iter().flat_map(|link| link.goal_ids.iter().copied()).collect::<BTreeSet<_>>();
    let linked_candidates = links
        .links
        .iter()
        .flat_map(|link| link.candidate_ids.iter().copied())
        .collect::<BTreeSet<_>>();

    let mut linked_by_node = BTreeMap::<crate::NodeId, &DefinitionSolveLink>::new();
    for link in &links.links {
        linked_by_node.insert(link.def_node_id, link);
    }

    let static_branch_counts = graph
        .edges
        .iter()
        .filter(|edge| edge.kind == DefinitionEdgeKind::ImplementedBy)
        .fold(BTreeMap::<crate::NodeId, usize>::new(), |mut counts, edge| {
            *counts.entry(edge.from).or_default() += 1;
            counts
        });

    let mut node_metrics = graph
        .nodes
        .iter()
        .map(|node| {
            let static_branch_count = *static_branch_counts.get(&node.id).unwrap_or(&0);
            let link = linked_by_node.get(&node.id).copied();
            let linked_goal_count = link.map_or(0, |link| link.goal_ids.len());
            let linked_candidate_count = link.map_or(0, |link| link.candidate_ids.len());
            DefinitionSolveNodeAnalysis {
                def_node_id: node.id,
                label: node.label.clone(),
                kind: node.kind.clone(),
                static_branch_count,
                linked_goal_count,
                linked_candidate_count,
                candidate_attempts_per_static_branch: if static_branch_count == 0 {
                    0.0
                } else {
                    linked_candidate_count as f64 / static_branch_count as f64
                },
            }
        })
        .collect::<Vec<_>>();
    node_metrics.sort_by_key(|entry| entry.def_node_id);

    let mut branching_hotspots = node_metrics
        .iter()
        .filter(|entry| entry.static_branch_count > 0)
        .cloned()
        .collect::<Vec<_>>();
    branching_hotspots.sort_by(|left, right| {
        right
            .static_branch_count
            .cmp(&left.static_branch_count)
            .then_with(|| right.linked_candidate_count.cmp(&left.linked_candidate_count))
            .then_with(|| left.def_node_id.cmp(&right.def_node_id))
    });
    branching_hotspots.truncate(top_n);

    let mut solve_load_hotspots = node_metrics
        .iter()
        .filter(|entry| entry.linked_goal_count > 0 || entry.linked_candidate_count > 0)
        .cloned()
        .collect::<Vec<_>>();
    solve_load_hotspots.sort_by(|left, right| {
        right
            .linked_candidate_count
            .cmp(&left.linked_candidate_count)
            .then_with(|| right.linked_goal_count.cmp(&left.linked_goal_count))
            .then_with(|| right.static_branch_count.cmp(&left.static_branch_count))
            .then_with(|| left.def_node_id.cmp(&right.def_node_id))
    });
    solve_load_hotspots.truncate(top_n);

    let projection_nodes_with_no_static_impl_branch = node_metrics
        .iter()
        .filter(|entry| {
            entry.kind == DefinitionNodeKind::Projection && entry.static_branch_count == 0
        })
        .take(top_n)
        .cloned()
        .collect();

    DefinitionSolveAnalysis {
        definition_stats: graph.stats(),
        solve_summary,
        node_metrics,
        branching_hotspots,
        solve_load_hotspots,
        projection_nodes_with_no_static_impl_branch,
        unlinked_goal_count: solve_index
            .goals
            .iter()
            .filter(|goal| !linked_goals.contains(&goal.id))
            .count(),
        unlinked_candidate_count: solve_index
            .candidates
            .iter()
            .filter(|candidate| !linked_candidates.contains(&candidate.id))
            .count(),
    }
}

#[must_use]
pub fn render_definition_solve_analysis_text(analysis: &DefinitionSolveAnalysis) -> String {
    let mut lines = vec![
        String::from("definition_solve_analysis:"),
        format!(
            "definition.nodes={} projections={} impl_branches={} static_cycles={} repeated_nodes={} max_depth_leaves={}",
            analysis.definition_stats.node_count,
            analysis.definition_stats.projection_count,
            analysis.definition_stats.impl_branch_count,
            analysis.definition_stats.cycle_count,
            analysis.definition_stats.repeated_nodes,
            analysis.definition_stats.max_depth_leaves
        ),
        format!(
            "solve.goals={} candidates={} max_goal_depth={} unlinked_goals={} unlinked_candidates={}",
            analysis.solve_summary.goals,
            analysis.solve_summary.candidates,
            analysis.solve_summary.max_goal_depth,
            analysis.unlinked_goal_count,
            analysis.unlinked_candidate_count
        ),
    ];

    push_node_section("branching_hotspots", &analysis.branching_hotspots, &mut lines);
    push_node_section("solve_load_hotspots", &analysis.solve_load_hotspots, &mut lines);
    push_node_section(
        "projection_nodes_with_no_static_impl_branch",
        &analysis.projection_nodes_with_no_static_impl_branch,
        &mut lines,
    );

    lines.join("\n")
}

fn push_node_section(
    label: &str,
    entries: &[DefinitionSolveNodeAnalysis],
    lines: &mut Vec<String>,
) {
    if entries.is_empty() {
        return;
    }
    lines.push(format!("{label}:"));
    for entry in entries {
        lines.push(format!(
            "node#{} branches={} goals={} candidates={} per_branch={:.2} :: {}",
            entry.def_node_id.value(),
            entry.static_branch_count,
            entry.linked_goal_count,
            entry.linked_candidate_count,
            entry.candidate_attempts_per_static_branch,
            one_line(&entry.label)
        ));
    }
}

#[derive(Debug, Clone)]
struct LinkKey {
    raw: String,
    normalized: String,
    exact_path: bool,
}

#[derive(Debug, Clone)]
struct SolveGoalLinkData {
    id: GoalId,
    keys: Vec<LinkKey>,
}

#[derive(Debug, Clone)]
struct SolveCandidateLinkData {
    id: CandidateId,
    keys: Vec<LinkKey>,
}

#[derive(Debug, Clone, Default)]
struct SolveLinkIndex {
    goals: Vec<SolveGoalLinkData>,
    candidates: Vec<SolveCandidateLinkData>,
}

impl SolveLinkIndex {
    fn from_tree(tree: &GoalTree) -> Self {
        let mut index = Self::default();
        for subject in &tree.subjects {
            let subject_keys = subject_link_keys(subject);
            for root in &subject.roots {
                collect_goal_link_data(root, &subject_keys, &mut index);
            }
        }
        index
    }
}

fn collect_goal_link_data(
    goal: &GoalTreeGoal,
    subject_keys: &[LinkKey],
    index: &mut SolveLinkIndex,
) {
    let mut goal_keys = subject_keys.to_vec();
    goal_keys.extend(predicate_link_keys(&goal.predicate));
    index.goals.push(SolveGoalLinkData {
        id: goal.id,
        keys: goal_keys,
    });

    for candidate in &goal.candidates {
        index.candidates.push(SolveCandidateLinkData {
            id: candidate.id,
            keys: candidate_link_keys(candidate),
        });
    }

    for child in &goal.children {
        collect_goal_link_data(child, subject_keys, index);
    }
}

fn definition_link_keys(node: &DefinitionNode) -> Vec<LinkKey> {
    let mut keys = Vec::new();
    for name in ["projection_key", "predicate_key", "impl_key", "alias_key", "definition_key"] {
        if let Some(value) = node.metadata.get(name) {
            keys.push(LinkKey::new(value, false));
        }
    }
    if let Some(def_path) = &node.def_path {
        keys.push(LinkKey::new(def_path, true));
    }
    keys.push(LinkKey::new(&node.label, false));
    if let Some(source) = &node.source {
        keys.push(LinkKey::new(source, false));
    }
    dedupe_keys(keys)
}

fn subject_link_keys(subject: &GoalTreeSubject) -> Vec<LinkKey> {
    let mut keys = Vec::new();
    for name in ["owner_path", "path"] {
        if let Some(value) = subject.metadata.get(name) {
            keys.push(LinkKey::new(value, true));
        }
    }
    if let Some(value) = subject.metadata.get("clause") {
        keys.push(LinkKey::new(value, false));
    }
    keys.push(LinkKey::new(&subject.label, true));
    dedupe_keys(keys)
}

fn predicate_link_keys(predicate: &PredicateRepr) -> Vec<LinkKey> {
    let mut keys = Vec::new();
    match predicate {
        PredicateRepr::DebugText(text) => keys.push(LinkKey::new(text, false)),
        PredicateRepr::TraitPredicate {
            trait_path,
            self_ty,
            args,
        } => {
            keys.push(LinkKey::new(trait_path, false));
            keys.push(LinkKey::new(self_ty, false));
            for arg in args {
                keys.push(LinkKey::new(arg, false));
            }
        },
        PredicateRepr::AliasRelate {
            lhs,
            rhs,
        } => {
            keys.push(LinkKey::new(lhs, false));
            keys.push(LinkKey::new(rhs, false));
        },
        PredicateRepr::NormalizesTo {
            alias,
            target,
        } => {
            keys.push(LinkKey::new(alias, false));
            keys.push(LinkKey::new(target, false));
        },
        PredicateRepr::Unknown => {},
    }
    keys.push(LinkKey::new(&predicate.debug_text(), false));
    dedupe_keys(keys)
}

fn candidate_link_keys(candidate: &GoalTreeCandidate) -> Vec<LinkKey> {
    let mut keys = Vec::new();
    for name in ["impl_key", "raw_candidate_kind"] {
        if let Some(value) = candidate.metadata.get(name) {
            keys.push(LinkKey::new(value, name == "impl_key"));
        }
    }
    dedupe_keys(keys)
}

fn match_link_keys(
    node: &DefinitionNode,
    definition_keys: &[LinkKey],
    solve_keys: &[LinkKey],
) -> Option<(DefinitionSolveLinkConfidence, String)> {
    let mut best = None::<(DefinitionSolveLinkConfidence, String)>;
    for definition_key in definition_keys {
        for solve_key in solve_keys {
            let confidence = if definition_key.exact_path
                && solve_key.exact_path
                && definition_key.raw == solve_key.raw
            {
                Some(DefinitionSolveLinkConfidence::ExactPath)
            } else if definition_key.exact_path
                && solve_key.raw.contains(&definition_key.raw)
                && definition_key.raw.contains("::")
            {
                Some(DefinitionSolveLinkConfidence::ExactPath)
            } else if definition_key.normalized == solve_key.normalized
                && is_substantial_key(&definition_key.normalized)
            {
                Some(DefinitionSolveLinkConfidence::NormalizedText)
            } else if allow_fallback_text(node)
                && contains_substantial(&definition_key.normalized, &solve_key.normalized)
            {
                Some(DefinitionSolveLinkConfidence::FallbackText)
            } else {
                None
            };

            if let Some(confidence) = confidence {
                let replace = best.as_ref().is_none_or(|(current, _)| confidence > *current);
                if replace {
                    best = Some((confidence, definition_key.raw.clone()));
                }
            }
        }
    }
    best
}

fn allow_fallback_text(node: &DefinitionNode) -> bool {
    matches!(
        node.kind,
        DefinitionNodeKind::Projection
            | DefinitionNodeKind::WherePredicate
            | DefinitionNodeKind::ImplBranch
            | DefinitionNodeKind::AssociatedTypeBody
            | DefinitionNodeKind::TypeAlias
    )
}

fn contains_substantial(left: &str, right: &str) -> bool {
    if !is_substantial_key(left) || !is_substantial_key(right) {
        return false;
    }
    left.contains(right) || right.contains(left)
}

fn is_substantial_key(value: &str) -> bool {
    value.len() >= 8 && (value.contains("::") || value.contains('<') || value.contains("as"))
}

fn dedupe_keys(keys: Vec<LinkKey>) -> Vec<LinkKey> {
    let mut out = Vec::<LinkKey>::new();
    let mut positions = BTreeMap::<String, usize>::new();
    for key in keys.into_iter().filter(|key| !key.normalized.is_empty()) {
        if let Some(index) = positions.get(&key.normalized).copied() {
            if key.exact_path && !out[index].exact_path {
                out[index] = key;
            }
        } else {
            positions.insert(key.normalized.clone(), out.len());
            out.push(key);
        }
    }
    out
}

impl LinkKey {
    fn new(raw: &str, exact_path: bool) -> Self {
        Self {
            raw: raw.to_owned(),
            normalized: normalize_link_text(raw),
            exact_path,
        }
    }
}

fn normalize_link_text(value: &str) -> String {
    unwrap_binder(value)
        .chars()
        .filter(|ch| !ch.is_whitespace() && *ch != '"' && *ch != '\'')
        .collect()
}

fn unwrap_binder(raw: &str) -> &str {
    let text = raw.trim();
    if let Some(stripped) = text.strip_prefix("Binder { value: ") {
        stripped.strip_suffix(", bound_vars: [] }").unwrap_or(stripped)
    } else {
        text
    }
}

fn one_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        CandidateId, CandidateKind, DefinitionEdge, DefinitionEdgeKind, DefinitionGraph,
        DefinitionNode, DefinitionNodeKind, GoalId, GoalResult, GoalTree, GoalTreeCandidate,
        GoalTreeGoal, GoalTreeSubject, NodeId, PredicateRepr, SubjectId, SubjectKind, TraceId,
    };

    use super::{
        DefinitionSolveLinkConfidence, build_definition_solve_analysis,
        build_definition_solve_links, render_definition_solve_analysis_text,
    };

    fn fixture_graph() -> DefinitionGraph {
        DefinitionGraph {
            roots: vec![NodeId::new(1)],
            nodes: vec![
                DefinitionNode {
                    id: NodeId::new(1),
                    kind: DefinitionNodeKind::TypeAlias,
                    label: String::from("type Fib<N>"),
                    def_path: Some(String::from("fixture::Fib")),
                    span_id: None,
                    source: None,
                    metadata: BTreeMap::from([(
                        String::from("alias_key"),
                        String::from("fixture::Fib"),
                    )]),
                },
                DefinitionNode {
                    id: NodeId::new(2),
                    kind: DefinitionNodeKind::Projection,
                    label: String::from("<() as Apply<OpWhile, State>>::Output"),
                    def_path: None,
                    span_id: None,
                    source: None,
                    metadata: BTreeMap::from([(
                        String::from("projection_key"),
                        String::from("<() as Apply<OpWhile, State>>::Output"),
                    )]),
                },
                DefinitionNode {
                    id: NodeId::new(3),
                    kind: DefinitionNodeKind::ImplBranch,
                    label: String::from("impl Apply<OpWhile, State> for ()"),
                    def_path: Some(String::from("fixture::{impl#0}")),
                    span_id: None,
                    source: None,
                    metadata: BTreeMap::from([(
                        String::from("impl_key"),
                        String::from("fixture::{impl#0}"),
                    )]),
                },
            ],
            edges: vec![
                DefinitionEdge {
                    from: NodeId::new(1),
                    to: NodeId::new(2),
                    kind: DefinitionEdgeKind::ExpandsTo,
                    label: String::from("expands_to"),
                    metadata: BTreeMap::new(),
                },
                DefinitionEdge {
                    from: NodeId::new(2),
                    to: NodeId::new(3),
                    kind: DefinitionEdgeKind::ImplementedBy,
                    label: String::from("implemented_by"),
                    metadata: BTreeMap::new(),
                },
            ],
        }
    }

    fn fixture_tree() -> GoalTree {
        GoalTree {
            trace_id: TraceId::new(1),
            subjects: vec![GoalTreeSubject {
                id: SubjectId::new(1),
                kind: SubjectKind::Predicate,
                label: String::from("fixture::Fib"),
                metadata: BTreeMap::from([(
                    String::from("owner_path"),
                    String::from("fixture::Fib"),
                )]),
                roots: vec![GoalTreeGoal {
                    id: GoalId::new(1),
                    parent_goal_id: None,
                    predicate: PredicateRepr::DebugText(String::from(
                        "Binder { value: NormalizesTo(<() as Apply<OpWhile, State>>::Output, T), bound_vars: [] }",
                    )),
                    result: GoalResult::Success,
                    depth: 0,
                    semantic_tags: Vec::new(),
                    candidates: vec![GoalTreeCandidate {
                        id: CandidateId::new(1),
                        kind: CandidateKind::Impl,
                        result: GoalResult::Success,
                        semantic_tags: Vec::new(),
                        metadata: BTreeMap::from([
                            (String::from("impl_key"), String::from("fixture::{impl#0}")),
                            (
                                String::from("raw_candidate_kind"),
                                String::from("ImplCandidate(fixture::{impl#0})"),
                            ),
                        ]),
                    }],
                    children: Vec::new(),
                }],
            }],
        }
    }

    #[test]
    fn links_definition_and_solve_by_metadata_and_text() {
        let links = build_definition_solve_links(&fixture_graph(), &fixture_tree());
        assert!(links.links.iter().any(|link| link.def_node_id == NodeId::new(1)
            && link.confidence == DefinitionSolveLinkConfidence::ExactPath));
        assert!(
            links.links.iter().any(|link| link.def_node_id == NodeId::new(2)
                && link.goal_ids.contains(&GoalId::new(1)))
        );
        assert!(links.links.iter().any(|link| link.def_node_id == NodeId::new(3)
            && link.candidate_ids.contains(&CandidateId::new(1))));
        let json = serde_json::to_string(&links).expect("links should serialize");
        let parsed: super::DefinitionSolveLinks =
            serde_json::from_str(&json).expect("links should deserialize");
        assert_eq!(parsed, links);
    }

    #[test]
    fn analysis_reports_static_and_solve_hotspots() {
        let graph = fixture_graph();
        let tree = fixture_tree();
        let links = build_definition_solve_links(&graph, &tree);
        let analysis = build_definition_solve_analysis(&graph, &tree, &links, 0, 10);
        assert_eq!(analysis.solve_summary.goals, 1);
        assert!(analysis.branching_hotspots.iter().any(|entry| {
            entry.def_node_id == NodeId::new(2) && entry.static_branch_count == 1
        }));
        let rendered = render_definition_solve_analysis_text(&analysis);
        assert!(rendered.contains("branching_hotspots"));
        assert!(rendered.contains("solve_load_hotspots"));
    }
}
