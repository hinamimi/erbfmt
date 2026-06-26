use crate::lexer::SourceLocation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub location: Option<SourceLocation>,
    pub severity: DiagnosticSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Warning,
    Error,
}

impl Diagnostic {
    pub(super) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            location: None,
            severity: DiagnosticSeverity::Error,
        }
    }

    #[cfg(test)]
    pub(super) fn located(message: impl Into<String>, location: SourceLocation) -> Self {
        Self::located_with_severity(message, location, DiagnosticSeverity::Error)
    }

    pub(super) fn located_with_severity(
        message: impl Into<String>,
        location: SourceLocation,
        severity: DiagnosticSeverity,
    ) -> Self {
        Self {
            message: message.into(),
            location: Some(location),
            severity,
        }
    }

    pub fn message_with_location(&self) -> String {
        match self.location {
            Some(location) => format!("{} at {}", self.message, location),
            None => self.message.clone(),
        }
    }

    pub fn is_error(&self) -> bool {
        self.severity == DiagnosticSeverity::Error
    }
}
