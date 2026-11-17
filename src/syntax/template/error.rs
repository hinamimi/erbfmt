use std::fmt;

use crate::syntax::erb::SourceLocation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedErbBlockEnd {
        token_index: usize,
        code: String,
    },
    UnexpectedErbBranch {
        token_index: usize,
        code: String,
    },
    UnclosedErbBlock {
        token_index: usize,
        code: String,
    },
    UnexpectedHtmlCloseTag {
        name: String,
        raw: String,
        location: Option<SourceLocation>,
    },
    MismatchedHtmlCloseTag {
        expected: String,
        found: String,
        raw: String,
        location: Option<SourceLocation>,
    },
    UnclosedHtmlTag {
        name: String,
        raw: String,
        location: Option<SourceLocation>,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedErbBlockEnd { token_index, code } => {
                write!(
                    f,
                    "unexpected ERB block end `{code}` at token {token_index}"
                )
            }
            Self::UnexpectedErbBranch { token_index, code } => {
                write!(f, "unexpected ERB branch `{code}` at token {token_index}")
            }
            Self::UnclosedErbBlock { token_index, code } => {
                write!(f, "unclosed ERB block `{code}` at token {token_index}")
            }
            Self::UnexpectedHtmlCloseTag { raw, .. } => {
                write!(f, "unexpected HTML close tag `{raw}`")
            }
            Self::MismatchedHtmlCloseTag { expected, raw, .. } => {
                write!(
                    f,
                    "mismatched HTML close tag `{raw}`, expected `</{expected}>`"
                )
            }
            Self::UnclosedHtmlTag { raw, .. } => write!(f, "unclosed HTML tag `{raw}`"),
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedParseError {
    pub(super) error: ParseError,
    pub(super) location: Option<SourceLocation>,
}

impl LocatedParseError {
    pub fn location(&self) -> Option<SourceLocation> {
        self.location
    }
    pub fn message(&self) -> String {
        match &self.error {
            ParseError::UnexpectedErbBlockEnd { code, .. } => {
                format!("unexpected ERB block end `{code}`")
            }
            ParseError::UnexpectedErbBranch { code, .. } => {
                format!("unexpected ERB branch `{code}`")
            }
            ParseError::UnclosedErbBlock { code, .. } => format!("unclosed ERB block `{code}`"),
            _ => self.error.to_string(),
        }
    }
}

impl fmt::Display for LocatedParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())?;
        if let Some(location) = self.location {
            write!(f, " at {location}")?;
        }
        Ok(())
    }
}

impl std::error::Error for LocatedParseError {}

impl ParseError {
    pub(super) fn token_index(&self) -> Option<usize> {
        match self {
            Self::UnexpectedErbBlockEnd { token_index, .. }
            | Self::UnexpectedErbBranch { token_index, .. }
            | Self::UnclosedErbBlock { token_index, .. } => Some(*token_index),
            Self::UnexpectedHtmlCloseTag { .. }
            | Self::MismatchedHtmlCloseTag { .. }
            | Self::UnclosedHtmlTag { .. } => None,
        }
    }

    pub(super) fn html_location(&self) -> Option<SourceLocation> {
        match self {
            Self::UnexpectedHtmlCloseTag { location, .. }
            | Self::MismatchedHtmlCloseTag { location, .. }
            | Self::UnclosedHtmlTag { location, .. } => *location,
            Self::UnexpectedErbBlockEnd { .. }
            | Self::UnexpectedErbBranch { .. }
            | Self::UnclosedErbBlock { .. } => None,
        }
    }
}

pub(super) fn located_parse_error(
    error: ParseError,
    location: Option<SourceLocation>,
) -> LocatedParseError {
    LocatedParseError { error, location }
}
