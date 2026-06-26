#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Html(String),
    ErbCode(String),
    ErbComment(String),
    ErbOutput(String),
    ErbBlockStart {
        kind: ErbBlockKind,
        code: String,
        output: bool,
    },
    ErbBranch {
        kind: ErbBranchKind,
        code: String,
    },
    ErbBlockEnd(String),
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
