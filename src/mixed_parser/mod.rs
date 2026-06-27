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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ParserOptions {
    pub allow_html_optional_closing_tags: bool,
}

#[allow(dead_code)]
pub fn parse(tokens: &[Token]) -> Result<Document, ParseError> {
    parse_with_options(tokens, ParserOptions::default())
}

pub fn parse_with_options(
    tokens: &[Token],
    options: ParserOptions,
) -> Result<Document, ParseError> {
    let mut parser = Parser {
        stack: vec![Frame::root()],
        options,
        source_end: None,
    };

    for (token_index, token) in tokens.iter().enumerate() {
        parser.parse_token(token_index, token)?;
    }

    parser.finish()
}

#[allow(dead_code)]
pub fn parse_spanned(tokens: &[SpannedToken]) -> Result<Document, LocatedParseError> {
    parse_spanned_with_options(tokens, ParserOptions::default())
}

pub fn parse_spanned_with_options(
    tokens: &[SpannedToken],
    options: ParserOptions,
) -> Result<Document, LocatedParseError> {
    let mut parser = Parser {
        stack: vec![Frame::root()],
        options,
        source_end: None,
    };

    for (token_index, spanned) in tokens.iter().enumerate() {
        parser.source_end = Some(spanned.span.end);
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
    options: ParserOptions,
    source_end: Option<usize>,
}

impl Parser {
    fn parse_token(&mut self, token_index: usize, token: &Token) -> Result<(), ParseError> {
        match token {
            Token::Html(fragment) => self.parse_html_fragment(fragment),
            Token::ErbCode(tag) => {
                self.push_node(Node::ErbCode(tag.clone()));
                Ok(())
            }
            Token::ErbComment(tag) => {
                self.push_node(Node::ErbComment(tag.clone()));
                Ok(())
            }
            Token::ErbOutput(tag) => {
                self.push_node(Node::ErbOutput(tag.clone()));
                Ok(())
            }
            Token::ErbBlockStart { kind, tag, output } => {
                self.stack
                    .push(Frame::erb(*kind, tag.clone(), *output, token_index, None));
                Ok(())
            }
            Token::ErbBranch { kind, tag } => self.add_erb_branch(token_index, *kind, tag),
            Token::ErbBlockEnd(tag) => self.close_erb_block(token_index, tag, None),
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
            Token::ErbCode(tag) => {
                self.push_spanned_node(Node::ErbCode(tag.clone()), range);
                Ok(())
            }
            Token::ErbComment(tag) => {
                self.push_spanned_node(Node::ErbComment(tag.clone()), range);
                Ok(())
            }
            Token::ErbOutput(tag) => {
                self.push_spanned_node(Node::ErbOutput(tag.clone()), range);
                Ok(())
            }
            Token::ErbBlockStart { kind, tag, output } => {
                self.stack.push(Frame::erb(
                    *kind,
                    tag.clone(),
                    *output,
                    token_index,
                    Some(range),
                ));
                Ok(())
            }
            Token::ErbBranch { kind, tag } => self.add_erb_branch(token_index, *kind, tag),
            Token::ErbBlockEnd(tag) => {
                self.close_erb_block(token_index, tag, Some(spanned.span.end))
            }
        }
    }

    fn parse_html_fragment(&mut self, fragment: &str) -> Result<(), ParseError> {
        for token in html::tokenize(fragment) {
            match token {
                HtmlToken::Text(text) => self.push_node(Node::HtmlText(text)),
                HtmlToken::OpenTag(tag) => {
                    self.close_optional_html_tags_before_open(&tag.name, None);
                    self.stack.push(Frame::html(tag, None, None));
                }
                HtmlToken::CloseTag(tag) => self.close_html_tag(tag, None, None, None)?,
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
                HtmlToken::OpenTag(tag) => {
                    self.close_optional_html_tags_before_open(&tag.name, Some(range.start));
                    self.stack.push(Frame::html(tag, location, Some(range)));
                }
                HtmlToken::CloseTag(tag) => {
                    self.close_html_tag(tag, location, Some(range.start), Some(range.end))?
                }
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
        close_start: Option<usize>,
        end: Option<usize>,
    ) -> Result<(), ParseError> {
        self.close_optional_html_tags_before_close(&tag.name, close_start);

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
                tag: erb_tag,
                output,
                range,
            } => {
                self.stack.push(Frame {
                    kind: FrameKind::Erb {
                        kind,
                        tag: erb_tag.clone(),
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
        end_tag: &lexer::ErbTag,
        end: Option<usize>,
    ) -> Result<(), ParseError> {
        let Some(frame) = self.stack.pop() else {
            return Err(ParseError::UnexpectedErbBlockEnd {
                token_index,
                code: end_tag.code.clone(),
            });
        };

        match frame.kind {
            FrameKind::Root => {
                self.stack.push(frame);
                Err(ParseError::UnexpectedErbBlockEnd {
                    token_index,
                    code: end_tag.code.clone(),
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
                ref tag,
                output,
                range,
                ..
            } => {
                let block_tag = tag.clone();
                let (children, branches) = frame.finish_erb_branches();
                let node = Node::ErbBlock {
                    kind,
                    tag: block_tag,
                    output,
                    end_tag: end_tag.clone(),
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
        tag: &lexer::ErbTag,
    ) -> Result<(), ParseError> {
        let Some(frame) = self.stack.last_mut() else {
            return Err(ParseError::UnexpectedErbBranch {
                token_index,
                code: tag.code.clone(),
            });
        };

        match frame.kind {
            FrameKind::Root => Err(ParseError::UnexpectedErbBranch {
                token_index,
                code: tag.code.clone(),
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
                frame.start_erb_branch(kind, tag.clone());
                Ok(())
            }
        }
    }

    fn finish(mut self) -> Result<Document, ParseError> {
        if self.options.allow_html_optional_closing_tags {
            while self
                .current_html_name()
                .is_some_and(optional_html_close_at_eof)
            {
                self.close_optional_html_tag(self.source_end);
            }
        }

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
                token_index, tag, ..
            } => Err(ParseError::UnclosedErbBlock {
                token_index,
                code: tag.code,
            }),
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

    fn close_optional_html_tags_before_open(&mut self, next_name: &str, end: Option<usize>) {
        if !self.options.allow_html_optional_closing_tags {
            return;
        }

        while self
            .current_html_name()
            .is_some_and(|current| optional_html_close_before_open(current, next_name))
        {
            self.close_optional_html_tag(end);
        }
    }

    fn close_optional_html_tags_before_close(&mut self, close_name: &str, end: Option<usize>) {
        if !self.options.allow_html_optional_closing_tags {
            return;
        }

        while self
            .current_html_name()
            .is_some_and(|current| optional_html_close_before_close(current, close_name))
        {
            self.close_optional_html_tag(end);
        }
    }

    fn current_html_name(&self) -> Option<&str> {
        let frame = self.stack.last()?;

        match &frame.kind {
            FrameKind::Html { name, .. } => Some(name),
            FrameKind::Root | FrameKind::Erb { .. } => None,
        }
    }

    fn close_optional_html_tag(&mut self, end: Option<usize>) {
        let frame = self
            .stack
            .pop()
            .expect("optional HTML close requires an HTML frame");

        let FrameKind::Html {
            name, raw, range, ..
        } = frame.kind
        else {
            self.stack.push(frame);
            return;
        };

        let node = Node::HtmlElement {
            name,
            open: raw,
            close: String::new(),
            children: frame.children,
        };
        self.push_node(wrap_spanned_node(node, range, end));
    }
}

pub(crate) fn optional_html_close_before_open(current: &str, next: &str) -> bool {
    let current = current.to_ascii_lowercase();
    let next = next.to_ascii_lowercase();

    match current.as_str() {
        "li" => matches!(next.as_str(), "li"),
        "dt" | "dd" => matches!(next.as_str(), "dt" | "dd"),
        "rt" | "rp" => matches!(next.as_str(), "rt" | "rp"),
        "option" => matches!(next.as_str(), "option" | "optgroup"),
        "optgroup" => matches!(next.as_str(), "optgroup"),
        "tr" => matches!(next.as_str(), "tr"),
        "td" | "th" => matches!(next.as_str(), "td" | "th"),
        "thead" => matches!(next.as_str(), "tbody" | "tfoot"),
        "tbody" => matches!(next.as_str(), "tbody" | "tfoot"),
        "p" => closes_paragraph_before(&next),
        _ => false,
    }
}

pub(crate) fn optional_html_close_before_close(current: &str, closing: &str) -> bool {
    let current = current.to_ascii_lowercase();
    let closing = closing.to_ascii_lowercase();

    match current.as_str() {
        "li" => matches!(closing.as_str(), "ul" | "ol" | "menu"),
        "dt" | "dd" => matches!(closing.as_str(), "dl"),
        "rt" | "rp" => matches!(closing.as_str(), "ruby"),
        "option" => matches!(closing.as_str(), "select" | "datalist" | "optgroup"),
        "optgroup" => matches!(closing.as_str(), "select"),
        "tr" => matches!(closing.as_str(), "table" | "thead" | "tbody" | "tfoot"),
        "td" | "th" => matches!(
            closing.as_str(),
            "tr" | "table" | "thead" | "tbody" | "tfoot"
        ),
        "thead" | "tbody" | "tfoot" => matches!(closing.as_str(), "table"),
        "p" => closes_paragraph_before_close(&closing),
        _ => false,
    }
}

fn optional_html_close_at_eof(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "li" | "p"
            | "dt"
            | "dd"
            | "rt"
            | "rp"
            | "option"
            | "optgroup"
            | "tr"
            | "td"
            | "th"
            | "thead"
            | "tbody"
            | "tfoot"
    )
}

fn closes_paragraph_before(next: &str) -> bool {
    matches!(
        next,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "details"
            | "div"
            | "dl"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hr"
            | "main"
            | "menu"
            | "nav"
            | "ol"
            | "p"
            | "pre"
            | "section"
            | "table"
            | "ul"
    )
}

fn closes_paragraph_before_close(closing: &str) -> bool {
    matches!(
        closing,
        "article"
            | "aside"
            | "blockquote"
            | "body"
            | "details"
            | "dialog"
            | "div"
            | "fieldset"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "header"
            | "html"
            | "li"
            | "main"
            | "nav"
            | "section"
            | "td"
            | "th"
    )
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
mod tests;
