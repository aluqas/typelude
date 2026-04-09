use typelude_tooling_core::{
    CandidateKind, GoalResult, GoalTreeGoal, GoalTreeSubject, PredicateRepr, RootHotspot,
};

use crate::solve_view::{CompactModeArg, SolveRenderOptions};

pub(crate) fn compact_candidate_kind(raw: &str, mode: CompactModeArg) -> String {
    match mode {
        CompactModeArg::Off => raw.to_string(),
        CompactModeArg::Basic => compact_candidate_kind_basic(raw),
        CompactModeArg::Aggressive => {
            compact_length(&compact_path_segments(&compact_candidate_kind_basic(raw), 3), 120)
        },
    }
}

pub(crate) fn compact_predicate(raw: &str, mode: CompactModeArg) -> String {
    match mode {
        CompactModeArg::Off => raw.to_string(),
        CompactModeArg::Basic => compact_predicate_basic(raw),
        CompactModeArg::Aggressive => compact_length(&compact_predicate_aggressive(raw), 120),
    }
}

pub(crate) fn render_subject(
    subject: &GoalTreeSubject,
    lines: &mut Vec<String>,
    options: SolveRenderOptions,
) {
    let kind = format!("{:?}", subject.kind).to_ascii_lowercase();
    let mut line = format!("subject #{} [{}]", subject.id.value(), kind);
    if let Some(owner_path) = subject.metadata.get("owner_path") {
        line.push_str(&format!(" owner={}", compact_path_segments(owner_path, 4)));
    } else if let Some(path) = subject.metadata.get("path") {
        line.push_str(&format!(" path={}", compact_path_segments(path, 4)));
    }
    if let Some(predicate_index) = subject.metadata.get("predicate_index") {
        line.push_str(&format!(" predicate_index={predicate_index}"));
    }
    if let Some(kind_meta) = subject.metadata.get("kind") {
        line.push_str(&format!(" rustc_kind={kind_meta}"));
    }
    lines.push(line);
    lines.push(format!("  label :: {}", maybe_compact_predicate(&subject.label, options)));
    if let Some(clause) = subject.metadata.get("clause") {
        lines.push(format!("  clause :: {}", maybe_compact_predicate(clause, options)));
    }
}

pub(crate) fn render_root_section(
    roots: &[RootHotspot],
    lines: &mut Vec<String>,
    options: SolveRenderOptions,
) {
    for root in roots {
        lines.push(format!(
            "{} :: goal={} result={} goals={} candidates={} max_depth={} unique_predicates={} unique_candidate_kinds={} dominant_predicate_family={} dominant_candidate_family={}",
            maybe_compact_predicate(
                &format!("{}::{}", root.subject_label, root.predicate),
                options
            ),
            root.goal_id,
            compact_result(&root.result, options.compact),
            root.goal_count,
            root.candidate_count,
            root.max_depth,
            root.unique_predicates,
            root.unique_candidate_kinds,
            root.dominant_predicate_family,
            root.dominant_candidate_family,
        ));
    }
}

pub(crate) fn render_goal(
    goal: &GoalTreeGoal,
    depth: usize,
    lines: &mut Vec<String>,
    options: SolveRenderOptions,
) {
    let indent = "  ".repeat(depth);
    let semantic = semantic_tags_text(&goal.semantic_tags)
        .map(|tags| format!(" semantic={tags}"))
        .unwrap_or_default();
    lines.push(format!(
        "{indent}goal #{} result={} depth={} candidates={}{} :: {}",
        goal.id.value(),
        compact_goal_result(goal.result, options.compact),
        goal.depth,
        goal.candidates.len(),
        semantic,
        maybe_compact_predicate(&predicate_text(&goal.predicate), options)
    ));
    for candidate in &goal.candidates {
        let semantic = semantic_tags_text(&candidate.semantic_tags)
            .map(|tags| format!(" semantic={tags}"))
            .unwrap_or_default();
        let raw = candidate_raw_kind(candidate)
            .map(|raw| format!(" raw={}", compact_candidate_detail(raw, options)))
            .unwrap_or_default();
        lines.push(format!(
            "{indent}  candidate #{} kind={} result={}{}{}",
            candidate.id.value(),
            maybe_compact_candidate_kind(&candidate_kind_text(&candidate.kind), options),
            compact_goal_result(candidate.result, options.compact),
            semantic,
            raw
        ));
    }
    for child in &goal.children {
        render_goal(child, depth + 1, lines, options);
    }
}

pub(crate) fn format_metric_diff(name: &str, left: usize, right: usize, delta: isize) -> String {
    format!("{name}: left={left} right={right} delta={delta:+}")
}

fn maybe_compact_candidate_kind(raw: &str, options: SolveRenderOptions) -> String {
    if options.show_raw_kind {
        raw.to_string()
    } else {
        compact_candidate_kind(raw, options.compact)
    }
}

fn maybe_compact_predicate(raw: &str, options: SolveRenderOptions) -> String {
    if options.show_full_predicate {
        raw.to_string()
    } else {
        compact_predicate(raw, options.compact)
    }
}

fn compact_candidate_kind_basic(raw: &str) -> String {
    typelude_tooling_core::candidate_family(raw)
}

fn compact_predicate_basic(raw: &str) -> String {
    let mut text = raw.trim();
    if let Some(stripped) = text.strip_prefix("Binder { value: ") {
        text = stripped.strip_suffix(", bound_vars: [] }").unwrap_or(stripped);
    }
    let family = typelude_tooling_core::predicate_family(text);
    if family != "Other" {
        return format!("{family}(...)");
    }
    text.to_string()
}

fn compact_predicate_aggressive(raw: &str) -> String {
    let basic = compact_predicate_basic(raw);
    compact_path_segments(&basic, 3)
}

fn compact_result(raw: &str, mode: CompactModeArg) -> String {
    match mode {
        CompactModeArg::Off => raw.to_string(),
        CompactModeArg::Basic => {
            raw.split_once('(').map_or_else(|| raw.to_string(), |(head, _)| format!("{head}(...)"))
        },
        CompactModeArg::Aggressive => raw
            .split_once("::")
            .map_or_else(|| compact_length(raw, 120), |(_, tail)| compact_length(tail, 120)),
    }
}

fn compact_goal_result(result: GoalResult, mode: CompactModeArg) -> String {
    compact_result(result.label(), mode)
}

fn compact_length(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        text.to_string()
    } else {
        format!("{}...", &text[..limit.saturating_sub(3)])
    }
}

fn compact_path_segments(text: &str, keep: usize) -> String {
    let parts = text.split("::").collect::<Vec<_>>();
    if parts.len() > keep {
        parts[parts.len() - keep..].join("::")
    } else {
        text.to_string()
    }
}

fn semantic_tags_text(tags: &[typelude_tooling_core::SemanticTag]) -> Option<String> {
    if tags.is_empty() {
        None
    } else {
        Some(tags.iter().map(|tag| tag.label()).collect::<Vec<_>>().join(","))
    }
}

fn candidate_raw_kind(candidate: &typelude_tooling_core::GoalTreeCandidate) -> Option<&str> {
    candidate.metadata.get("raw_candidate_kind").map(String::as_str)
}

fn compact_candidate_detail(raw: &str, options: SolveRenderOptions) -> String {
    if options.show_raw_kind {
        raw.to_string()
    } else {
        compact_length(&compact_path_segments(raw, 4), 120)
    }
}

fn predicate_text(predicate: &PredicateRepr) -> String {
    predicate.debug_text()
}

fn candidate_kind_text(kind: &CandidateKind) -> String {
    kind.label()
}
