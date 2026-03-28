use serde::{Deserialize, Serialize};

macro_rules! define_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub struct $name(pub u64);

        impl $name {
            #[must_use]
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            #[must_use]
            pub const fn value(self) -> u64 {
                self.0
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookId {
    TraitSolve,
    Diagnostics,
    ItemStructure,
    LegacyRustcLog,
}

define_id!(RunId);
define_id!(TraceId);
define_id!(EventId);
define_id!(NodeId);
define_id!(SubjectId);
define_id!(GoalId);
define_id!(CandidateId);
define_id!(DiagId);
define_id!(SpanId);
define_id!(TypeId);

#[cfg(test)]
mod tests {
    use super::{EventId, HookId, RunId, TraceId};

    #[test]
    fn ids_preserve_values() {
        assert_eq!(RunId::new(3).value(), 3);
        assert_eq!(TraceId::new(7).value(), 7);
        assert_eq!(EventId::new(11).value(), 11);
    }

    #[test]
    fn hook_ids_serialize_stably() {
        let json = serde_json::to_string(&HookId::TraitSolve).expect("hook id should serialize");
        assert_eq!(json, "\"trait_solve\"");
    }
}
