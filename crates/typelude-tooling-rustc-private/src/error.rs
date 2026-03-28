use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisError {
    message: String,
}

impl AnalysisError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub type AnalysisResult<T> = Result<T, AnalysisError>;

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for AnalysisError {}

impl From<String> for AnalysisError {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for AnalysisError {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
