//! `InspectGoal` / `InspectCandidate` など new solver の inspect ツリーから
//! `PredicateRepr`・`CandidateKind`・`GoalResult`・セマンティックタグへ変換する。

use std::collections::BTreeMap;

use rustc_trait_selection::solve::inspect::{InspectCandidate, InspectGoal};
use typelude_tooling_core::{
    CandidateKind, GoalResult, PredicateRepr, SemanticTag,
};

/// フォーカスフィルタやデバッグ用のゴール述語の `Debug` 文字列。
pub fn inspect_goal_predicate_debug<'tcx>(goal: &InspectGoal<'_, 'tcx>) -> String {
    format!("{:?}", goal.goal().predicate)
}

pub fn inspect_goal_predicate_repr<'tcx>(goal: &InspectGoal<'_, 'tcx>) -> PredicateRepr {
    lower_predicate_repr(&inspect_goal_predicate_debug(goal))
}

pub fn inspect_goal_result_debug<'tcx>(goal: &InspectGoal<'_, 'tcx>) -> String {
    format!("{:?}", goal.result())
}

pub fn inspect_candidate_kind_debug<'tcx>(candidate: &InspectCandidate<'_, 'tcx>) -> String {
    format!("{:?}", candidate.kind())
}

pub fn inspect_candidate_result_debug<'tcx>(candidate: &InspectCandidate<'_, 'tcx>) -> String {
    format!("{:?}", candidate.result())
}

/// `raw_candidate_kind` メタデータ付きで候補種別を下位化する。
pub fn candidate_kind_with_metadata<'tcx>(
    candidate: &InspectCandidate<'_, 'tcx>,
) -> (CandidateKind, BTreeMap<String, String>) {
    let raw = inspect_candidate_kind_debug(candidate);
    let kind = lower_candidate_kind(&raw);
    let metadata = BTreeMap::from([(String::from("raw_candidate_kind"), raw)]);
    (kind, metadata)
}

pub fn lower_goal_result_from_candidate<'tcx>(
    candidate: &InspectCandidate<'_, 'tcx>,
) -> GoalResult {
    lower_goal_result(&inspect_candidate_result_debug(candidate))
}

pub fn lower_goal_result_from_goal<'tcx>(goal: &InspectGoal<'_, 'tcx>) -> GoalResult {
    lower_goal_result(&inspect_goal_result_debug(goal))
}

pub fn lower_predicate_repr(raw: &str) -> PredicateRepr {
    let text = raw.trim();
    if text.contains("AliasRelate(") {
        let inner = text
            .split_once("AliasRelate(")
            .and_then(|(_, rest)| rest.rsplit_once(')'))
            .map(|(body, _)| body)
            .unwrap_or_default();
        if let Some((lhs, rhs)) = split_top_level_once(inner) {
            return PredicateRepr::AliasRelate {
                lhs: lhs.to_string(),
                rhs: rhs.to_string(),
            };
        }
    }
    if text.contains("NormalizesTo(") {
        let inner = text
            .split_once("NormalizesTo(")
            .and_then(|(_, rest)| rest.rsplit_once(')'))
            .map(|(body, _)| body)
            .unwrap_or_default();
        if let Some((alias, target)) = split_top_level_once(inner) {
            return PredicateRepr::NormalizesTo {
                alias: alias.to_string(),
                target: target.to_string(),
            };
        }
    }
    if text.contains("TraitPredicate(") {
        let trait_path = if let Some(start) = text.find("TraitPredicate(") {
            let inner = &text[start + "TraitPredicate(".len()..];
            inner.split_once(')').map_or(inner, |(body, _)| body).to_string()
        } else {
            text.to_string()
        };
        return PredicateRepr::TraitPredicate {
            trait_path,
            self_ty: String::from("<unknown>"),
            args: Vec::new(),
        };
    }
    if text.is_empty() {
        PredicateRepr::Unknown
    } else {
        PredicateRepr::DebugText(text.to_string())
    }
}

pub fn lower_candidate_kind(raw: &str) -> CandidateKind {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("paramenv") {
        CandidateKind::ParamEnv
    } else if lower.contains("impl") {
        CandidateKind::Impl
    } else if lower.contains("builtin") {
        CandidateKind::Builtin
    } else if lower.contains("aliasrelate") {
        CandidateKind::AliasRelate
    } else if lower.contains("normalize") {
        CandidateKind::Normalize
    } else {
        CandidateKind::Unknown(raw.to_string())
    }
}

pub fn lower_goal_result(raw: &str) -> GoalResult {
    let lower = raw.to_ascii_lowercase();
    if lower.contains("no_solution") || lower.contains("nosolution") {
        GoalResult::NoSolution
    } else if lower.contains("ambiguous") {
        GoalResult::Ambiguous
    } else if lower.contains("unsupported") {
        GoalResult::Unsupported
    } else if lower.contains("ok") || lower.contains("yes") {
        GoalResult::Success
    } else {
        GoalResult::Error
    }
}

pub fn semantic_tags_for_predicate(predicate: &PredicateRepr) -> Vec<SemanticTag> {
    let text = predicate.debug_text();
    let mut tags = Vec::new();
    if text.contains("EIf") {
        tags.push(SemanticTag::BranchLike);
    }
    if text.contains("EWhile") {
        tags.push(SemanticTag::LoopLike);
    }
    if text.contains("EGet") {
        tags.push(SemanticTag::LookupLike);
    }
    if text.contains("EMap") {
        tags.push(SemanticTag::MapLike);
    }
    if text.contains("EApp") {
        tags.push(SemanticTag::ApplyLike);
    }
    if text.contains("Helper") {
        tags.push(SemanticTag::HelperDispatchLike);
    }
    if text.contains("Op") {
        tags.push(SemanticTag::VmOpLike);
    }
    if tags.is_empty() {
        tags.push(SemanticTag::EvalLike);
    }
    tags
}

pub fn semantic_tags_for_candidate(candidate_kind: &CandidateKind) -> Vec<SemanticTag> {
    match candidate_kind {
        CandidateKind::AliasRelate => vec![SemanticTag::HelperDispatchLike],
        CandidateKind::Normalize => vec![SemanticTag::EvalLike],
        CandidateKind::ParamEnv | CandidateKind::Impl | CandidateKind::Builtin => {
            vec![SemanticTag::EvalLike]
        },
        CandidateKind::Unknown(_) => vec![SemanticTag::Unknown],
    }
}

fn split_top_level_once(input: &str) -> Option<(&str, &str)> {
    let mut depth = 0_i32;
    for (index, ch) in input.char_indices() {
        match ch {
            '<' | '(' | '[' | '{' => depth += 1,
            '>' | ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                let lhs = input[..index].trim();
                let rhs = input[index + 1..].trim();
                return Some((lhs, rhs));
            },
            _ => {},
        }
    }
    None
}
