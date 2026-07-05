use std::fmt;

use super::Token;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

pub(super) fn spanned_token(input: &str, start: usize, end: usize, token: Token) -> SpannedToken {
    SpannedToken {
        token,
        span: Span {
            start,
            end,
            location: source_location(input, start),
        },
    }
}

pub fn source_location(input: &str, position: usize) -> SourceLocation {
    let mut line = 1;
    let mut column = 1;

    for (index, ch) in input.char_indices() {
        if index >= position {
            break;
        }

        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    SourceLocation { line, column }
}
