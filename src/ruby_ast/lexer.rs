use super::error::RubyAstError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Token {
    pub kind: TokenKind,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TokenKind {
    DoubleColon,
    Identifier(String),
    Number(String),
    Operator(String),
    PercentLiteral(String),
    Punct(char),
    SafeNavigation,
    String(String),
    Symbol(String),
}

pub(super) fn tokenize(source: &str) -> Result<Vec<Token>, RubyAstError> {
    let mut tokens = Vec::new();
    let mut chars = source.char_indices().peekable();

    while let Some((offset, ch)) = chars.next() {
        if ch.is_whitespace() {
            continue;
        }

        if ch == '#' {
            return Err(RubyAstError::new(
                "Ruby comments are not supported in AST mode",
                offset,
            ));
        }

        if ch == '\'' || ch == '"' {
            tokens.push(Token {
                kind: TokenKind::String(read_string(source, offset, ch)?),
                offset,
            });

            while chars
                .peek()
                .is_some_and(|(index, _)| *index < string_end(source, offset, ch))
            {
                chars.next();
            }

            continue;
        }

        if ch == '%'
            && let Some((literal, end)) = read_percent_literal(source, offset)?
        {
            tokens.push(Token {
                kind: TokenKind::PercentLiteral(literal),
                offset,
            });

            while chars.peek().is_some_and(|(index, _)| *index < end) {
                chars.next();
            }

            continue;
        }

        if ch == ':' {
            if chars.peek().is_some_and(|(_, next)| *next == ':') {
                chars.next();
                tokens.push(Token {
                    kind: TokenKind::DoubleColon,
                    offset,
                });
                continue;
            }

            if chars
                .peek()
                .is_some_and(|(_, next)| is_identifier_start(*next))
            {
                let (symbol, end) = read_symbol(source, offset + ch.len_utf8());
                tokens.push(Token {
                    kind: TokenKind::Symbol(symbol),
                    offset,
                });

                while chars.peek().is_some_and(|(index, _)| *index < end) {
                    chars.next();
                }

                continue;
            }

            tokens.push(Token {
                kind: TokenKind::Punct(':'),
                offset,
            });
            continue;
        }

        if is_identifier_start(ch) || ch == '@' {
            let (identifier, end) = read_identifier(source, offset);
            tokens.push(Token {
                kind: TokenKind::Identifier(identifier),
                offset,
            });

            while chars.peek().is_some_and(|(index, _)| *index < end) {
                chars.next();
            }

            continue;
        }

        if ch == '&' && chars.peek().is_some_and(|(_, next)| *next == '.') {
            chars.next();
            tokens.push(Token {
                kind: TokenKind::SafeNavigation,
                offset,
            });
            continue;
        }

        if ch == '.' && chars.peek().is_some_and(|(_, next)| *next == '.') {
            chars.next();
            let mut operator = "..";

            if chars.peek().is_some_and(|(_, next)| *next == '.') {
                chars.next();
                operator = "...";
            }

            tokens.push(Token {
                kind: TokenKind::Operator(operator.to_string()),
                offset,
            });
            continue;
        }

        if ch.is_ascii_digit() {
            let (number, end) = read_number(source, offset);
            tokens.push(Token {
                kind: TokenKind::Number(number),
                offset,
            });

            while chars.peek().is_some_and(|(index, _)| *index < end) {
                chars.next();
            }

            continue;
        }

        if matches!(ch, '(' | ')' | '[' | ']' | '{' | '}' | ',' | '.' | '?') {
            tokens.push(Token {
                kind: TokenKind::Punct(ch),
                offset,
            });
            continue;
        }

        if is_operator_char(ch) {
            let (operator, end) = read_operator(source, offset);
            tokens.push(Token {
                kind: TokenKind::Operator(operator),
                offset,
            });

            while chars.peek().is_some_and(|(index, _)| *index < end) {
                chars.next();
            }

            continue;
        }

        return Err(RubyAstError::new(
            format!("unsupported Ruby character `{ch}`"),
            offset,
        ));
    }

    Ok(tokens)
}

fn read_string(source: &str, start: usize, quote: char) -> Result<String, RubyAstError> {
    let mut escaped = false;

    for (offset, ch) in source[start + quote.len_utf8()..].char_indices() {
        let absolute = start + quote.len_utf8() + offset;

        if escaped {
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == quote {
            return Ok(source[start..absolute + ch.len_utf8()].to_string());
        }
    }

    Err(RubyAstError::new("unterminated Ruby string", start))
}

fn string_end(source: &str, start: usize, quote: char) -> usize {
    read_string(source, start, quote)
        .map(|value| start + value.len())
        .unwrap_or(source.len())
}

fn read_percent_literal(
    source: &str,
    start: usize,
) -> Result<Option<(String, usize)>, RubyAstError> {
    let mut chars = source[start + '%'.len_utf8()..].char_indices();
    let Some((first_offset, first)) = chars.next() else {
        return Ok(None);
    };

    let tagged = first.is_ascii_alphabetic();
    let (opener_offset, opener) = if tagged {
        let Some((opener_offset, opener)) = chars.next() else {
            return Ok(None);
        };

        (opener_offset, opener)
    } else {
        (first_offset, first)
    };

    let Some(closer) = percent_literal_closer(opener, tagged) else {
        return Ok(None);
    };

    let opener_absolute = start + '%'.len_utf8() + opener_offset;
    let mut depth = 1;
    let mut escaped = false;

    for (offset, ch) in source[opener_absolute + opener.len_utf8()..].char_indices() {
        let absolute = opener_absolute + opener.len_utf8() + offset;

        if escaped {
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == opener && opener != closer {
            depth += 1;
            continue;
        }

        if ch == closer {
            depth -= 1;

            if depth == 0 {
                let end = absolute + ch.len_utf8();
                return Ok(Some((source[start..end].to_string(), end)));
            }
        }
    }

    Err(RubyAstError::new(
        "unterminated Ruby percent literal",
        start,
    ))
}

fn percent_literal_closer(opener: char, tagged: bool) -> Option<char> {
    match opener {
        '(' => Some(')'),
        '[' => Some(']'),
        '{' => Some('}'),
        '<' => Some('>'),
        _ if tagged && !opener.is_ascii_alphanumeric() && !opener.is_whitespace() => Some(opener),
        _ => None,
    }
}

fn read_symbol(source: &str, start: usize) -> (String, usize) {
    let (identifier, end) = read_identifier(source, start);
    (format!(":{identifier}"), end)
}

fn read_identifier(source: &str, start: usize) -> (String, usize) {
    let mut end = start;

    for (offset, ch) in source[start..].char_indices() {
        if offset == 0 {
            if ch == '@' {
                end = start + ch.len_utf8();
                continue;
            }

            if !is_identifier_start(ch) {
                break;
            }
        } else if !is_identifier_continue(ch) {
            if matches!(ch, '?' | '!') {
                end = start + offset + ch.len_utf8();
            }
            break;
        }

        end = start + offset + ch.len_utf8();
    }

    (source[start..end].to_string(), end)
}

fn read_number(source: &str, start: usize) -> (String, usize) {
    let mut end = start;
    let mut seen_decimal = false;
    let mut chars = source[start..].char_indices().peekable();

    while let Some((offset, ch)) = chars.next() {
        if ch.is_ascii_digit() || ch == '_' {
            end = start + offset + ch.len_utf8();
            continue;
        }

        if ch == '.' && !seen_decimal && chars.peek().is_some_and(|(_, next)| next.is_ascii_digit())
        {
            seen_decimal = true;
            end = start + offset + ch.len_utf8();
            continue;
        }

        break;
    }

    (source[start..end].to_string(), end)
}

fn read_operator(source: &str, start: usize) -> (String, usize) {
    let mut end = start;

    for (offset, ch) in source[start..].char_indices() {
        if !is_operator_char(ch) {
            break;
        }

        end = start + offset + ch.len_utf8();
    }

    (source[start..end].to_string(), end)
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric()
}

fn is_operator_char(ch: char) -> bool {
    matches!(
        ch,
        '=' | '!' | '<' | '>' | '&' | '|' | '+' | '-' | '*' | '/' | '%'
    )
}
