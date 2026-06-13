use std::fmt;

use crate::{
    html::{self, HtmlTag, HtmlToken},
    lexer::{ErbBlockKind, Token},
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
        children: Vec<Node>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedErbBlockEnd {
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
    },
    MismatchedHtmlCloseTag {
        expected: String,
        found: String,
        raw: String,
    },
    UnclosedHtmlTag {
        name: String,
        raw: String,
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
            Self::UnclosedErbBlock { token_index, code } => {
                write!(f, "unclosed ERB block `{code}` at token {token_index}")
            }
            Self::UnexpectedHtmlCloseTag { raw, .. } => {
                write!(f, "unexpected HTML close tag `{raw}`")
            }
            Self::MismatchedHtmlCloseTag {
                expected,
                found,
                raw,
            } => {
                write!(
                    f,
                    "mismatched HTML close tag `{raw}`, expected closing tag for `{expected}`, found `{found}`"
                )
            }
            Self::UnclosedHtmlTag { raw, .. } => write!(f, "unclosed HTML tag `{raw}`"),
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse(tokens: &[Token]) -> Result<Document, ParseError> {
    let mut parser = Parser {
        stack: vec![Frame::root()],
    };

    for (token_index, token) in tokens.iter().enumerate() {
        parser.parse_token(token_index, token)?;
    }

    parser.finish()
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
            Token::ErbBlockStart { kind, code } => {
                self.stack
                    .push(Frame::erb(*kind, code.clone(), token_index));
                Ok(())
            }
            Token::ErbBlockEnd(code) => self.close_erb_block(token_index, code),
        }
    }

    fn parse_html_fragment(&mut self, fragment: &str) -> Result<(), ParseError> {
        for token in html::tokenize(fragment) {
            match token {
                HtmlToken::Text(text) => self.push_node(Node::HtmlText(text)),
                HtmlToken::OpenTag(tag) => self.stack.push(Frame::html(tag)),
                HtmlToken::CloseTag(tag) => self.close_html_tag(tag)?,
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

    fn close_html_tag(&mut self, tag: HtmlTag) -> Result<(), ParseError> {
        let Some(frame) = self.stack.pop() else {
            return Err(ParseError::UnexpectedHtmlCloseTag {
                name: tag.name,
                raw: tag.raw,
            });
        };

        match frame.kind {
            FrameKind::Root => {
                self.stack.push(frame);
                Err(ParseError::UnexpectedHtmlCloseTag {
                    name: tag.name,
                    raw: tag.raw,
                })
            }
            FrameKind::Erb {
                kind,
                token_index,
                code,
                ..
            } => {
                self.stack.push(Frame {
                    kind: FrameKind::Erb {
                        kind,
                        code: code.clone(),
                        token_index,
                    },
                    children: frame.children,
                });
                Err(ParseError::UnexpectedHtmlCloseTag {
                    name: tag.name,
                    raw: tag.raw,
                })
            }
            FrameKind::Html { name, raw } => {
                if !name.eq_ignore_ascii_case(&tag.name) {
                    self.stack.push(Frame {
                        kind: FrameKind::Html {
                            name: name.clone(),
                            raw,
                        },
                        children: frame.children,
                    });
                    return Err(ParseError::MismatchedHtmlCloseTag {
                        expected: name,
                        found: tag.name,
                        raw: tag.raw,
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
            FrameKind::Html { name, raw } => {
                self.stack.push(Frame {
                    kind: FrameKind::Html {
                        name: name.clone(),
                        raw: raw.clone(),
                    },
                    children: frame.children,
                });
                Err(ParseError::UnclosedHtmlTag { name, raw })
            }
            FrameKind::Erb {
                kind,
                code: block_code,
                ..
            } => {
                self.push_node(Node::ErbBlock {
                    kind,
                    code: block_code,
                    children: frame.children,
                });
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
            FrameKind::Html { name, raw } => Err(ParseError::UnclosedHtmlTag { name, raw }),
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
}

impl Frame {
    fn root() -> Self {
        Self {
            kind: FrameKind::Root,
            children: Vec::new(),
        }
    }

    fn html(tag: HtmlTag) -> Self {
        Self {
            kind: FrameKind::Html {
                name: tag.name,
                raw: tag.raw,
            },
            children: Vec::new(),
        }
    }

    fn erb(kind: ErbBlockKind, code: String, token_index: usize) -> Self {
        Self {
            kind: FrameKind::Erb {
                kind,
                code,
                token_index,
            },
            children: Vec::new(),
        }
    }
}

enum FrameKind {
    Root,
    Html {
        name: String,
        raw: String,
    },
    Erb {
        kind: ErbBlockKind,
        code: String,
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
    fn keeps_erb_blocks_with_html_children() {
        let tokens = tokenize("<% if user %><ul><li>Hello</li></ul><% end %>").unwrap();
        let document = parse(&tokens).unwrap();

        assert_eq!(
            document,
            Document {
                children: vec![Node::ErbBlock {
                    kind: ErbBlockKind::If,
                    code: "if user".to_string(),
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
                    }]
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
                raw: "</div>".to_string()
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
                raw: "</span>".to_string()
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
                raw: "<div>".to_string()
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
                raw: "<div>".to_string()
            }
        );
    }
}
