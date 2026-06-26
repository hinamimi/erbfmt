use crate::{
    html::HtmlTag,
    lexer::{ErbBlockKind, ErbBranchKind, SourceLocation},
};

use super::{ErbBranch, Node, SourceRange};

pub(super) struct Frame {
    pub(super) kind: FrameKind,
    pub(super) children: Vec<Node>,
    pub(super) initial_children: Option<Vec<Node>>,
    pub(super) branches: Vec<ErbBranch>,
    pub(super) active_branch: Option<ErbBranchHeader>,
}

impl Frame {
    pub(super) fn root() -> Self {
        Self {
            kind: FrameKind::Root,
            children: Vec::new(),
            initial_children: None,
            branches: Vec::new(),
            active_branch: None,
        }
    }

    pub(super) fn html(
        tag: HtmlTag,
        location: Option<SourceLocation>,
        range: Option<SourceRange>,
    ) -> Self {
        Self {
            kind: FrameKind::Html {
                name: tag.name,
                raw: tag.raw,
                location,
                range,
            },
            children: Vec::new(),
            initial_children: None,
            branches: Vec::new(),
            active_branch: None,
        }
    }

    pub(super) fn erb(
        kind: ErbBlockKind,
        code: String,
        output: bool,
        token_index: usize,
        range: Option<SourceRange>,
    ) -> Self {
        Self {
            kind: FrameKind::Erb {
                kind,
                code,
                output,
                token_index,
                range,
            },
            children: Vec::new(),
            initial_children: None,
            branches: Vec::new(),
            active_branch: None,
        }
    }

    pub(super) fn start_erb_branch(&mut self, kind: ErbBranchKind, code: String) {
        if let Some(active_branch) = self.active_branch.take() {
            self.branches.push(ErbBranch {
                kind: active_branch.kind,
                code: active_branch.code,
                children: std::mem::take(&mut self.children),
            });
        } else {
            self.initial_children = Some(std::mem::take(&mut self.children));
        }

        self.active_branch = Some(ErbBranchHeader { kind, code });
    }

    pub(super) fn finish_erb_branches(mut self) -> (Vec<Node>, Vec<ErbBranch>) {
        if let Some(active_branch) = self.active_branch.take() {
            self.branches.push(ErbBranch {
                kind: active_branch.kind,
                code: active_branch.code,
                children: std::mem::take(&mut self.children),
            });
        }

        (
            self.initial_children.unwrap_or(self.children),
            self.branches,
        )
    }
}

pub(super) struct ErbBranchHeader {
    kind: ErbBranchKind,
    code: String,
}

pub(super) enum FrameKind {
    Root,
    Html {
        name: String,
        raw: String,
        location: Option<SourceLocation>,
        range: Option<SourceRange>,
    },
    Erb {
        kind: ErbBlockKind,
        code: String,
        output: bool,
        token_index: usize,
        range: Option<SourceRange>,
    },
}
