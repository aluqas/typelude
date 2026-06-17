use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::{NodeId, SpanId, ToolingError, ToolingResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefinitionNodeKind {
    TypeAlias,
    Projection,
    Trait,
    ImplBranch,
    AssociatedTypeBody,
    WherePredicate,
    TypeRef,
    ExternalLeaf,
    Cycle,
    Unresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefinitionEdgeKind {
    ExpandsTo,
    ProjectsTo,
    ImplementedBy,
    HasWherePredicate,
    HasAssociatedTypeBody,
    References,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionNode {
    pub id: NodeId,
    pub kind: DefinitionNodeKind,
    pub label: String,
    pub def_path: Option<String>,
    pub span_id: Option<SpanId>,
    pub source: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionEdge {
    pub from: NodeId,
    pub to: NodeId,
    pub kind: DefinitionEdgeKind,
    pub label: String,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DefinitionGraph {
    pub roots: Vec<NodeId>,
    pub nodes: Vec<DefinitionNode>,
    pub edges: Vec<DefinitionEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DefinitionStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub projection_count: usize,
    pub impl_branch_count: usize,
    pub where_predicate_count: usize,
    pub cycle_count: usize,
    #[serde(default)]
    pub repeated_nodes: usize,
    pub external_leaf_count: usize,
    #[serde(default)]
    pub max_depth_leaves: usize,
    pub max_depth: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DefinitionCompactMode {
    #[default]
    Off,
    Basic,
    Aggressive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefinitionRenderOptions {
    pub compact: DefinitionCompactMode,
    #[serde(default)]
    pub focus: Vec<String>,
    pub root_node: Option<NodeId>,
    pub show_sources: bool,
}

impl Default for DefinitionRenderOptions {
    fn default() -> Self {
        Self {
            compact: DefinitionCompactMode::Off,
            focus: Vec::new(),
            root_node: None,
            show_sources: true,
        }
    }
}

impl DefinitionGraph {
    #[must_use]
    pub fn stats(&self) -> DefinitionStats {
        let mut stats = DefinitionStats {
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
            max_depth: self.max_depth(),
            ..DefinitionStats::default()
        };
        for node in &self.nodes {
            match node.kind {
                DefinitionNodeKind::Projection => stats.projection_count += 1,
                DefinitionNodeKind::ImplBranch => stats.impl_branch_count += 1,
                DefinitionNodeKind::WherePredicate => stats.where_predicate_count += 1,
                DefinitionNodeKind::Cycle => {
                    if node
                        .metadata
                        .get("reason")
                        .is_some_and(|reason| reason == "already_expanded")
                    {
                        stats.repeated_nodes += 1;
                    } else {
                        stats.cycle_count += 1;
                    }
                },
                DefinitionNodeKind::ExternalLeaf => stats.external_leaf_count += 1,
                DefinitionNodeKind::Unresolved => {
                    if node.metadata.get("reason").is_some_and(|reason| reason == "max_depth") {
                        stats.max_depth_leaves += 1;
                    }
                },
                DefinitionNodeKind::TypeAlias
                | DefinitionNodeKind::Trait
                | DefinitionNodeKind::AssociatedTypeBody
                | DefinitionNodeKind::TypeRef => {},
            }
        }
        stats
    }

    #[must_use]
    pub fn to_tree_text(&self) -> String {
        if self.nodes.is_empty() {
            return String::from("definition_tree: empty");
        }

        self.render_tree_text(&DefinitionRenderOptions::default())
    }

    #[must_use]
    pub fn to_tree_text_with_options(
        &self,
        options: &DefinitionRenderOptions,
    ) -> ToolingResult<String> {
        let graph = self.view(options)?;
        Ok(graph.render_tree_text(options))
    }

    #[must_use]
    pub fn view(&self, options: &DefinitionRenderOptions) -> ToolingResult<Self> {
        if self.nodes.is_empty() {
            return Ok(Self::default());
        }

        let nodes = self.node_map();
        let children = self.children_map();
        let roots = if let Some(root_node) = options.root_node {
            if !nodes.contains_key(&root_node) {
                return Err(ToolingError::Parse(format!(
                    "definition node {} does not exist",
                    root_node.value()
                )));
            }
            vec![root_node]
        } else if self.roots.is_empty() {
            inferred_roots(self)
        } else {
            self.roots.clone()
        };

        let reachable = descendants_of(&roots, &children);
        let mut included = BTreeSet::<NodeId>::new();
        if options.focus.is_empty() {
            included = reachable;
        } else {
            let parents = self.parents_map();
            let matched = self
                .nodes
                .iter()
                .filter(|node| reachable.contains(&node.id))
                .filter(|node| node_matches_focus(node, &options.focus))
                .map(|node| node.id)
                .collect::<Vec<_>>();
            for node_id in matched {
                included.extend(ancestors_of(node_id, &parents));
                included.extend(descendants_of(&[node_id], &children));
            }
        }

        let mut nodes = self
            .nodes
            .iter()
            .filter(|node| included.contains(&node.id))
            .cloned()
            .collect::<Vec<_>>();
        nodes.sort_by_key(|node| node.id);
        let mut edges = self
            .edges
            .iter()
            .filter(|edge| included.contains(&edge.from) && included.contains(&edge.to))
            .cloned()
            .collect::<Vec<_>>();
        edges.sort_by_key(|edge| (edge.from, edge.to, edge.label.clone()));

        let incoming = edges.iter().map(|edge| edge.to).collect::<BTreeSet<_>>();
        let roots = roots
            .into_iter()
            .filter(|root| included.contains(root))
            .chain(nodes.iter().map(|node| node.id).filter(|node_id| !incoming.contains(node_id)))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();

        Ok(Self {
            roots,
            nodes,
            edges,
        })
    }

    fn render_tree_text(&self, options: &DefinitionRenderOptions) -> String {
        if self.nodes.is_empty() {
            return String::from("definition_tree: empty");
        }

        let stats = self.stats();
        let mut lines = vec![
            String::from("view_kind=definition_tree"),
            format!(
                "nodes={} edges={} projections={} impl_branches={} where_predicates={} cycles={} repeated_nodes={} external_leaves={} max_depth_leaves={} max_depth={}",
                stats.node_count,
                stats.edge_count,
                stats.projection_count,
                stats.impl_branch_count,
                stats.where_predicate_count,
                stats.cycle_count,
                stats.repeated_nodes,
                stats.external_leaf_count,
                stats.max_depth_leaves,
                stats.max_depth
            ),
        ];
        let nodes = self.node_map();
        let children = self.children_map();
        let cycle_targets = self.definition_key_targets();
        let roots = if self.roots.is_empty() {
            inferred_roots(self)
        } else {
            self.roots.clone()
        };
        for root in roots {
            self.render_tree_node(
                root,
                0,
                &nodes,
                &children,
                &cycle_targets,
                &mut BTreeSet::new(),
                &mut lines,
                options,
            );
        }
        lines.join("\n")
    }

    #[must_use]
    pub fn to_dot_with_options(&self, options: &DefinitionRenderOptions) -> ToolingResult<String> {
        Ok(self.view(options)?.to_dot())
    }

    #[must_use]
    pub fn to_mermaid_with_options(
        &self,
        options: &DefinitionRenderOptions,
    ) -> ToolingResult<String> {
        Ok(self.view(options)?.to_mermaid())
    }

    #[must_use]
    pub fn to_dot(&self) -> String {
        let mut out = String::from("digraph definition {\n  rankdir=TB;\n");
        for node in &self.nodes {
            out.push_str(&format!(
                "  n{} [label=\"{}\" shape={}];\n",
                node.id.value(),
                escape_dot(&node.label),
                dot_shape(&node.kind)
            ));
        }
        for edge in &self.edges {
            out.push_str(&format!(
                "  n{} -> n{} [label=\"{}\"];\n",
                edge.from.value(),
                edge.to.value(),
                escape_dot(&edge.label)
            ));
        }
        out.push('}');
        out
    }

    #[must_use]
    pub fn to_mermaid(&self) -> String {
        let mut out = String::from("flowchart TD\n");
        for node in &self.nodes {
            let id = node.id.value();
            let label = node.label.replace('"', "'");
            let shape = match node.kind {
                DefinitionNodeKind::TypeAlias => format!("n{id}([\"{label}\"])"),
                DefinitionNodeKind::Projection => format!("n{id}{{\"{label}\"}}"),
                DefinitionNodeKind::ImplBranch => format!("n{id}[\"{label}\"]"),
                DefinitionNodeKind::AssociatedTypeBody => format!("n{id}([\"{label}\"])"),
                DefinitionNodeKind::WherePredicate => format!("n{id}((\"{label}\"))"),
                DefinitionNodeKind::Trait
                | DefinitionNodeKind::TypeRef
                | DefinitionNodeKind::ExternalLeaf
                | DefinitionNodeKind::Cycle
                | DefinitionNodeKind::Unresolved => format!("n{id}[\"{label}\"]"),
            };
            out.push_str(&format!("  {shape}\n"));
        }
        for edge in &self.edges {
            out.push_str(&format!(
                "  n{} -->|{}| n{}\n",
                edge.from.value(),
                edge.label.replace('"', "'"),
                edge.to.value()
            ));
        }
        out
    }

    fn render_tree_node(
        &self,
        node_id: NodeId,
        depth: usize,
        nodes: &BTreeMap<NodeId, &DefinitionNode>,
        children: &BTreeMap<NodeId, Vec<&DefinitionEdge>>,
        cycle_targets: &BTreeMap<String, NodeId>,
        stack: &mut BTreeSet<NodeId>,
        lines: &mut Vec<String>,
        options: &DefinitionRenderOptions,
    ) {
        let Some(node) = nodes.get(&node_id) else {
            return;
        };
        let indent = "  ".repeat(depth);
        let kind = node_kind(&node.kind);
        let mut line =
            format!("{indent}- {kind}: {}", format_definition_text(&node.label, options.compact));
        if let Some(def_path) = &node.def_path {
            line.push_str(&format!(" [{def_path}]"));
        }
        if options.show_sources {
            if let Some(source) = &node.source {
                let source = format_definition_text(source, options.compact);
                if source != node.label {
                    line.push_str(&format!(" = {source}"));
                }
            }
        }
        match node.kind {
            DefinitionNodeKind::Cycle => {
                let reason = node.metadata.get("reason").map_or("cycle", String::as_str);
                let key = node.metadata.get("cycle_key").map_or(&node.label, |key| key);
                if let Some(target) = cycle_targets.get(key) {
                    line.push_str(&format!(" {reason} -> node#{}", target.value()));
                } else {
                    line.push_str(&format!(
                        " {reason} -> {}",
                        format_definition_text(key, options.compact)
                    ));
                }
            },
            DefinitionNodeKind::Unresolved
                if node.metadata.get("reason").is_some_and(|reason| reason == "max_depth") =>
            {
                line.push_str(" reason=max_depth");
            },
            _ => {
                if let Some(reason) = node.metadata.get("reason") {
                    line.push_str(&format!(" reason={reason}"));
                }
            },
        }
        lines.push(line);

        if !stack.insert(node_id) {
            lines.push(format!("{indent}  - cycle: node#{}", node.id.value()));
            return;
        }
        if let Some(edges) = children.get(&node_id) {
            for edge in edges {
                let edge_indent = "  ".repeat(depth + 1);
                lines.push(format!("{edge_indent}=> {}: {}", edge.label, edge_kind(&edge.kind)));
                self.render_tree_node(
                    edge.to,
                    depth + 2,
                    nodes,
                    children,
                    cycle_targets,
                    stack,
                    lines,
                    options,
                );
            }
        }
        stack.remove(&node_id);
    }

    fn node_map(&self) -> BTreeMap<NodeId, &DefinitionNode> {
        self.nodes.iter().map(|node| (node.id, node)).collect()
    }

    fn children_map(&self) -> BTreeMap<NodeId, Vec<&DefinitionEdge>> {
        let mut children = BTreeMap::<NodeId, Vec<&DefinitionEdge>>::new();
        for edge in &self.edges {
            children.entry(edge.from).or_default().push(edge);
        }
        children
    }

    fn parents_map(&self) -> BTreeMap<NodeId, Vec<NodeId>> {
        let mut parents = BTreeMap::<NodeId, Vec<NodeId>>::new();
        for edge in &self.edges {
            parents.entry(edge.to).or_default().push(edge.from);
        }
        parents
    }

    fn definition_key_targets(&self) -> BTreeMap<String, NodeId> {
        let mut targets = BTreeMap::new();
        for node in &self.nodes {
            if matches!(node.kind, DefinitionNodeKind::Cycle) {
                continue;
            }
            if let Some(key) = node.metadata.get("definition_key") {
                targets.entry(key.clone()).or_insert(node.id);
            }
        }
        targets
    }

    fn max_depth(&self) -> usize {
        let children = self.children_map();
        let roots = if self.roots.is_empty() {
            inferred_roots(self)
        } else {
            self.roots.clone()
        };
        let mut max_depth = 0;
        let mut queue = VecDeque::<(NodeId, usize)>::new();
        for root in roots {
            queue.push_back((root, 0));
        }
        let mut seen = BTreeSet::<NodeId>::new();
        while let Some((node_id, depth)) = queue.pop_front() {
            if !seen.insert(node_id) {
                continue;
            }
            max_depth = max_depth.max(depth);
            if let Some(edges) = children.get(&node_id) {
                for edge in edges {
                    queue.push_back((edge.to, depth + 1));
                }
            }
        }
        max_depth
    }
}

fn descendants_of(
    roots: &[NodeId],
    children: &BTreeMap<NodeId, Vec<&DefinitionEdge>>,
) -> BTreeSet<NodeId> {
    let mut included = BTreeSet::<NodeId>::new();
    let mut queue = VecDeque::<NodeId>::new();
    for root in roots {
        queue.push_back(*root);
    }
    while let Some(node_id) = queue.pop_front() {
        if !included.insert(node_id) {
            continue;
        }
        if let Some(edges) = children.get(&node_id) {
            for edge in edges {
                queue.push_back(edge.to);
            }
        }
    }
    included
}

fn ancestors_of(node_id: NodeId, parents: &BTreeMap<NodeId, Vec<NodeId>>) -> BTreeSet<NodeId> {
    let mut included = BTreeSet::<NodeId>::new();
    let mut queue = VecDeque::from([node_id]);
    while let Some(current) = queue.pop_front() {
        if !included.insert(current) {
            continue;
        }
        if let Some(parent_ids) = parents.get(&current) {
            for parent_id in parent_ids {
                queue.push_back(*parent_id);
            }
        }
    }
    included
}

fn node_matches_focus(node: &DefinitionNode, focus: &[String]) -> bool {
    let mut haystack = vec![node.label.as_str()];
    if let Some(def_path) = &node.def_path {
        haystack.push(def_path);
    }
    if let Some(source) = &node.source {
        haystack.push(source);
    }
    for (key, value) in &node.metadata {
        haystack.push(key);
        haystack.push(value);
    }
    focus
        .iter()
        .any(|needle| !needle.is_empty() && haystack.iter().any(|value| value.contains(needle)))
}

fn inferred_roots(graph: &DefinitionGraph) -> Vec<NodeId> {
    let targets = graph.edges.iter().map(|edge| edge.to).collect::<BTreeSet<_>>();
    graph.nodes.iter().filter(|node| !targets.contains(&node.id)).map(|node| node.id).collect()
}

fn one_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn format_definition_text(value: &str, compact: DefinitionCompactMode) -> String {
    let line = one_line(value);
    match compact {
        DefinitionCompactMode::Off => line,
        DefinitionCompactMode::Basic => truncate_middle(&line, 180),
        DefinitionCompactMode::Aggressive => truncate_middle(&line, 96),
    }
}

fn truncate_middle(value: &str, limit: usize) -> String {
    let char_count = value.chars().count();
    if char_count <= limit {
        return value.to_owned();
    }
    if limit <= 8 {
        return value.chars().take(limit).collect();
    }
    let keep = limit - 3;
    let left = keep / 2;
    let right = keep - left;
    let prefix = value.chars().take(left).collect::<String>();
    let suffix =
        value.chars().rev().take(right).collect::<Vec<_>>().into_iter().rev().collect::<String>();
    format!("{prefix}...{suffix}")
}

fn edge_kind(kind: &DefinitionEdgeKind) -> &'static str {
    match kind {
        DefinitionEdgeKind::ExpandsTo => "expands_to",
        DefinitionEdgeKind::ProjectsTo => "projects_to",
        DefinitionEdgeKind::ImplementedBy => "implemented_by",
        DefinitionEdgeKind::HasWherePredicate => "has_where_predicate",
        DefinitionEdgeKind::HasAssociatedTypeBody => "has_associated_type_body",
        DefinitionEdgeKind::References => "references",
    }
}

fn node_kind(kind: &DefinitionNodeKind) -> &'static str {
    match kind {
        DefinitionNodeKind::TypeAlias => "type_alias",
        DefinitionNodeKind::Projection => "projection",
        DefinitionNodeKind::Trait => "trait",
        DefinitionNodeKind::ImplBranch => "impl_branch",
        DefinitionNodeKind::AssociatedTypeBody => "associated_type_body",
        DefinitionNodeKind::WherePredicate => "where_predicate",
        DefinitionNodeKind::TypeRef => "type_ref",
        DefinitionNodeKind::ExternalLeaf => "external_leaf",
        DefinitionNodeKind::Cycle => "cycle",
        DefinitionNodeKind::Unresolved => "unresolved",
    }
}

fn dot_shape(kind: &DefinitionNodeKind) -> &'static str {
    match kind {
        DefinitionNodeKind::TypeAlias => "ellipse",
        DefinitionNodeKind::Projection => "diamond",
        DefinitionNodeKind::Trait => "hexagon",
        DefinitionNodeKind::ImplBranch => "box",
        DefinitionNodeKind::AssociatedTypeBody => "ellipse",
        DefinitionNodeKind::WherePredicate => "note",
        DefinitionNodeKind::TypeRef => "box",
        DefinitionNodeKind::ExternalLeaf => "box",
        DefinitionNodeKind::Cycle => "octagon",
        DefinitionNodeKind::Unresolved => "octagon",
    }
}

fn escape_dot(value: &str) -> String {
    one_line(value).replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        DefinitionCompactMode, DefinitionEdge, DefinitionEdgeKind, DefinitionGraph,
        DefinitionNode, DefinitionNodeKind, DefinitionRenderOptions,
    };
    use crate::NodeId;

    fn sample_graph() -> DefinitionGraph {
        DefinitionGraph {
            roots: vec![NodeId::new(1)],
            nodes: vec![
                DefinitionNode {
                    id: NodeId::new(1),
                    kind: DefinitionNodeKind::TypeAlias,
                    label: String::from("type Fib<N>"),
                    def_path: Some(String::from("ty_fibonacci::Fib")),
                    span_id: None,
                    source: Some(String::from("type Fib<N> = <() as Apply<OpWhile, N>>::Output;")),
                    metadata: BTreeMap::new(),
                },
                DefinitionNode {
                    id: NodeId::new(2),
                    kind: DefinitionNodeKind::Projection,
                    label: String::from("<() as Apply<OpWhile, N>>::Output"),
                    def_path: None,
                    span_id: None,
                    source: None,
                    metadata: BTreeMap::new(),
                },
            ],
            edges: vec![DefinitionEdge {
                from: NodeId::new(1),
                to: NodeId::new(2),
                kind: DefinitionEdgeKind::ExpandsTo,
                label: String::from("expands_to"),
                metadata: BTreeMap::new(),
            }],
        }
    }

    #[test]
    fn definition_graph_json_roundtrips() {
        let graph = sample_graph();
        let json = serde_json::to_string(&graph).expect("definition graph should serialize");
        let parsed: DefinitionGraph =
            serde_json::from_str(&json).expect("definition graph should deserialize");
        assert_eq!(parsed, graph);
    }

    #[test]
    fn tree_renderer_includes_projection() {
        let rendered = sample_graph().to_tree_text();
        assert!(rendered.contains("definition_tree"));
        assert!(rendered.contains("projection"));
        assert!(rendered.contains("Apply<OpWhile"));
    }

    #[test]
    fn graph_renderers_emit_shapes() {
        let graph = sample_graph();
        assert!(graph.to_dot().contains("digraph definition"));
        assert!(graph.to_mermaid().contains("flowchart TD"));
    }

    #[test]
    fn tree_renderer_can_focus_and_reroot() {
        let graph = sample_graph();
        let focused = graph
            .to_tree_text_with_options(&DefinitionRenderOptions {
                compact: DefinitionCompactMode::Basic,
                focus: vec![String::from("Apply<OpWhile")],
                root_node: None,
                show_sources: false,
            })
            .expect("focused tree should render");
        assert!(focused.contains("type Fib<N>"));
        assert!(focused.contains("Apply<OpWhile"));

        let rerooted = graph
            .to_tree_text_with_options(&DefinitionRenderOptions {
                root_node: Some(NodeId::new(2)),
                show_sources: false,
                ..DefinitionRenderOptions::default()
            })
            .expect("rerooted tree should render");
        assert!(!rerooted.contains("type_alias"));
        assert!(rerooted.contains("projection"));
    }

    #[test]
    fn tree_renderer_marks_repeats_cycles_and_max_depth() {
        let graph = DefinitionGraph {
            roots: vec![NodeId::new(1)],
            nodes: vec![
                DefinitionNode {
                    id: NodeId::new(1),
                    kind: DefinitionNodeKind::TypeAlias,
                    label: String::from("type A"),
                    def_path: Some(String::from("fixture::A")),
                    span_id: None,
                    source: None,
                    metadata: BTreeMap::from([(
                        String::from("definition_key"),
                        String::from("alias:fixture::A:"),
                    )]),
                },
                DefinitionNode {
                    id: NodeId::new(2),
                    kind: DefinitionNodeKind::Cycle,
                    label: String::from("alias:fixture::A:"),
                    def_path: None,
                    span_id: None,
                    source: None,
                    metadata: BTreeMap::from([
                        (String::from("reason"), String::from("already_expanded")),
                        (String::from("cycle_key"), String::from("alias:fixture::A:")),
                    ]),
                },
                DefinitionNode {
                    id: NodeId::new(3),
                    kind: DefinitionNodeKind::Unresolved,
                    label: String::from("max depth reached (1)"),
                    def_path: None,
                    span_id: None,
                    source: None,
                    metadata: BTreeMap::from([(
                        String::from("reason"),
                        String::from("max_depth"),
                    )]),
                },
            ],
            edges: vec![
                DefinitionEdge {
                    from: NodeId::new(1),
                    to: NodeId::new(2),
                    kind: DefinitionEdgeKind::References,
                    label: String::from("already expanded"),
                    metadata: BTreeMap::new(),
                },
                DefinitionEdge {
                    from: NodeId::new(1),
                    to: NodeId::new(3),
                    kind: DefinitionEdgeKind::References,
                    label: String::from("max_depth"),
                    metadata: BTreeMap::new(),
                },
            ],
        };
        let rendered = graph.to_tree_text();
        assert!(rendered.contains("repeated_nodes=1"));
        assert!(rendered.contains("max_depth_leaves=1"));
        assert!(rendered.contains("already_expanded -> node#1"));
        assert!(rendered.contains("reason=max_depth"));
    }
}
