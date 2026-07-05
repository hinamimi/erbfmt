use super::ast::{AssignmentOperator, Block};
use super::error::RubyAstError;

pub(super) fn split_do_block(source: &str) -> Result<(&str, Option<Block>), RubyAstError> {
    let mut scanner = Scanner::default();
    let mut block_start = None;

    for (offset, ch) in source.char_indices() {
        if scanner.is_top_level()
            && ch.is_whitespace()
            && source[offset..].trim_start().starts_with("do")
        {
            let leading = source[offset..].len() - source[offset..].trim_start().len();
            let candidate = offset + leading;
            let after_do = &source[candidate + "do".len()..];

            if after_do.is_empty() || after_do.chars().next().is_some_and(char::is_whitespace) {
                block_start = Some(candidate);
            }
        }

        if scanner.is_top_level() && ch.is_whitespace() {
            let leading = source[offset..].len() - source[offset..].trim_start().len();
            let candidate = offset + leading;
            let before = source[..candidate].trim_end();
            let suffix = source[candidate..].trim();

            if before.ends_with(')')
                && suffix.starts_with('{')
                && is_balanced_brace_block_suffix(suffix)
            {
                block_start = Some(candidate);
            }
        }

        scanner.consume(ch)?;
    }

    let Some(block_start) = block_start else {
        return Ok((source, None));
    };

    let call = source[..block_start].trim_end();
    let suffix = source[block_start..].trim();
    let block = parse_block_suffix(suffix, block_start)?;

    Ok((call, Some(block)))
}

pub(super) fn split_top_level_statements(source: &str) -> Result<Vec<&str>, RubyAstError> {
    let mut scanner = Scanner::default();
    let mut statements = Vec::new();
    let mut start = 0;

    for (offset, ch) in source.char_indices() {
        if scanner.is_top_level() && matches!(ch, ';' | '\n') {
            let statement = source[start..offset].trim();

            if !statement.is_empty() {
                statements.push(statement);
            }

            start = offset + ch.len_utf8();
            continue;
        }

        scanner.consume(ch)?;
    }

    let statement = source[start..].trim();
    if !statement.is_empty() {
        statements.push(statement);
    }

    Ok(statements)
}

pub(super) fn split_assignment(
    source: &str,
) -> Result<Option<(&str, AssignmentOperator, &str)>, RubyAstError> {
    let mut scanner = Scanner::default();

    for (offset, ch) in source.char_indices() {
        if scanner.is_top_level()
            && let Some((raw_operator, operator)) = assignment_operator_at(source, offset)
        {
            let target = source[..offset].trim();
            let value = source[offset + raw_operator.len()..].trim();

            if target.is_empty() || value.is_empty() {
                return Err(RubyAstError::new("invalid Ruby assignment", offset));
            }

            return Ok(Some((target, operator, value)));
        }

        scanner.consume(ch)?;
    }

    Ok(None)
}

fn assignment_operator_at(
    source: &str,
    offset: usize,
) -> Option<(&'static str, AssignmentOperator)> {
    let remaining = &source[offset..];

    for (raw, operator) in [
        ("||=", AssignmentOperator::Or),
        ("&&=", AssignmentOperator::And),
        ("+=", AssignmentOperator::Add),
        ("-=", AssignmentOperator::Subtract),
        ("*=", AssignmentOperator::Multiply),
        ("/=", AssignmentOperator::Divide),
        ("%=", AssignmentOperator::Modulo),
        ("=", AssignmentOperator::Assign),
    ] {
        if remaining.starts_with(raw) && is_assignment_boundary(source, offset, raw.len()) {
            return Some((raw, operator));
        }
    }

    None
}

fn is_assignment_boundary(source: &str, offset: usize, len: usize) -> bool {
    let before = source[..offset].chars().next_back();
    let after = source[offset + len..].chars().next();

    if len == 1 {
        !matches!(
            before,
            Some('=' | '!' | '<' | '>' | '&' | '|' | '+' | '-' | '*' | '/' | '%')
        ) && !matches!(after, Some('=' | '>'))
    } else {
        true
    }
}

fn parse_block_suffix(suffix: &str, offset: usize) -> Result<Block, RubyAstError> {
    if suffix.starts_with('{') {
        return parse_brace_block_suffix(suffix, offset);
    }

    let rest = suffix
        .strip_prefix("do")
        .ok_or_else(|| RubyAstError::new("expected `do` block suffix", offset))?
        .trim();

    if rest.is_empty() {
        return Ok(Block { params: Vec::new() });
    }

    if !rest.starts_with('|') || !rest.ends_with('|') {
        return Err(RubyAstError::new(
            "only `do |param|` block suffixes are supported",
            offset,
        ));
    }

    let params = parse_block_params(&rest[1..rest.len() - 1], offset)?;

    Ok(Block { params })
}

fn parse_brace_block_suffix(suffix: &str, offset: usize) -> Result<Block, RubyAstError> {
    if !is_balanced_brace_block_suffix(suffix) {
        return Err(RubyAstError::new("unbalanced `{` block suffix", offset));
    }

    let body = suffix[1..suffix.len() - 1].trim();
    if !body.starts_with('|') {
        return Ok(Block { params: Vec::new() });
    }

    let Some(end) = body[1..].find('|').map(|index| index + 1) else {
        return Err(RubyAstError::new(
            "unterminated `{ |param| }` block parameters",
            offset,
        ));
    };

    let params = parse_block_params(&body[1..end], offset)?;

    Ok(Block { params })
}

fn parse_block_params(source: &str, offset: usize) -> Result<Vec<String>, RubyAstError> {
    let mut params = Vec::new();

    for param in source.split(',').map(str::trim) {
        if param.is_empty() {
            continue;
        }

        if !is_block_param_name(param) {
            return Err(RubyAstError::new(
                format!("unsupported Ruby block parameter `{param}`"),
                offset,
            ));
        }

        params.push(param.to_string());
    }

    Ok(params)
}

pub(super) fn is_block_param_name(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        && !is_keyword(value)
}

fn is_balanced_brace_block_suffix(suffix: &str) -> bool {
    if !suffix.starts_with('{') || !suffix.ends_with('}') {
        return false;
    }

    let mut scanner = Scanner::default();

    for (offset, ch) in suffix.char_indices() {
        if offset > 0 && scanner.is_top_level() {
            return false;
        }

        if scanner.consume(ch).is_err() {
            return false;
        }
    }

    scanner.is_top_level()
}

pub(super) fn is_keyword(value: &str) -> bool {
    matches!(
        value,
        "alias"
            | "and"
            | "begin"
            | "break"
            | "case"
            | "class"
            | "def"
            | "do"
            | "else"
            | "elsif"
            | "end"
            | "ensure"
            | "false"
            | "for"
            | "if"
            | "in"
            | "module"
            | "next"
            | "nil"
            | "not"
            | "or"
            | "redo"
            | "rescue"
            | "return"
            | "self"
            | "super"
            | "then"
            | "true"
            | "unless"
            | "until"
            | "when"
            | "while"
            | "yield"
    )
}

#[derive(Default)]
struct Scanner {
    stack: Vec<char>,
    string: Option<char>,
    escaped: bool,
}

impl Scanner {
    fn is_top_level(&self) -> bool {
        self.string.is_none() && self.stack.is_empty()
    }

    fn consume(&mut self, ch: char) -> Result<(), RubyAstError> {
        if let Some(quote) = self.string {
            if self.escaped {
                self.escaped = false;
                return Ok(());
            }

            if ch == '\\' {
                self.escaped = true;
                return Ok(());
            }

            if ch == quote {
                self.string = None;
            }

            return Ok(());
        }

        match ch {
            '\'' | '"' => self.string = Some(ch),
            '(' | '[' | '{' => self.stack.push(ch),
            ')' if self.stack.pop() != Some('(') => {
                return Err(RubyAstError::new("unbalanced `)`", 0));
            }
            ']' if self.stack.pop() != Some('[') => {
                return Err(RubyAstError::new("unbalanced `]`", 0));
            }
            '}' if self.stack.pop() != Some('{') => {
                return Err(RubyAstError::new("unbalanced `}`", 0));
            }
            _ => {}
        }

        Ok(())
    }
}
