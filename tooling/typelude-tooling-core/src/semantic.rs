use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticTag {
    EvalLike,
    ApplyLike,
    BranchLike,
    LoopLike,
    LookupLike,
    MapLike,
    HelperDispatchLike,
    VmOpLike,
    Unknown,
}

impl SemanticTag {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::EvalLike => "eval_like",
            Self::ApplyLike => "apply_like",
            Self::BranchLike => "branch_like",
            Self::LoopLike => "loop_like",
            Self::LookupLike => "lookup_like",
            Self::MapLike => "map_like",
            Self::HelperDispatchLike => "helper_dispatch_like",
            Self::VmOpLike => "vm_op_like",
            Self::Unknown => "unknown",
        }
    }
}
