use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Html(String),
    ErbCode(String),
    ErbOutput(String),
    ErbBlockStart { kind: ErbBlockKind, code: String },
    ErbBranch { kind: ErbBranchKind, code: String },
    ErbBlockEnd(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    position: usize,
    location: SourceLocation,
    message: String,
}

impl LexError {
    fn unterminated_erb(input: &str, position: usize) -> Self {
        Self {
            position,
            location: source_location(input, position),
            message: "unterminated ERB tag".to_string(),
        }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.message, self.location)
    }
}

impl std::error::Error for LexError {}

#[cfg(test)]
pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
    Ok(tokenize_with_spans(input)?
        .into_iter()
        .map(|spanned| spanned.token)
        .collect())
}

pub fn tokenize_with_spans(input: &str) -> Result<Vec<SpannedToken>, LexError> {
    let mut tokens = Vec::new();
    let mut cursor = 0;
    let mut search_cursor = 0;

    while let Some(relative_start) = input[search_cursor..].find("<%") {
        let start = search_cursor + relative_start;

        if is_inside_html_tag(input, start) {
            search_cursor = start + "<%".len();
            continue;
        }

        if start > cursor {
            tokens.push(spanned_token(
                input,
                cursor,
                start,
                Token::Html(input[cursor..start].to_string()),
            ));
        }

        let tag_content_start = start + "<%".len();
        let (is_output, code_start) = if input[tag_content_start..].starts_with('=') {
            (true, tag_content_start + "=".len())
        } else {
            (false, tag_content_start)
        };

        let Some(relative_end) = input[code_start..].find("%>") else {
            return Err(LexError::unterminated_erb(input, start));
        };

        let code_end = code_start + relative_end;
        let code = input[code_start..code_end].trim().to_string();
        let token_end = code_end + "%>".len();
        let token = if is_output {
            Token::ErbOutput(code)
        } else {
            classify_code(code)
        };

        tokens.push(spanned_token(input, start, token_end, token));
        cursor = token_end;
        search_cursor = cursor;
    }

    if cursor < input.len() {
        tokens.push(spanned_token(
            input,
            cursor,
            input.len(),
            Token::Html(input[cursor..].to_string()),
        ));
    }

    Ok(tokens)
}

fn spanned_token(input: &str, start: usize, end: usize, token: Token) -> SpannedToken {
    SpannedToken {
        token,
        span: Span {
            start,
            end,
            location: source_location(input, start),
        },
    }
}

pub fn source_location(input: &str, position: usize) -> SourceLocation {
    let mut line = 1;
    let mut column = 1;

    for (index, ch) in input.char_indices() {
        if index >= position {
            break;
        }

        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    SourceLocation { line, column }
}

fn is_inside_html_tag(input: &str, position: usize) -> bool {
    let mut cursor = 0;
    let mut inside_tag = false;
    let mut quote = None;

    while cursor < position {
        if input[cursor..].starts_with("<%") {
            let Some(relative_end) = input[cursor + "<%".len()..].find("%>") else {
                return inside_tag;
            };
            cursor += "<%".len() + relative_end + "%>".len();
            continue;
        }

        if !inside_tag && input[cursor..].starts_with("<!--") {
            let Some(relative_end) = input[cursor + "<!--".len()..].find("-->") else {
                return false;
            };
            cursor += "<!--".len() + relative_end + "-->".len();
            continue;
        }

        let ch = input[cursor..]
            .chars()
            .next()
            .expect("cursor is inside input");

        if inside_tag {
            match quote {
                Some(active_quote) if ch == active_quote => quote = None,
                Some(_) => {}
                None if ch == '"' || ch == '\'' => quote = Some(ch),
                None if ch == '>' => inside_tag = false,
                None => {}
            }
        } else if ch == '<' && starts_html_tag_like(input, cursor) {
            inside_tag = true;
        }

        cursor += ch.len_utf8();
    }

    inside_tag
}

fn starts_html_tag_like(input: &str, position: usize) -> bool {
    let Some(rest) = input[position..].strip_prefix('<') else {
        return false;
    };

    rest.starts_with("!--")
        || rest
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic() || matches!(ch, '/' | '!' | '?'))
}

fn classify_code(code: String) -> Token {
    if starts_with_keyword(&code, "if") {
        Token::ErbBlockStart {
            kind: ErbBlockKind::If,
            code,
        }
    } else if starts_with_keyword(&code, "unless") {
        Token::ErbBlockStart {
            kind: ErbBlockKind::Unless,
            code,
        }
    } else if starts_with_keyword(&code, "case") {
        Token::ErbBlockStart {
            kind: ErbBlockKind::Case,
            code,
        }
    } else if starts_with_keyword(&code, "begin") {
        Token::ErbBlockStart {
            kind: ErbBlockKind::Begin,
            code,
        }
    } else if starts_with_keyword(&code, "else") {
        Token::ErbBranch {
            kind: ErbBranchKind::Else,
            code,
        }
    } else if starts_with_keyword(&code, "elsif") {
        Token::ErbBranch {
            kind: ErbBranchKind::Elsif,
            code,
        }
    } else if starts_with_keyword(&code, "when") {
        Token::ErbBranch {
            kind: ErbBranchKind::When,
            code,
        }
    } else if starts_with_keyword(&code, "end") {
        Token::ErbBlockEnd(code)
    } else if starts_with_keyword(&code, "do") || ends_with_do_block(&code) {
        Token::ErbBlockStart {
            kind: ErbBlockKind::Do,
            code,
        }
    } else {
        Token::ErbCode(code)
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

        assert_eq!(tokens, vec![Token::ErbCode(String::new())]);
    }

    #[test]
    fn tokenizes_erb_output_tag() {
        let tokens = tokenize("<%= user.name %>").unwrap();

        assert_eq!(tokens, vec![Token::ErbOutput("user.name".to_string())]);
    }

    #[test]
    fn tokenizes_html_fragments_around_erb() {
        let tokens = tokenize("<p>Hello <%= user.name %></p>").unwrap();

        assert_eq!(
            tokens,
            vec![
                Token::Html("<p>Hello ".to_string()),
                Token::ErbOutput("user.name".to_string()),
                Token::Html("</p>".to_string())
            ]
        );
    }

    #[test]
    fn keeps_erb_output_inside_html_tag_attributes_as_html() {
        let tokens = tokenize(
            r#"<a href="/users/<%= user.id %>" aria-label="<%= user.name %>"><%= user.name %></a>"#,
        )
        .unwrap();

        assert_eq!(
            tokens,
            vec![
                Token::Html(
                    r#"<a href="/users/<%= user.id %>" aria-label="<%= user.name %>">"#.to_string()
                ),
                Token::ErbOutput("user.name".to_string()),
                Token::Html("</a>".to_string())
            ]
        );
    }

    #[test]
    fn tokenizes_erb_after_non_tag_less_than_sign() {
        let tokens = tokenize("2 < 3 <%= result %>").unwrap();

        assert_eq!(
            tokens,
            vec![
                Token::Html("2 < 3 ".to_string()),
                Token::ErbOutput("result".to_string())
            ]
        );
    }

    #[test]
    fn tokenizes_supported_erb_control_tags() {
        let cases = [
            ("<% if user %>", ErbBlockKind::If, "if user"),
            (
                "<% unless user.guest? %>",
                ErbBlockKind::Unless,
                "unless user.guest?",
            ),
            ("<% case user.role %>", ErbBlockKind::Case, "case user.role"),
            ("<% do %>", ErbBlockKind::Do, "do"),
        ];

        for (input, kind, code) in cases {
            assert_eq!(
                tokenize(input).unwrap(),
                vec![Token::ErbBlockStart {
                    kind,
                    code: code.to_string()
                }]
            );
        }
    }

    #[test]
    fn tokenizes_erb_block_end_tag() {
        let tokens = tokenize("<% end %>").unwrap();

        assert_eq!(tokens, vec![Token::ErbBlockEnd("end".to_string())]);
    }

    #[test]
    fn tokenizes_erb_branch_tags() {
        let cases = [
            ("<% else %>", ErbBranchKind::Else, "else"),
            ("<% elsif admin? %>", ErbBranchKind::Elsif, "elsif admin?"),
            (
                "<% when \"admin\" %>",
                ErbBranchKind::When,
                "when \"admin\"",
            ),
        ];

        for (input, kind, code) in cases {
            assert_eq!(
                tokenize(input).unwrap(),
                vec![Token::ErbBranch {
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
            vec![Token::ErbBlockStart {
                kind: ErbBlockKind::Begin,
                code: "begin".to_string()
            }]
        );
    }

    #[test]
    fn tokenizes_do_block_expression() {
        let tokens = tokenize("<% users.each do |user| %>").unwrap();

        assert_eq!(
            tokens,
            vec![Token::ErbBlockStart {
                kind: ErbBlockKind::Do,
                code: "users.each do |user|".to_string()
            }]
        );
    }

    #[test]
    fn reports_unterminated_erb_tag() {
        let error = tokenize("<div><% if user").unwrap_err();

        assert_eq!(
            error.to_string(),
            "unterminated ERB tag at line 1, column 6"
        );
    }
}
