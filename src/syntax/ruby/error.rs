use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RubyAstError {
    pub message: String,
    pub offset: usize,
}

impl RubyAstError {
    pub fn new(message: impl Into<String>, offset: usize) -> Self {
        Self {
            message: message.into(),
            offset,
        }
    }
}

impl fmt::Display for RubyAstError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.message, self.offset)
    }
}

impl std::error::Error for RubyAstError {}
