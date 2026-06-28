use super::{ErbBlockKind, ErbBranchKind, ErbTag, Token};

pub(super) fn classify_code(tag: ErbTag) -> Token {
    if is_self_contained_control_flow(&tag.code) {
        Token::ErbCode(tag)
    } else if starts_with_keyword(&tag.code, "if") {
        Token::ErbBlockStart {
            kind: ErbBlockKind::If,
            tag,
            output: false,
        }
    } else if starts_with_keyword(&tag.code, "unless") {
        Token::ErbBlockStart {
            kind: ErbBlockKind::Unless,
            tag,
            output: false,
        }
    } else if starts_with_keyword(&tag.code, "case") {
        Token::ErbBlockStart {
            kind: ErbBlockKind::Case,
            tag,
            output: false,
        }
    } else if starts_with_keyword(&tag.code, "begin") {
        Token::ErbBlockStart {
            kind: ErbBlockKind::Begin,
            tag,
            output: false,
        }
    } else if starts_with_keyword(&tag.code, "else") {
        Token::ErbBranch {
            kind: ErbBranchKind::Else,
            tag,
        }
    } else if starts_with_keyword(&tag.code, "elsif") {
        Token::ErbBranch {
            kind: ErbBranchKind::Elsif,
            tag,
        }
    } else if starts_with_keyword(&tag.code, "when") {
        Token::ErbBranch {
            kind: ErbBranchKind::When,
            tag,
        }
    } else if starts_with_keyword(&tag.code, "rescue") {
        Token::ErbBranch {
            kind: ErbBranchKind::Rescue,
            tag,
        }
    } else if starts_with_keyword(&tag.code, "ensure") {
        Token::ErbBranch {
            kind: ErbBranchKind::Ensure,
            tag,
        }
    } else if starts_with_keyword(&tag.code, "end") {
        Token::ErbBlockEnd(tag)
    } else if starts_with_keyword(&tag.code, "do") || ends_with_do_block(&tag.code) {
        Token::ErbBlockStart {
            kind: ErbBlockKind::Do,
            tag,
            output: false,
        }
    } else {
        Token::ErbCode(tag)
    }
}

pub(super) fn classify_output_code(tag: ErbTag) -> Token {
    if !is_self_contained_control_flow(&tag.code) && ends_with_do_block(&tag.code) {
        Token::ErbBlockStart {
            kind: ErbBlockKind::Do,
            tag,
            output: true,
        }
    } else {
        Token::ErbOutput(tag)
    }
}

fn is_self_contained_control_flow(code: &str) -> bool {
    contains_ruby_keyword(code, "end")
        && (starts_with_keyword(code, "if")
            || starts_with_keyword(code, "unless")
            || starts_with_keyword(code, "case")
            || starts_with_keyword(code, "begin")
            || starts_with_keyword(code, "do")
            || contains_ruby_keyword(code, "do"))
}

fn contains_ruby_keyword(code: &str, keyword: &str) -> bool {
    let mut scanner = RubyKeywordScanner::default();
    let mut chars = code.char_indices().peekable();

    while let Some((index, ch)) = chars.next() {
        if scanner.consume(ch) {
            continue;
        }

        if is_identifier_start(ch) {
            let start = index;
            let mut end = index + ch.len_utf8();

            while let Some((next_index, next)) = chars.peek().copied() {
                if !is_identifier_char(next) {
                    break;
                }

                chars.next();
                end = next_index + next.len_utf8();
            }

            if &code[start..end] == keyword && has_ruby_keyword_boundaries(code, start, end) {
                return true;
            }
        }
    }

    false
}

fn has_ruby_keyword_boundaries(code: &str, start: usize, end: usize) -> bool {
    let before = code[..start].chars().next_back();
    let after = code[end..].chars().next();

    let left = before.is_none_or(|c| !is_identifier_char(c) && !matches!(c, ':' | '.'));
    let right = after.is_none_or(|c| !is_identifier_char(c) && c != ':');

    left && right
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

fn is_identifier_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

#[derive(Default)]
struct RubyKeywordScanner {
    string: Option<char>,
    escaped: bool,
    comment: bool,
}

impl RubyKeywordScanner {
    fn consume(&mut self, ch: char) -> bool {
        if self.comment {
            if ch == '\n' {
                self.comment = false;
            }

            return true;
        }

        if let Some(quote) = self.string {
            if self.escaped {
                self.escaped = false;
                return true;
            }

            if ch == '\\' {
                self.escaped = true;
                return true;
            }

            if ch == quote {
                self.string = None;
            }

            return true;
        }

        match ch {
            '\'' | '"' => {
                self.string = Some(ch);
                true
            }
            '#' => {
                self.comment = true;
                true
            }
            _ => false,
        }
    }
}
