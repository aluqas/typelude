#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusFilter {
    needle: Option<String>,
}

impl FocusFilter {
    #[must_use]
    pub fn new(needle: Option<String>) -> Self {
        Self {
            needle: needle.map(|value| value.to_ascii_lowercase()),
        }
    }

    #[must_use]
    pub fn matches(&self, value: &str) -> bool {
        self.needle
            .as_ref()
            .map(|needle| value.to_ascii_lowercase().contains(needle))
            .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::FocusFilter;

    #[test]
    fn focus_filter_is_case_insensitive() {
        let filter = FocusFilter::new(Some(String::from("runwriter")));
        assert!(filter.matches("RunWriter"));
        assert!(!filter.matches("StateT"));
    }
}
