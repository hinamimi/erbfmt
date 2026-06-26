use super::{ErbBlockKind, ErbBranchKind, Token};

pub(super) fn classify_code(code: String) -> Token {
    if starts_with_keyword(&code, "if") {
        Token::ErbBlockStart {
            kind: ErbBlockKind::If,
            code,
            output: false,
        }
    } else if starts_with_keyword(&code, "unless") {
        Token::ErbBlockStart {
            kind: ErbBlockKind::Unless,
            code,
            output: false,
        }
    } else if starts_with_keyword(&code, "case") {
        Token::ErbBlockStart {
            kind: ErbBlockKind::Case,
            code,
            output: false,
        }
    } else if starts_with_keyword(&code, "begin") {
        Token::ErbBlockStart {
            kind: ErbBlockKind::Begin,
            code,
            output: false,
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
    } else if starts_with_keyword(&code, "rescue") {
        Token::ErbBranch {
            kind: ErbBranchKind::Rescue,
            code,
        }
    } else if starts_with_keyword(&code, "ensure") {
        Token::ErbBranch {
            kind: ErbBranchKind::Ensure,
            code,
        }
    } else if starts_with_keyword(&code, "end") {
        Token::ErbBlockEnd(code)
    } else if starts_with_keyword(&code, "do") || ends_with_do_block(&code) {
        Token::ErbBlockStart {
            kind: ErbBlockKind::Do,
            code,
            output: false,
        }
    } else {
        Token::ErbCode(code)
    }
}

pub(super) fn classify_output_code(code: String) -> Token {
    if ends_with_do_block(&code) {
        Token::ErbBlockStart {
            kind: ErbBlockKind::Do,
            code,
            output: true,
        }
    } else {
        Token::ErbOutput(code)
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
