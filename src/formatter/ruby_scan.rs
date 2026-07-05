#[derive(Default)]
pub(super) struct RubyScanState {
    stack: Vec<char>,
    string: Option<char>,
    escaped: bool,
}

impl RubyScanState {
    pub(super) fn is_top_level(&self) -> bool {
        self.string.is_none() && self.stack.is_empty()
    }

    pub(super) fn is_balanced(&self) -> bool {
        self.string.is_none() && self.stack.is_empty() && !self.escaped
    }

    pub(super) fn consume(&mut self, ch: char) -> bool {
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
            '(' | '[' | '{' => {
                self.stack.push(ch);
                true
            }
            ')' => self.stack.pop() == Some('('),
            ']' => self.stack.pop() == Some('['),
            '}' => self.stack.pop() == Some('{'),
            _ => true,
        }
    }
}

pub(super) fn contains_top_level_newline(value: &str) -> bool {
    let mut state = RubyScanState::default();

    for ch in value.chars() {
        if state.is_top_level() && ch == '\n' {
            return true;
        }

        if !state.consume(ch) {
            return true;
        }
    }

    false
}

pub(super) fn is_single_balanced_brace_suffix(value: &str) -> bool {
    is_single_balanced_wrapped_value(value, '{', '}')
}

pub(super) fn is_single_hash_literal(value: &str) -> bool {
    is_single_balanced_wrapped_value(value, '{', '}')
}

pub(super) fn is_single_array_literal(value: &str) -> bool {
    is_single_balanced_wrapped_value(value, '[', ']')
}

fn is_single_balanced_wrapped_value(value: &str, open: char, close: char) -> bool {
    if !value.starts_with(open) || !value.ends_with(close) {
        return false;
    }

    let mut state = RubyScanState::default();

    for (offset, ch) in value.char_indices() {
        if offset > 0 && state.is_top_level() {
            return false;
        }

        if !state.consume(ch) {
            return false;
        }
    }

    state.is_balanced()
}

pub(super) fn split_top_level_arguments(arguments: &str) -> Option<Vec<String>> {
    let mut state = RubyScanState::default();
    let mut start = 0;
    let mut result = Vec::new();

    for (index, ch) in arguments.char_indices() {
        if state.is_top_level() && ch == ',' {
            let argument = arguments[start..index].trim();
            if argument.is_empty() {
                return None;
            }

            result.push(argument.to_string());
            start = index + ch.len_utf8();
            continue;
        }

        if !state.consume(ch) {
            return None;
        }
    }

    if !state.is_balanced() {
        return None;
    }

    let last = arguments[start..].trim();
    if !last.is_empty() {
        result.push(last.to_string());
    }

    if result.is_empty() || result.iter().any(String::is_empty) {
        return None;
    }

    Some(result)
}

pub(super) fn trim_blank_edges(mut lines: Vec<&str>) -> Vec<&str> {
    while lines.first().is_some_and(|line| line.trim().is_empty()) {
        lines.remove(0);
    }

    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }

    lines
}

pub(super) fn leading_whitespace_count(line: &str) -> usize {
    line.chars().take_while(|ch| ch.is_whitespace()).count()
}

pub(super) fn strip_leading_whitespace(line: &str, count: usize) -> &str {
    if count == 0 {
        return line;
    }

    for (stripped, (index, ch)) in line.char_indices().enumerate() {
        if stripped == count || !ch.is_whitespace() {
            return &line[index..];
        }
    }

    ""
}
