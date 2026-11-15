use crate::syntax::erb::SourceLocation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub location: Option<SourceLocation>,
    pub severity: DiagnosticSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Info,
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

    pub fn rule_id(&self) -> Option<&'static str> {
        if self.message.starts_with("empty ERB branch") {
            Some("emptyErbBranch")
        } else if self.message.starts_with("empty ERB code tag")
            || self.message.starts_with("empty ERB output tag")
        {
            Some("emptyErbCodeTag")
        } else if self.message.starts_with("empty ERB control block") {
            Some("emptyErbControlBlock")
        } else if self.message.starts_with("deprecated HTML tag") {
            Some("noDeprecatedHtmlTag")
        } else if self.message.starts_with("deprecated HTML attribute") {
            Some("noDeprecatedHtmlAttribute")
        } else if self.message.starts_with("duplicate HTML attribute") {
            Some("noDuplicateHtmlAttribute")
        } else if self
            .message
            .starts_with("invalid HTML boolean attribute value")
            || self
                .message
                .starts_with("redundant HTML boolean attribute value")
        {
            Some("noInvalidHtmlBooleanAttribute")
        } else if self.message.starts_with("invalid HTML nesting") {
            Some("noInvalidHtmlNesting")
        } else if self
            .message
            .starts_with("HTML attribute value must use double quotes")
        {
            Some("noNonDoubleQuotedHtmlAttributeValue")
        } else if self.message.starts_with("self-closing HTML tag") {
            Some("noSelfClosingHtmlTag")
        } else if self.message.starts_with("unknown HTML attribute") {
            Some("noUnknownHtmlAttribute")
        } else if self.message.starts_with("unsupported ERB block starter") {
            Some("unsupportedErbBlockStarter")
        } else {
            None
        }
    }

    pub fn is_error(&self) -> bool {
        self.severity == DiagnosticSeverity::Error
    }
}
