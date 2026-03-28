use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectFilter {
    needle: Option<String>,
}

impl SubjectFilter {
    #[must_use]
    pub fn new(needle: Option<String>) -> Self {
        Self {
            needle: needle.map(|value| value.to_ascii_lowercase()),
        }
    }

    #[must_use]
    pub fn matches(&self, label: &str, metadata: &BTreeMap<String, String>) -> bool {
        self.needle
            .as_ref()
            .map(|needle| {
                label.to_ascii_lowercase().contains(needle)
                    || metadata.values().any(|value| value.to_ascii_lowercase().contains(needle))
            })
            .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::SubjectFilter;

    #[test]
    fn subject_filter_matches_label_and_metadata() {
        let filter = SubjectFilter::new(Some(String::from("runwriter")));
        let mut metadata = BTreeMap::new();
        metadata.insert(String::from("owner_path"), String::from("core::writer_t::RunWriter"));
        assert!(filter.matches("unrelated", &metadata));
        assert!(filter.matches("RunWriter", &BTreeMap::new()));
        assert!(!filter.matches("StateT", &BTreeMap::new()));
    }
}
