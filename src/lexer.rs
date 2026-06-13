use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Html(String),
    Erb { kind: ErbKind, code: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErbKind {
    Code,
    Output,
    If,
    Unless,
    Case,
    Do,
    Begin,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    position: usize,
    message: String,
}

impl LexError {
    fn unterminated_erb(position: usize) -> Self {
        Self {
            position,
            message: "unterminated ERB tag".to_string(),
        }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.message, self.position)
    }
}

impl std::error::Error for LexError {}

pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let mut cursor = 0;

    while let Some(relative_start) = input[cursor..].find("<%") {
        let start = cursor + relative_start;

        if start > cursor {
            tokens.push(Token::Html(input[cursor..start].to_string()));
        }

        let tag_content_start = start + "<%".len();
        let (kind_hint, code_start) = if input[tag_content_start..].starts_with('=') {
            (ErbKind::Output, tag_content_start + "=".len())
        } else {
            (ErbKind::Code, tag_content_start)
        };

        let Some(relative_end) = input[code_start..].find("%>") else {
            return Err(LexError::unterminated_erb(start));
        };

        let code_end = code_start + relative_end;
        let code = input[code_start..code_end].trim().to_string();
        let kind = match kind_hint {
            ErbKind::Output => ErbKind::Output,
            ErbKind::Code => classify_code(&code),
            _ => unreachable!("lexer only creates code and output hints"),
        };

        tokens.push(Token::Erb { kind, code });
        cursor = code_end + "%>".len();
    }

    if cursor < input.len() {
        tokens.push(Token::Html(input[cursor..].to_string()));
    }

    Ok(tokens)
}

fn classify_code(code: &str) -> ErbKind {
    if starts_with_keyword(code, "if") {
        ErbKind::If
    } else if starts_with_keyword(code, "unless") {
        ErbKind::Unless
    } else if starts_with_keyword(code, "case") {
        ErbKind::Case
    } else if starts_with_keyword(code, "begin") {
        ErbKind::Begin
    } else if starts_with_keyword(code, "end") {
        ErbKind::End
    } else if starts_with_keyword(code, "do") || ends_with_do_block(code) {
        ErbKind::Do
    } else {
        ErbKind::Code
    }
}

fn starts_with_keyword(code: &str, keyword: &str) -> bool {
    let trimmed = code.trim_start();

    if !trimmed.starts_with(keyword) {
        return false;
    }

    trimmed[keyword.len()..]
        .chars()
        .next()
        .is_none_or(|c| !is_identifier_char(c))
}

fn ends_with_do_block(code: &str) -> bool {
    let trimmed = code.trim_end();
    let Some(index) = find_last_keyword(trimmed, "do") else {
        return false;
    };

    let rest = trimmed[index + "do".len()..].trim();
    rest.is_empty() || (rest.starts_with('|') && rest.ends_with('|'))
}

fn find_last_keyword(code: &str, keyword: &str) -> Option<usize> {
    code.match_indices(keyword)
        .filter_map(|(index, _)| {
            let before = code[..index].chars().next_back();
            let after = code[index + keyword.len()..].chars().next();

            let has_left_boundary = before.is_none_or(char::is_whitespace);
            let has_right_boundary = after.is_none_or(|c| !is_identifier_char(c));

            has_left_boundary
                .then_some(index)
                .filter(|_| has_right_boundary)
        })
        .last()
}

fn is_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '?' | '!')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_html() {
        let tokens = tokenize("<div>Hello</div>").unwrap();

        assert_eq!(tokens, vec![Token::Html("<div>Hello</div>".to_string())]);
    }

    #[test]
    fn tokenizes_empty_erb_code_tag() {
        let tokens = tokenize("<% %>").unwrap();

        assert_eq!(
            tokens,
            vec![Token::Erb {
                kind: ErbKind::Code,
                code: String::new()
            }]
        );
    }

    #[test]
    fn tokenizes_erb_output_tag() {
        let tokens = tokenize("<%= user.name %>").unwrap();

        assert_eq!(
            tokens,
            vec![Token::Erb {
                kind: ErbKind::Output,
                code: "user.name".to_string()
            }]
        );
    }

    #[test]
    fn tokenizes_html_fragments_around_erb() {
        let tokens = tokenize("<p>Hello <%= user.name %></p>").unwrap();

        assert_eq!(
            tokens,
            vec![
                Token::Html("<p>Hello ".to_string()),
                Token::Erb {
                    kind: ErbKind::Output,
                    code: "user.name".to_string()
                },
                Token::Html("</p>".to_string())
            ]
        );
    }

    #[test]
    fn tokenizes_supported_erb_control_tags() {
        let cases = [
            ("<% if user %>", ErbKind::If, "if user"),
            (
                "<% unless user.guest? %>",
                ErbKind::Unless,
                "unless user.guest?",
            ),
            ("<% case user.role %>", ErbKind::Case, "case user.role"),
            ("<% do %>", ErbKind::Do, "do"),
            ("<% end %>", ErbKind::End, "end"),
        ];

        for (input, kind, code) in cases {
            assert_eq!(
                tokenize(input).unwrap(),
                vec![Token::Erb {
                    kind,
                    code: code.to_string()
                }]
            );
        }
    }

    #[test]
    fn tokenizes_begin_control_tag() {
        let tokens = tokenize("<% begin %>").unwrap();

        assert_eq!(
            tokens,
            vec![Token::Erb {
                kind: ErbKind::Begin,
                code: "begin".to_string()
            }]
        );
    }

    #[test]
    fn tokenizes_do_block_expression() {
        let tokens = tokenize("<% users.each do |user| %>").unwrap();

        assert_eq!(
            tokens,
            vec![Token::Erb {
                kind: ErbKind::Do,
                code: "users.each do |user|".to_string()
            }]
        );
    }

    #[test]
    fn reports_unterminated_erb_tag() {
        let error = tokenize("<div><% if user").unwrap_err();

        assert_eq!(error.to_string(), "unterminated ERB tag at byte 5");
    }
}
