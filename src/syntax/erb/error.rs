use std::fmt;

use super::{SourceLocation, source_location};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    position: usize,
    location: SourceLocation,
    message: String,
}

impl LexError {
    pub(super) fn unterminated_erb(input: &str, position: usize) -> Self {
        Self {
            position,
            location: source_location(input, position),
            message: "unterminated ERB tag".to_string(),
        }
    }

    pub(super) fn unsupported_erb_marker(input: &str, position: usize, marker: &str) -> Self {
        Self {
            position,
            location: source_location(input, position),
            message: format!("unsupported ERB marker `{marker}`"),
        }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.message, self.location)
    }
}

impl std::error::Error for LexError {}
