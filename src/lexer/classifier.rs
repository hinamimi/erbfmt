use super::{ErbBlockKind, ErbBranchKind, ErbTag, Token};

pub(super) fn classify_code(tag: ErbTag) -> Token {
    if starts_with_keyword(&tag.code, "if") {
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
    if ends_with_do_block(&tag.code) {
        Token::ErbBlockStart {
            kind: ErbBlockKind::Do,
            tag,
            output: true,
        }
    } else {
        Token::ErbOutput(tag)
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
