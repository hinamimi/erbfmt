use crate::{
    html::{self, HtmlTag, HtmlToken},
    lexer::{self, ErbBranchKind, SourceLocation, SpannedToken, Token},
};

mod ast;
mod error;
mod frame;

pub use ast::{Document, ErbBranch, Node, SourceRange};
pub use error::{LocatedParseError, ParseError};
use frame::{Frame, FrameKind};

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

    error::located_parse_error(error, location)
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
            Token::ErbComment(comment) => {
                self.push_node(Node::ErbComment(comment.clone()));
                Ok(())
            }
            Token::ErbOutput(code) => {
                self.push_node(Node::ErbOutput(code.clone()));
                Ok(())
            }
            Token::ErbBlockStart { kind, code, output } => {
                self.stack
                    .push(Frame::erb(*kind, code.clone(), *output, token_index, None));
                Ok(())
            }
            Token::ErbBranch { kind, code } => self.add_erb_branch(token_index, *kind, code),
            Token::ErbBlockEnd(code) => self.close_erb_block(token_index, code, None),
        }
    }

    fn parse_spanned_token(
        &mut self,
        token_index: usize,
        spanned: &SpannedToken,
    ) -> Result<(), ParseError> {
        let range = SourceRange {
            start: spanned.span.start,
            end: spanned.span.end,
        };

        match &spanned.token {
            Token::Html(fragment) => self.parse_spanned_html_fragment(
                fragment,
                spanned.span.start,
                spanned.span.location,
            ),
            Token::ErbCode(code) => {
                self.push_spanned_node(Node::ErbCode(code.clone()), range);
                Ok(())
            }
            Token::ErbComment(comment) => {
                self.push_spanned_node(Node::ErbComment(comment.clone()), range);
                Ok(())
            }
            Token::ErbOutput(code) => {
                self.push_spanned_node(Node::ErbOutput(code.clone()), range);
                Ok(())
            }
            Token::ErbBlockStart { kind, code, output } => {
                self.stack.push(Frame::erb(
                    *kind,
                    code.clone(),
                    *output,
                    token_index,
                    Some(range),
                ));
                Ok(())
            }
            Token::ErbBranch { kind, code } => self.add_erb_branch(token_index, *kind, code),
            Token::ErbBlockEnd(code) => {
                self.close_erb_block(token_index, code, Some(spanned.span.end))
            }
        }
    }

    fn parse_html_fragment(&mut self, fragment: &str) -> Result<(), ParseError> {
        for token in html::tokenize(fragment) {
            match token {
                HtmlToken::Text(text) => self.push_node(Node::HtmlText(text)),
                HtmlToken::OpenTag(tag) => self.stack.push(Frame::html(tag, None, None)),
                HtmlToken::CloseTag(tag) => self.close_html_tag(tag, None, None)?,
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
        fragment_start: usize,
        fragment_location: SourceLocation,
    ) -> Result<(), ParseError> {
        for spanned in html::tokenize_with_spans(fragment) {
            let range = SourceRange {
                start: fragment_start + spanned.span.start,
                end: fragment_start + spanned.span.end,
            };
            let location = Some(relative_source_location(
                fragment_location,
                fragment,
                spanned.span.start,
            ));

            match spanned.token {
                HtmlToken::Text(text) => self.push_spanned_node(Node::HtmlText(text), range),
                HtmlToken::OpenTag(tag) => self.stack.push(Frame::html(tag, location, Some(range))),
                HtmlToken::CloseTag(tag) => self.close_html_tag(tag, location, Some(range.end))?,
                HtmlToken::SelfClosingTag(tag) => {
                    self.push_spanned_node(
                        Node::HtmlSelfClosing {
                            name: tag.name,
                            raw: tag.raw,
                        },
                        range,
                    );
                }
                HtmlToken::VoidTag(tag) => {
                    self.push_spanned_node(
                        Node::HtmlVoid {
                            name: tag.name,
                            raw: tag.raw,
                        },
                        range,
                    );
                }
                HtmlToken::Comment(comment) => {
                    self.push_spanned_node(Node::HtmlComment(comment), range)
                }
                HtmlToken::Doctype(doctype) => {
                    self.push_spanned_node(Node::HtmlDoctype(doctype), range)
                }
            }
        }

        Ok(())
    }

    fn close_html_tag(
        &mut self,
        tag: HtmlTag,
        location: Option<SourceLocation>,
        end: Option<usize>,
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
                range,
            } => {
                self.stack.push(Frame {
                    kind: FrameKind::Erb {
                        kind,
                        code: code.clone(),
                        output,
                        token_index,
                        range,
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
                range,
            } => {
                if !name.eq_ignore_ascii_case(&tag.name) {
                    self.stack.push(Frame {
                        kind: FrameKind::Html {
                            name: name.clone(),
                            raw,
                            location: open_location,
                            range,
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

                let node = Node::HtmlElement {
                    name,
                    open: raw,
                    close: tag.raw,
                    children: frame.children,
                };
                self.push_node(wrap_spanned_node(node, range, end));
                Ok(())
            }
        }
    }

    fn close_erb_block(
        &mut self,
        token_index: usize,
        code: &str,
        end: Option<usize>,
    ) -> Result<(), ParseError> {
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
                range,
            } => {
                self.stack.push(Frame {
                    kind: FrameKind::Html {
                        name: name.clone(),
                        raw: raw.clone(),
                        location,
                        range,
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
                range,
                ..
            } => {
                let block_code = code.clone();
                let (children, branches) = frame.finish_erb_branches();
                let node = Node::ErbBlock {
                    kind,
                    code: block_code,
                    output,
                    children,
                    branches,
                };
                self.push_node(wrap_spanned_node(node, range, end));
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
                ..
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
                ..
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

    fn push_spanned_node(&mut self, node: Node, range: SourceRange) {
        self.push_node(Node::Spanned {
            node: Box::new(node),
            range,
        });
    }
}

fn wrap_spanned_node(node: Node, start: Option<SourceRange>, end: Option<usize>) -> Node {
    match (start, end) {
        (Some(start), Some(end)) => Node::Spanned {
            node: Box::new(node),
            range: SourceRange {
                start: start.start,
                end,
            },
        },
        _ => node,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{ErbBlockKind, tokenize, tokenize_with_spans};

    #[test]
    fn keeps_absolute_ranges_for_spanned_html_subtrees() {
        let input = "<section>\n  <p>Hello</p>\n</section>\n";
        let tokens = tokenize_with_spans(input).unwrap();
        let document = parse_spanned(&tokens).unwrap();
        let section = &document.children[0];

        assert_eq!(
            section.source_range(),
            Some(SourceRange { start: 0, end: 35 })
        );

        let Node::HtmlElement { children, .. } = section.unspanned() else {
            panic!("section element expected");
        };
        let paragraph = children
            .iter()
            .find(|node| matches!(node.unspanned(), Node::HtmlElement { .. }))
            .expect("paragraph element exists");

        assert_eq!(
            paragraph.source_range(),
            Some(SourceRange { start: 12, end: 24 })
        );
    }

    #[test]
    fn keeps_absolute_ranges_for_spanned_erb_blocks_and_comments() {
        let input = "<%# note %>\n<% if user %>\n<p>Hello</p>\n<% end %>\n";
        let tokens = tokenize_with_spans(input).unwrap();
        let document = parse_spanned(&tokens).unwrap();
        let comment = &document.children[0];
        let block = document
            .children
            .iter()
            .find(|node| matches!(node.unspanned(), Node::ErbBlock { .. }))
            .expect("ERB block exists");

        assert_eq!(
            comment.source_range(),
            Some(SourceRange { start: 0, end: 11 })
        );
        assert!(matches!(comment.unspanned(), Node::ErbComment(comment) if comment == "note"));
        assert_eq!(
            block.source_range(),
            Some(SourceRange { start: 12, end: 48 })
        );
    }

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
