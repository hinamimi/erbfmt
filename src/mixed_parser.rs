use std::fmt;

use crate::{
    html::{self, HtmlTag, HtmlToken},
    lexer::{self, ErbBlockKind, ErbBranchKind, SourceLocation, SpannedToken, Token},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub children: Vec<Node>,
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
    ErbOutput(String),
    ErbBlock {
        kind: ErbBlockKind,
        code: String,
        output: bool,
        children: Vec<Node>,
        branches: Vec<ErbBranch>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErbBranch {
    pub kind: ErbBranchKind,
    pub code: String,
    pub children: Vec<Node>,
}

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
    error: ParseError,
    location: Option<SourceLocation>,
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
    fn token_index(&self) -> Option<usize> {
        match self {
            Self::UnexpectedErbBlockEnd { token_index, .. }
            | Self::UnexpectedErbBranch { token_index, .. }
            | Self::UnclosedErbBlock { token_index, .. } => Some(*token_index),
            Self::UnexpectedHtmlCloseTag { .. }
            | Self::MismatchedHtmlCloseTag { .. }
            | Self::UnclosedHtmlTag { .. } => None,
        }
    }

    fn html_location(&self) -> Option<SourceLocation> {
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

#[allow(dead_code)]
pub fn parse(tokens: &[Token]) -> Result<Document, ParseError> {
    let mut parser = Parser {
        stack: vec![Frame::root()],
    };

    for (token_index, token) in tokens.iter().enumerate() {
        parser.parse_token(token_index, token)?;
    }

    parser.finish()
}

pub fn parse_spanned(tokens: &[SpannedToken]) -> Result<Document, LocatedParseError> {
    let mut parser = Parser {
        stack: vec![Frame::root()],
    };

    for (token_index, spanned) in tokens.iter().enumerate() {
        parser
            .parse_spanned_token(token_index, spanned)
            .map_err(|error| located_parse_error(error, tokens))?;
    }

    parser
        .finish()
        .map_err(|error| located_parse_error(error, tokens))
}

fn located_parse_error(error: ParseError, tokens: &[SpannedToken]) -> LocatedParseError {
    let location = error.html_location().or_else(|| {
        error
            .token_index()
            .and_then(|token_index| tokens.get(token_index))
            .map(|spanned| spanned.span.location)
    });

    LocatedParseError { error, location }
}

fn relative_source_location(
    base: SourceLocation,
    fragment: &str,
    position: usize,
) -> SourceLocation {
    let relative = lexer::source_location(fragment, position);

    if relative.line == 1 {
        SourceLocation {
            line: base.line,
            column: base.column + relative.column - 1,
        }
    } else {
        SourceLocation {
            line: base.line + relative.line - 1,
            column: relative.column,
        }
    }
}

struct Parser {
    stack: Vec<Frame>,
}

impl Parser {
    fn parse_token(&mut self, token_index: usize, token: &Token) -> Result<(), ParseError> {
        match token {
            Token::Html(fragment) => self.parse_html_fragment(fragment),
            Token::ErbCode(code) => {
                self.push_node(Node::ErbCode(code.clone()));
                Ok(())
            }
            Token::ErbOutput(code) => {
                self.push_node(Node::ErbOutput(code.clone()));
                Ok(())
            }
            Token::ErbBlockStart { kind, code, output } => {
                self.stack
                    .push(Frame::erb(*kind, code.clone(), *output, token_index));
                Ok(())
            }
            Token::ErbBranch { kind, code } => self.add_erb_branch(token_index, *kind, code),
            Token::ErbBlockEnd(code) => self.close_erb_block(token_index, code),
        }
    }

    fn parse_spanned_token(
        &mut self,
        token_index: usize,
        spanned: &SpannedToken,
    ) -> Result<(), ParseError> {
        match &spanned.token {
            Token::Html(fragment) => {
                self.parse_spanned_html_fragment(fragment, spanned.span.location)
            }
            token => self.parse_token(token_index, token),
        }
    }

    fn parse_html_fragment(&mut self, fragment: &str) -> Result<(), ParseError> {
        for token in html::tokenize(fragment) {
            match token {
                HtmlToken::Text(text) => self.push_node(Node::HtmlText(text)),
                HtmlToken::OpenTag(tag) => self.stack.push(Frame::html(tag, None)),
                HtmlToken::CloseTag(tag) => self.close_html_tag(tag, None)?,
                HtmlToken::SelfClosingTag(tag) => {
                    self.push_node(Node::HtmlSelfClosing {
                        name: tag.name,
                        raw: tag.raw,
                    });
                }
                HtmlToken::VoidTag(tag) => {
                    self.push_node(Node::HtmlVoid {
                        name: tag.name,
                        raw: tag.raw,
                    });
                }
                HtmlToken::Comment(comment) => self.push_node(Node::HtmlComment(comment)),
                HtmlToken::Doctype(doctype) => self.push_node(Node::HtmlDoctype(doctype)),
            }
        }

        Ok(())
    }

    fn parse_spanned_html_fragment(
        &mut self,
        fragment: &str,
        fragment_location: SourceLocation,
    ) -> Result<(), ParseError> {
        for spanned in html::tokenize_with_spans(fragment) {
            let location = Some(relative_source_location(
                fragment_location,
                fragment,
                spanned.span.start,
            ));

            match spanned.token {
                HtmlToken::Text(text) => self.push_node(Node::HtmlText(text)),
                HtmlToken::OpenTag(tag) => self.stack.push(Frame::html(tag, location)),
                HtmlToken::CloseTag(tag) => self.close_html_tag(tag, location)?,
                HtmlToken::SelfClosingTag(tag) => {
                    self.push_node(Node::HtmlSelfClosing {
                        name: tag.name,
                        raw: tag.raw,
                    });
                }
                HtmlToken::VoidTag(tag) => {
                    self.push_node(Node::HtmlVoid {
                        name: tag.name,
                        raw: tag.raw,
                    });
                }
                HtmlToken::Comment(comment) => self.push_node(Node::HtmlComment(comment)),
                HtmlToken::Doctype(doctype) => self.push_node(Node::HtmlDoctype(doctype)),
            }
        }

        Ok(())
    }

    fn close_html_tag(
        &mut self,
        tag: HtmlTag,
        location: Option<SourceLocation>,
    ) -> Result<(), ParseError> {
        let Some(frame) = self.stack.pop() else {
            return Err(ParseError::UnexpectedHtmlCloseTag {
                name: tag.name,
                raw: tag.raw,
                location,
            });
        };

        match frame.kind {
            FrameKind::Root => {
                self.stack.push(frame);
                Err(ParseError::UnexpectedHtmlCloseTag {
                    name: tag.name,
                    raw: tag.raw,
                    location,
                })
            }
            FrameKind::Erb {
                kind,
                token_index,
                code,
                output,
                ..
            } => {
                self.stack.push(Frame {
                    kind: FrameKind::Erb {
                        kind,
                        code: code.clone(),
                        output,
                        token_index,
                    },
                    children: frame.children,
                    initial_children: frame.initial_children,
                    branches: frame.branches,
                    active_branch: frame.active_branch,
                });
                Err(ParseError::UnexpectedHtmlCloseTag {
                    name: tag.name,
                    raw: tag.raw,
                    location,
                })
            }
            FrameKind::Html {
                name,
                raw,
                location: open_location,
            } => {
                if !name.eq_ignore_ascii_case(&tag.name) {
                    self.stack.push(Frame {
                        kind: FrameKind::Html {
                            name: name.clone(),
                            raw,
                            location: open_location,
                        },
                        children: frame.children,
                        initial_children: frame.initial_children,
                        branches: frame.branches,
                        active_branch: frame.active_branch,
                    });
                    return Err(ParseError::MismatchedHtmlCloseTag {
                        expected: name,
                        found: tag.name,
                        raw: tag.raw,
                        location,
                    });
                }

                self.push_node(Node::HtmlElement {
                    name,
                    open: raw,
                    close: tag.raw,
                    children: frame.children,
                });
                Ok(())
            }
        }
    }

    fn close_erb_block(&mut self, token_index: usize, code: &str) -> Result<(), ParseError> {
        let Some(frame) = self.stack.pop() else {
            return Err(ParseError::UnexpectedErbBlockEnd {
                token_index,
                code: code.to_string(),
            });
        };

        match frame.kind {
            FrameKind::Root => {
                self.stack.push(frame);
                Err(ParseError::UnexpectedErbBlockEnd {
                    token_index,
                    code: code.to_string(),
                })
            }
            FrameKind::Html {
                name,
                raw,
                location,
            } => {
                self.stack.push(Frame {
                    kind: FrameKind::Html {
                        name: name.clone(),
                        raw: raw.clone(),
                        location,
                    },
                    children: frame.children,
                    initial_children: frame.initial_children,
                    branches: frame.branches,
                    active_branch: frame.active_branch,
                });
                Err(ParseError::UnclosedHtmlTag {
                    name,
                    raw,
                    location,
                })
            }
            FrameKind::Erb {
                kind,
                ref code,
                output,
                ..
            } => {
                let block_code = code.clone();
                let (children, branches) = frame.finish_erb_branches();
                self.push_node(Node::ErbBlock {
                    kind,
                    code: block_code,
                    output,
                    children,
                    branches,
                });
                Ok(())
            }
        }
    }

    fn add_erb_branch(
        &mut self,
        token_index: usize,
        kind: ErbBranchKind,
        code: &str,
    ) -> Result<(), ParseError> {
        let Some(frame) = self.stack.last_mut() else {
            return Err(ParseError::UnexpectedErbBranch {
                token_index,
                code: code.to_string(),
            });
        };

        match frame.kind {
            FrameKind::Root => Err(ParseError::UnexpectedErbBranch {
                token_index,
                code: code.to_string(),
            }),
            FrameKind::Html {
                ref name,
                ref raw,
                location,
            } => Err(ParseError::UnclosedHtmlTag {
                name: name.clone(),
                raw: raw.clone(),
                location,
            }),
            FrameKind::Erb { .. } => {
                frame.start_erb_branch(kind, code.to_string());
                Ok(())
            }
        }
    }

    fn finish(mut self) -> Result<Document, ParseError> {
        if self.stack.len() == 1 {
            let root = self.stack.pop().expect("root frame exists");
            return Ok(Document {
                children: root.children,
            });
        }

        let frame = self.stack.pop().expect("unclosed frame exists");
        match frame.kind {
            FrameKind::Root => unreachable!("root frame cannot be unclosed"),
            FrameKind::Html {
                name,
                raw,
                location,
            } => Err(ParseError::UnclosedHtmlTag {
                name,
                raw,
                location,
            }),
            FrameKind::Erb {
                token_index, code, ..
            } => Err(ParseError::UnclosedErbBlock { token_index, code }),
        }
    }

    fn push_node(&mut self, node: Node) {
        self.stack
            .last_mut()
            .expect("parser stack always has a root frame")
            .children
            .push(node);
    }
}

struct Frame {
    kind: FrameKind,
    children: Vec<Node>,
    initial_children: Option<Vec<Node>>,
    branches: Vec<ErbBranch>,
    active_branch: Option<ErbBranchHeader>,
}

impl Frame {
    fn root() -> Self {
        Self {
            kind: FrameKind::Root,
            children: Vec::new(),
            initial_children: None,
            branches: Vec::new(),
            active_branch: None,
        }
    }

    fn html(tag: HtmlTag, location: Option<SourceLocation>) -> Self {
        Self {
            kind: FrameKind::Html {
                name: tag.name,
                raw: tag.raw,
                location,
            },
            children: Vec::new(),
            initial_children: None,
            branches: Vec::new(),
            active_branch: None,
        }
    }

    fn erb(kind: ErbBlockKind, code: String, output: bool, token_index: usize) -> Self {
        Self {
            kind: FrameKind::Erb {
                kind,
                code,
                output,
                token_index,
            },
            children: Vec::new(),
            initial_children: None,
            branches: Vec::new(),
            active_branch: None,
        }
    }

    fn start_erb_branch(&mut self, kind: ErbBranchKind, code: String) {
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

    fn finish_erb_branches(mut self) -> (Vec<Node>, Vec<ErbBranch>) {
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

struct ErbBranchHeader {
    kind: ErbBranchKind,
    code: String,
}

enum FrameKind {
    Root,
    Html {
        name: String,
        raw: String,
        location: Option<SourceLocation>,
    },
    Erb {
        kind: ErbBlockKind,
        code: String,
        output: bool,
        token_index: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn parses_nested_html_elements() {
        let tokens = tokenize("<div><p>Hello</p></div>").unwrap();
        let document = parse(&tokens).unwrap();

        assert_eq!(
            document,
            Document {
                children: vec![Node::HtmlElement {
                    name: "div".to_string(),
                    open: "<div>".to_string(),
                    close: "</div>".to_string(),
                    children: vec![Node::HtmlElement {
                        name: "p".to_string(),
                        open: "<p>".to_string(),
                        close: "</p>".to_string(),
                        children: vec![Node::HtmlText("Hello".to_string())]
                    }]
                }]
            }
        );
    }

    #[test]
    fn preserves_inline_erb_output_inside_html() {
        let tokens = tokenize("<p>Hello, <%= user.name %></p>").unwrap();
        let document = parse(&tokens).unwrap();

        assert_eq!(
            document,
            Document {
                children: vec![Node::HtmlElement {
                    name: "p".to_string(),
                    open: "<p>".to_string(),
                    close: "</p>".to_string(),
                    children: vec![
                        Node::HtmlText("Hello, ".to_string()),
                        Node::ErbOutput("user.name".to_string())
                    ]
                }]
            }
        );
    }

    #[test]
    fn preserves_erb_output_inside_html_attributes() {
        let tokens = tokenize(r#"<a href="/users/<%= user.id %>"><%= user.name %></a>"#).unwrap();
        let document = parse(&tokens).unwrap();

        assert_eq!(
            document,
            Document {
                children: vec![Node::HtmlElement {
                    name: "a".to_string(),
                    open: r#"<a href="/users/<%= user.id %>">"#.to_string(),
                    close: "</a>".to_string(),
                    children: vec![Node::ErbOutput("user.name".to_string())]
                }]
            }
        );
    }

    #[test]
    fn keeps_erb_blocks_with_html_children() {
        let tokens = tokenize("<% if user %><ul><li>Hello</li></ul><% end %>").unwrap();
        let document = parse(&tokens).unwrap();

        assert_eq!(
            document,
            Document {
                children: vec![Node::ErbBlock {
                    kind: ErbBlockKind::If,
                    code: "if user".to_string(),
                    output: false,
                    children: vec![Node::HtmlElement {
                        name: "ul".to_string(),
                        open: "<ul>".to_string(),
                        close: "</ul>".to_string(),
                        children: vec![Node::HtmlElement {
                            name: "li".to_string(),
                            open: "<li>".to_string(),
                            close: "</li>".to_string(),
                            children: vec![Node::HtmlText("Hello".to_string())]
                        }]
                    }],
                    branches: vec![]
                }]
            }
        );
    }

    #[test]
    fn keeps_erb_branches_with_children() {
        let tokens =
            tokenize("<% if admin? %><p>Admin</p><% elsif user? %><p>User</p><% else %><p>Guest</p><% end %>")
                .unwrap();
        let document = parse(&tokens).unwrap();

        assert_eq!(
            document,
            Document {
                children: vec![Node::ErbBlock {
                    kind: ErbBlockKind::If,
                    code: "if admin?".to_string(),
                    output: false,
                    children: vec![Node::HtmlElement {
                        name: "p".to_string(),
                        open: "<p>".to_string(),
                        close: "</p>".to_string(),
                        children: vec![Node::HtmlText("Admin".to_string())]
                    }],
                    branches: vec![
                        ErbBranch {
                            kind: ErbBranchKind::Elsif,
                            code: "elsif user?".to_string(),
                            children: vec![Node::HtmlElement {
                                name: "p".to_string(),
                                open: "<p>".to_string(),
                                close: "</p>".to_string(),
                                children: vec![Node::HtmlText("User".to_string())]
                            }]
                        },
                        ErbBranch {
                            kind: ErbBranchKind::Else,
                            code: "else".to_string(),
                            children: vec![Node::HtmlElement {
                                name: "p".to_string(),
                                open: "<p>".to_string(),
                                close: "</p>".to_string(),
                                children: vec![Node::HtmlText("Guest".to_string())]
                            }]
                        }
                    ]
                }]
            }
        );
    }

    #[test]
    fn keeps_case_when_branches() {
        let tokens = tokenize("<% case role %><% when \"admin\" %><p>Admin</p><% when \"user\" %><p>User</p><% end %>")
            .unwrap();
        let document = parse(&tokens).unwrap();

        assert_eq!(
            document,
            Document {
                children: vec![Node::ErbBlock {
                    kind: ErbBlockKind::Case,
                    code: "case role".to_string(),
                    output: false,
                    children: vec![],
                    branches: vec![
                        ErbBranch {
                            kind: ErbBranchKind::When,
                            code: "when \"admin\"".to_string(),
                            children: vec![Node::HtmlElement {
                                name: "p".to_string(),
                                open: "<p>".to_string(),
                                close: "</p>".to_string(),
                                children: vec![Node::HtmlText("Admin".to_string())]
                            }]
                        },
                        ErbBranch {
                            kind: ErbBranchKind::When,
                            code: "when \"user\"".to_string(),
                            children: vec![Node::HtmlElement {
                                name: "p".to_string(),
                                open: "<p>".to_string(),
                                close: "</p>".to_string(),
                                children: vec![Node::HtmlText("User".to_string())]
                            }]
                        }
                    ]
                }]
            }
        );
    }

    #[test]
    fn keeps_output_erb_do_blocks() {
        let tokens =
            tokenize("<%= form_with model: user do |form| %><p>Hello</p><% end %>").unwrap();
        let document = parse(&tokens).unwrap();

        assert_eq!(
            document,
            Document {
                children: vec![Node::ErbBlock {
                    kind: ErbBlockKind::Do,
                    code: "form_with model: user do |form|".to_string(),
                    output: true,
                    children: vec![Node::HtmlElement {
                        name: "p".to_string(),
                        open: "<p>".to_string(),
                        close: "</p>".to_string(),
                        children: vec![Node::HtmlText("Hello".to_string())]
                    }],
                    branches: vec![]
                }]
            }
        );
    }

    #[test]
    fn keeps_begin_rescue_ensure_branches() {
        let tokens = tokenize(
            "<% begin %><p>Saving</p><% rescue => error %><p>Failed</p><% ensure %><p>Done</p><% end %>",
        )
        .unwrap();
        let document = parse(&tokens).unwrap();

        assert_eq!(
            document,
            Document {
                children: vec![Node::ErbBlock {
                    kind: ErbBlockKind::Begin,
                    code: "begin".to_string(),
                    output: false,
                    children: vec![Node::HtmlElement {
                        name: "p".to_string(),
                        open: "<p>".to_string(),
                        close: "</p>".to_string(),
                        children: vec![Node::HtmlText("Saving".to_string())]
                    }],
                    branches: vec![
                        ErbBranch {
                            kind: ErbBranchKind::Rescue,
                            code: "rescue => error".to_string(),
                            children: vec![Node::HtmlElement {
                                name: "p".to_string(),
                                open: "<p>".to_string(),
                                close: "</p>".to_string(),
                                children: vec![Node::HtmlText("Failed".to_string())]
                            }]
                        },
                        ErbBranch {
                            kind: ErbBranchKind::Ensure,
                            code: "ensure".to_string(),
                            children: vec![Node::HtmlElement {
                                name: "p".to_string(),
                                open: "<p>".to_string(),
                                close: "</p>".to_string(),
                                children: vec![Node::HtmlText("Done".to_string())]
                            }]
                        }
                    ]
                }]
            }
        );
    }

    #[test]
    fn preserves_void_comments_and_doctype() {
        let tokens = tokenize("<!DOCTYPE html><!-- hi --><img src=\"x.png\">").unwrap();
        let document = parse(&tokens).unwrap();

        assert_eq!(
            document,
            Document {
                children: vec![
                    Node::HtmlDoctype("<!DOCTYPE html>".to_string()),
                    Node::HtmlComment("<!-- hi -->".to_string()),
                    Node::HtmlVoid {
                        name: "img".to_string(),
                        raw: "<img src=\"x.png\">".to_string()
                    }
                ]
            }
        );
    }

    #[test]
    fn reports_unexpected_html_close_tag() {
        let tokens = tokenize("</div>").unwrap();
        let error = parse(&tokens).unwrap_err();

        assert_eq!(
            error,
            ParseError::UnexpectedHtmlCloseTag {
                name: "div".to_string(),
                raw: "</div>".to_string(),
                location: None
            }
        );
    }

    #[test]
    fn reports_mismatched_html_close_tag() {
        let tokens = tokenize("<div></span>").unwrap();
        let error = parse(&tokens).unwrap_err();

        assert_eq!(
            error,
            ParseError::MismatchedHtmlCloseTag {
                expected: "div".to_string(),
                found: "span".to_string(),
                raw: "</span>".to_string(),
                location: None
            }
        );
    }

    #[test]
    fn reports_unclosed_html_tag() {
        let tokens = tokenize("<div><p>Hello</p>").unwrap();
        let error = parse(&tokens).unwrap_err();

        assert_eq!(
            error,
            ParseError::UnclosedHtmlTag {
                name: "div".to_string(),
                raw: "<div>".to_string(),
                location: None
            }
        );
    }

    #[test]
    fn reports_unclosed_html_before_erb_block_end() {
        let tokens = tokenize("<% if user %><div><% end %>").unwrap();
        let error = parse(&tokens).unwrap_err();

        assert_eq!(
            error,
            ParseError::UnclosedHtmlTag {
                name: "div".to_string(),
                raw: "<div>".to_string(),
                location: None
            }
        );
    }
}
