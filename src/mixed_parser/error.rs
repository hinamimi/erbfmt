use std::fmt;

use crate::lexer::SourceLocation;

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

impl fmt::Display for LocatedParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(location) = self.location else {
            return write!(f, "{}", self.error);
        };

        match &self.error {
            ParseError::UnexpectedErbBlockEnd { code, .. } => {
                write!(f, "unexpected ERB block end `{code}` at {location}")
            }
            ParseError::UnexpectedErbBranch { code, .. } => {
                write!(f, "unexpected ERB branch `{code}` at {location}")
            }
            ParseError::UnclosedErbBlock { code, .. } => {
                write!(f, "unclosed ERB block `{code}` at {location}")
            }
            ParseError::UnexpectedHtmlCloseTag { .. }
            | ParseError::MismatchedHtmlCloseTag { .. }
            | ParseError::UnclosedHtmlTag { .. } => write!(f, "{} at {location}", self.error),
        }
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
