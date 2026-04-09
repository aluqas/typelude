use crate::HookId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisConfig {
    pub enabled: Vec<HookId>,
    pub focus: Option<String>,
    pub subject_filter: Option<String>,
    pub max_events: usize,
    pub max_depth: usize,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            enabled: default_hooks_from_env(),
            focus: std::env::var("TYPELUDE_TOOLING_FOCUS").ok(),
            subject_filter: std::env::var("TYPELUDE_TOOLING_SUBJECT_FILTER").ok(),
            max_events: parse_env_usize("TYPELUDE_TOOLING_MAX_EVENTS", 50_000),
            max_depth: parse_env_usize("TYPELUDE_TOOLING_MAX_DEPTH", 32),
        }
    }
}

fn default_hooks_from_env() -> Vec<HookId> {
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

fn parse_env_usize(name: &str, default: usize) -> usize {
    std::env::var(name).ok().and_then(|value| value.parse::<usize>().ok()).unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::AnalysisConfig;
    use crate::HookId;

    #[test]
    fn default_config_has_all_primary_hooks() {
        let config = AnalysisConfig::default();
        assert!(config.enabled.contains(&HookId::TraitSolve));
        assert!(config.enabled.contains(&HookId::ItemStructure));
        assert!(config.enabled.contains(&HookId::Diagnostics));
    }
}
