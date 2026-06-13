use std::fmt;

use crate::lexer::{ErbBlockKind, Token};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Html(String),
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
    UnexpectedBlockEnd { token_index: usize, code: String },
    UnclosedBlock { token_index: usize, code: String },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedBlockEnd { token_index, code } => {
                write!(
                    f,
                    "unexpected ERB block end `{code}` at token {token_index}"
                )
            }
            Self::UnclosedBlock { token_index, code } => {
                write!(f, "unclosed ERB block `{code}` at token {token_index}")
            }
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse(tokens: &[Token]) -> Result<Document, ParseError> {
    let mut cursor = 0;
    let children = parse_nodes(tokens, &mut cursor, false)?;

    Ok(Document { children })
}

fn parse_nodes(
    tokens: &[Token],
    cursor: &mut usize,
    stop_on_block_end: bool,
) -> Result<Vec<Node>, ParseError> {
    let mut nodes = Vec::new();

    while let Some(token) = tokens.get(*cursor) {
        match token {
            Token::Html(html) => {
                nodes.push(Node::Html(html.clone()));
                *cursor += 1;
            }
            Token::ErbCode(code) => {
                nodes.push(Node::ErbCode(code.clone()));
                *cursor += 1;
            }
            Token::ErbBranch { code, .. } => {
                nodes.push(Node::ErbCode(code.clone()));
                *cursor += 1;
            }
            Token::ErbOutput(code) => {
                nodes.push(Node::ErbOutput(code.clone()));
                *cursor += 1;
            }
            Token::ErbBlockStart { kind, code } => {
                let block_index = *cursor;
                *cursor += 1;
                let children = parse_nodes(tokens, cursor, true).map_err(|error| match error {
                    ParseError::UnclosedBlock {
                        code: unclosed_code,
                        ..
                    } if unclosed_code.is_empty() => ParseError::UnclosedBlock {
                        token_index: block_index,
                        code: code.clone(),
                    },
                    ParseError::UnexpectedBlockEnd { .. } | ParseError::UnclosedBlock { .. } => {
                        error
                    }
                })?;

                nodes.push(Node::ErbBlock {
                    kind: *kind,
                    code: code.clone(),
                    children,
                });
            }
            Token::ErbBlockEnd(code) => {
                if stop_on_block_end {
                    *cursor += 1;
                    return Ok(nodes);
                }

                return Err(ParseError::UnexpectedBlockEnd {
                    token_index: *cursor,
                    code: code.clone(),
                });
            }
        }
    }

    if stop_on_block_end {
        return Err(ParseError::UnclosedBlock {
            token_index: tokens.len(),
            code: String::new(),
        });
    }

    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{ErbBlockKind, tokenize};

    #[test]
    fn parses_plain_html() {
        let tokens = tokenize("<div>Hello</div>").unwrap();
        let document = parse(&tokens).unwrap();

        assert_eq!(
            document,
            Document {
                children: vec![Node::Html("<div>Hello</div>".to_string())]
            }
        );
    }

    #[test]
    fn parses_erb_output_and_code_nodes() {
        let tokens = tokenize("<% greeting = \"Hi\" %><%= greeting %>").unwrap();
        let document = parse(&tokens).unwrap();

        assert_eq!(
            document,
            Document {
                children: vec![
                    Node::ErbCode("greeting = \"Hi\"".to_string()),
                    Node::ErbOutput("greeting".to_string())
                ]
            }
        );
    }

    #[test]
    fn parses_erb_block_children() {
        let tokens = tokenize("<% if user %><p><%= user.name %></p><% end %>").unwrap();
        let document = parse(&tokens).unwrap();

        assert_eq!(
            document,
            Document {
                children: vec![Node::ErbBlock {
                    kind: ErbBlockKind::If,
                    code: "if user".to_string(),
                    children: vec![
                        Node::Html("<p>".to_string()),
                        Node::ErbOutput("user.name".to_string()),
                        Node::Html("</p>".to_string())
                    ]
                }]
            }
        );
    }

    #[test]
    fn parses_nested_erb_blocks() {
        let tokens =
            tokenize("<% if user %><% users.each do |user| %><%= user.name %><% end %><% end %>")
                .unwrap();
        let document = parse(&tokens).unwrap();

        assert_eq!(
            document,
            Document {
                children: vec![Node::ErbBlock {
                    kind: ErbBlockKind::If,
                    code: "if user".to_string(),
                    children: vec![Node::ErbBlock {
                        kind: ErbBlockKind::Do,
                        code: "users.each do |user|".to_string(),
                        children: vec![Node::ErbOutput("user.name".to_string())]
                    }]
                }]
            }
        );
    }

    #[test]
    fn reports_unexpected_block_end() {
        let tokens = tokenize("<% end %>").unwrap();
        let error = parse(&tokens).unwrap_err();

        assert_eq!(
            error,
            ParseError::UnexpectedBlockEnd {
                token_index: 0,
                code: "end".to_string()
            }
        );
    }

    #[test]
    fn reports_unclosed_block() {
        let tokens = tokenize("<% if user %><p>Hello</p>").unwrap();
        let error = parse(&tokens).unwrap_err();

        assert_eq!(
            error,
            ParseError::UnclosedBlock {
                token_index: 0,
                code: "if user".to_string()
            }
        );
    }
}
