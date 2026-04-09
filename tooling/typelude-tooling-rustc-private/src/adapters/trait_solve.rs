//! `InspectGoal` / `InspectCandidate` など new solver の inspect ツリーから
//! rustc-private 固有のデバッグ文字列を抽出する薄いアダプタ。

use rustc_trait_selection::solve::inspect::{InspectCandidate, InspectGoal};

/// フォーカスフィルタやデバッグ用のゴール述語の `Debug` 文字列。
pub fn inspect_goal_predicate_debug<'tcx>(goal: &InspectGoal<'_, 'tcx>) -> String {
    format!("{:?}", goal.goal().predicate)
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
