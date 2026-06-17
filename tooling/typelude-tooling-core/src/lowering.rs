use crate::{CandidateKind, GoalResult, PredicateRepr, SemanticTag};

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

#[cfg(test)]
mod tests {
    use super::{
        lower_candidate_kind, lower_goal_result, lower_predicate_repr,
        semantic_tags_for_candidate, semantic_tags_for_predicate,
    };
    use crate::{CandidateKind, GoalResult, PredicateRepr, SemanticTag};

    #[test]
    fn lowers_alias_relate_predicate() {
        let predicate = lower_predicate_repr("AliasRelate(Foo, Bar)");
        assert_eq!(
            predicate,
            PredicateRepr::AliasRelate {
                lhs: String::from("Foo"),
                rhs: String::from("Bar"),
            }
        );
    }

    #[test]
    fn lowers_candidate_kind_from_debug_text() {
        assert_eq!(lower_candidate_kind("ParamEnv"), CandidateKind::ParamEnv);
        assert_eq!(lower_candidate_kind("MyImplCandidate"), CandidateKind::Impl);
    }

    #[test]
    fn lowers_goal_result_from_debug_text() {
        assert_eq!(lower_goal_result("NoSolution"), GoalResult::NoSolution);
        assert_eq!(lower_goal_result("YES"), GoalResult::Success);
    }

    #[test]
    fn semantic_tags_detect_typelude_patterns() {
        let predicate = PredicateRepr::DebugText(String::from("EGet<Arr, Key>"));
        assert!(semantic_tags_for_predicate(&predicate).contains(&SemanticTag::LookupLike));
        assert_eq!(
            semantic_tags_for_candidate(&CandidateKind::AliasRelate),
            vec![SemanticTag::HelperDispatchLike]
        );
    }
}
