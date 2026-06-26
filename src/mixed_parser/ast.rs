use crate::lexer::{ErbBlockKind, ErbBranchKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    HtmlText(String),
    HtmlElement {
        name: String,
        open: String,
        close: String,
        children: Vec<Node>,
    },
    HtmlSelfClosing {
        name: String,
        raw: String,
    },
    HtmlVoid {
        name: String,
        raw: String,
    },
    HtmlComment(String),
    HtmlDoctype(String),
    ErbCode(String),
    ErbComment(String),
    ErbOutput(String),
    ErbBlock {
        kind: ErbBlockKind,
        code: String,
        output: bool,
        children: Vec<Node>,
        branches: Vec<ErbBranch>,
    },
    Spanned {
        node: Box<Node>,
        range: SourceRange,
    },
}

impl Node {
    pub fn unspanned(&self) -> &Self {
        match self {
            Self::Spanned { node, .. } => node.unspanned(),
            node => node,
        }
    }

    pub fn source_range(&self) -> Option<SourceRange> {
        match self {
            Self::Spanned { range, .. } => Some(*range),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErbBranch {
    pub kind: ErbBranchKind,
    pub code: String,
    pub children: Vec<Node>,
}
