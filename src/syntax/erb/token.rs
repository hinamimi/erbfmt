#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Html(String),
    ErbCode(ErbTag),
    ErbComment(ErbTag),
    ErbOutput(ErbTag),
    ErbBlockStart {
        kind: ErbBlockKind,
        tag: ErbTag,
        output: bool,
    },
    ErbBranch {
        kind: ErbBranchKind,
        tag: ErbTag,
    },
    ErbBlockEnd(ErbTag),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErbTag {
    pub code: String,
    pub syntax: ErbTagSyntax,
}

impl ErbTag {
    pub fn new(code: String, syntax: ErbTagSyntax) -> Self {
        Self { code, syntax }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErbTagSyntax {
    pub open: ErbTagOpen,
    pub close: ErbTagClose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErbTagOpen {
    Code,
    TrimCode,
    Output,
    RawOutput,
    Comment,
}

impl ErbTagOpen {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Code => "<%",
            Self::TrimCode => "<%-",
            Self::Output => "<%=",
            Self::RawOutput => "<%==",
            Self::Comment => "<%#",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErbTagClose {
    Normal,
    Trim,
}

impl ErbTagClose {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "%>",
            Self::Trim => "-%>",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErbBlockKind {
    If,
    Unless,
    Case,
    Do,
    Begin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErbBranchKind {
    Else,
    Elsif,
    When,
    Rescue,
    Ensure,
}
