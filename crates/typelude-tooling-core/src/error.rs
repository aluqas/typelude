use std::{error::Error, fmt, io};

#[derive(Debug)]
pub enum ToolingError {
    Io(io::Error),
    Json(String),
    Parse(String),
    Command(String),
    Unsupported(String),
}

pub type ToolingResult<T> = Result<T, ToolingError>;

impl fmt::Display for ToolingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "io error: {error}"),
            Self::Json(error) => write!(f, "json error: {error}"),
            Self::Parse(error) => write!(f, "parse error: {error}"),
            Self::Command(error) => write!(f, "command error: {error}"),
            Self::Unsupported(error) => write!(f, "unsupported: {error}"),
        }
    }
}

impl Error for ToolingError {}

impl From<io::Error> for ToolingError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for ToolingError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value.to_string())
    }
}
