use std::borrow::Cow;

use crate::syntax::ruby::{Expression, parse};

use super::ruby_scan::{
    RubyScanState, contains_top_level_newline, is_single_array_literal,
    is_single_balanced_brace_suffix, is_single_hash_literal, leading_whitespace_count,
    split_top_level_arguments, strip_leading_whitespace, trim_blank_edges,
};
use super::ruby_wrap_ast::{
    argument_contains_foldable_keyword_hash, expression_is_foldable, foldable_call,
};

pub(super) fn fold_command_call(code: &str) -> Option<Vec<String>> {
    let code = code.trim();

    if code.is_empty() || code.contains("<<") {
        return None;
    }

    let code = if code.contains('\n') && contains_top_level_newline(code) {
        Cow::Owned(normalize_multiline_command_call_source(code)?)
    } else {
        Cow::Borrowed(code)
    };
    let code = code.as_ref();

    let (code, modifier_suffix) = split_top_level_modifier_suffix(code)?;
    if let Some(suffix) = modifier_suffix
        && !modifier_suffix_condition_is_foldable(suffix)
    {
        return None;
    }

    let program = parse(code).ok()?;
    let parsed_call = foldable_call(&program)?;
    let (call, block_suffix) = split_top_level_block_suffix(code)?;
    let (callee, arguments) = split_call(call)?;
    let arguments = split_top_level_arguments(arguments)?;

    if parsed_call.callee != callee {
        return None;
    }

    let can_fold_single_argument = arguments.len() >= 2
        || parsed_call
            .arguments
            .iter()
            .any(argument_contains_foldable_keyword_hash);

    if !can_fold_single_argument {
        return None;
    }

    let mut argument_lines = Vec::new();
    for (index, argument) in arguments.iter().enumerate() {
        let comma = index + 1 < arguments.len();
        let lines = format_argument_lines(argument, comma);

        argument_lines.extend(lines);
    }

    let mut lines = vec![format!("{callee}(")];
    lines.extend(argument_lines);

    match (block_suffix, modifier_suffix) {
        (Some(block), Some(modifier)) => lines.push(format!(") {block} {modifier}")),
        (Some(block), None) => lines.push(format!(") {block}")),
        (None, Some(modifier)) => lines.push(format!(") {modifier}")),
        (None, None) => lines.push(")".to_string()),
    }

    Some(lines)
}

fn normalize_multiline_command_call_source(code: &str) -> Option<String> {
    if code.contains("=>") {
        return None;
    }

    let lines = trim_blank_edges(code.lines().collect());
    if lines.len() < 2 {
        return None;
    }

    let trimmed_lines = lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    if trimmed_lines.len() < 2
        || trimmed_lines.iter().any(|line| line.starts_with('#'))
        || !trimmed_lines[..trimmed_lines.len() - 1]
            .iter()
            .all(|line| line.ends_with(','))
    {
        return None;
    }

    let mut state = RubyScanState::default();
    let mut normalized = String::new();
    let mut last_was_space = false;
    let mut saw_top_level_newline = false;

    for ch in code.chars() {
        if state.is_top_level() && matches!(ch, '\n' | '\r') {
            saw_top_level_newline = true;
            if !last_was_space {
                normalized.push(' ');
                last_was_space = true;
            }
            continue;
        }

        if !state.consume(ch) {
            return None;
        }

        if ch.is_whitespace() {
            if !last_was_space {
                normalized.push(' ');
                last_was_space = true;
            }
        } else {
            normalized.push(ch);
            last_was_space = false;
        }
    }

    (saw_top_level_newline && state.is_balanced()).then(|| normalized.trim().to_string())
}

fn format_argument_lines(argument: &str, comma: bool) -> Vec<String> {
    let mut lines = fold_keyword_hash_argument(argument)
        .or_else(|| fold_keyword_array_argument(argument))
        .or_else(|| fold_keyword_call_value_argument(argument))
        .or_else(|| fold_chained_method_argument(argument))
        .unwrap_or_else(|| normalize_multiline_argument(argument));
    let last_index = lines.len().saturating_sub(1);

    for (index, line) in lines.iter_mut().enumerate() {
        *line = format!("  {line}");

        if comma && index == last_index {
            line.push(',');
        }
    }

    lines
}

fn fold_keyword_call_value_argument(argument: &str) -> Option<Vec<String>> {
    let (keyword, value) = split_label_value_argument(argument)?;

    if !value.contains('(') || (!value.contains('\n') && value.len() <= 80) {
        return None;
    }

    let mut lines = fold_command_call(value)?;
    if lines.len() < 2 {
        return None;
    }

    lines[0] = format!("{keyword}: {}", lines[0]);
    Some(lines)
}

fn fold_chained_method_argument(argument: &str) -> Option<Vec<String>> {
    if argument.contains('\n') || argument.len() <= 80 {
        return None;
    }

    let (prefix, expression) = split_label_value_argument(argument)
        .map(|(keyword, value)| (format!("{keyword}: "), value))
        .unwrap_or_else(|| (String::new(), argument.trim()));

    let program = parse(expression).ok()?;
    if !program.is_format_candidate() || !program.statements.iter().all(expression_is_foldable) {
        return None;
    }

    let parts = split_top_level_method_chain(expression)?;
    if parts.len() < 3 {
        return None;
    }

    let parts = compact_chained_method_segments(parts);
    let mut lines = Vec::with_capacity(parts.len());
    lines.push(format!("{prefix}{}", parts[0]));
    lines.extend(parts.into_iter().skip(1).map(|part| format!("  {part}")));

    Some(lines)
}

fn compact_chained_method_segments(parts: Vec<&str>) -> Vec<String> {
    let Some((base, segments)) = parts.split_first() else {
        return Vec::new();
    };

    let mut result = vec![(*base).to_string()];
    let mut index = 0;

    while index < segments.len() {
        let segment = segments[index];

        if index + 1 < segments.len()
            && is_plain_method_segment(segment)
            && method_segment_has_arguments(segments[index + 1])
        {
            result.push(format!("{segment}{}", segments[index + 1]));
            index += 2;
            continue;
        }

        result.push(segment.to_string());
        index += 1;
    }

    result
}

fn is_plain_method_segment(segment: &str) -> bool {
    segment
        .strip_prefix('.')
        .or_else(|| segment.strip_prefix("&."))
        .is_some_and(is_keyword_argument_name)
}

fn method_segment_has_arguments(segment: &str) -> bool {
    segment.contains('(')
}

fn split_label_value_argument(argument: &str) -> Option<(&str, &str)> {
    let mut state = RubyScanState::default();

    for (index, ch) in argument.char_indices() {
        if state.is_top_level() && ch == ':' {
            let keyword = argument[..index].trim();
            let value = argument[index + ch.len_utf8()..].trim();

            return (is_label_argument_name(keyword) && !value.is_empty())
                .then_some((keyword, value));
        }

        if !state.consume(ch) {
            return None;
        }
    }

    None
}

fn split_top_level_method_chain(expression: &str) -> Option<Vec<&str>> {
    let mut state = RubyScanState::default();
    let mut parts = Vec::new();
    let mut segment_start = 0;

    for (index, ch) in expression.char_indices() {
        if state.is_top_level() && is_method_chain_separator(expression, index, ch) {
            if parts.is_empty() {
                let base = expression[..index].trim();
                if base.is_empty() {
                    return None;
                }

                parts.push(base);
            } else {
                let segment = expression[segment_start..index].trim();
                if !is_method_chain_segment(segment) {
                    return None;
                }

                parts.push(segment);
            }

            segment_start = index;
        }

        if !state.consume(ch) {
            return None;
        }
    }

    if parts.is_empty() || !state.is_balanced() {
        return None;
    }

    let last = expression[segment_start..].trim();
    if !is_method_chain_segment(last) {
        return None;
    }

    parts.push(last);
    Some(parts)
}

fn is_method_chain_separator(expression: &str, index: usize, ch: char) -> bool {
    if expression[index..].starts_with("&.") {
        return true;
    }

    if ch != '.' {
        return false;
    }

    let previous = expression[..index].chars().next_back();
    let next = expression[index + '.'.len_utf8()..].chars().next();

    !matches!(previous, Some('.' | '&')) && !matches!(next, Some('.'))
}

fn is_method_chain_segment(segment: &str) -> bool {
    let Some(rest) = segment
        .strip_prefix('.')
        .or_else(|| segment.strip_prefix("&."))
    else {
        return false;
    };

    rest.chars()
        .next()
        .is_some_and(|ch| matches!(ch, 'a'..='z' | 'A'..='Z' | '_'))
}

fn fold_keyword_hash_argument(argument: &str) -> Option<Vec<String>> {
    if argument.contains('\n') {
        return None;
    }

    let (keyword, hash) = split_keyword_hash_argument(argument)?;
    let entries = split_top_level_arguments(&hash[1..hash.len() - 1])?;

    if entries.len() < 2 {
        return None;
    }

    let mut lines = vec![format!("{keyword}: {{")];

    for (index, entry) in entries.iter().enumerate() {
        lines.extend(format_hash_entry_lines(entry, index + 1 < entries.len()));
    }

    lines.push("}".to_string());
    Some(lines)
}

fn fold_keyword_array_argument(argument: &str) -> Option<Vec<String>> {
    if argument.contains('\n') || argument.len() <= 80 {
        return None;
    }

    let (keyword, value) = split_label_value_argument(argument)?;
    if !is_single_array_literal(value) {
        return None;
    }

    let program = parse(value).ok()?;
    if !matches!(
        program.statements.as_slice(),
        [Expression::Array(values)] if values.len() >= 2 && values.iter().all(expression_is_foldable)
    ) {
        return None;
    }

    let entries = split_top_level_arguments(&value[1..value.len() - 1])?;
    if entries.len() < 2 {
        return None;
    }

    let mut lines = vec![format!("{keyword}: [")];
    for (index, entry) in entries.iter().enumerate() {
        let mut line = format!("  {entry}");

        if index + 1 < entries.len() {
            line.push(',');
        }

        lines.push(line);
    }
    lines.push("]".to_string());

    Some(lines)
}

fn split_keyword_hash_argument(argument: &str) -> Option<(&str, &str)> {
    let mut state = RubyScanState::default();

    for (index, ch) in argument.char_indices() {
        if state.is_top_level() && ch == ':' {
            let keyword = argument[..index].trim();
            let hash = argument[index + ch.len_utf8()..].trim();

            return (is_label_argument_name(keyword) && is_single_hash_literal(hash))
                .then_some((keyword, hash));
        }

        if !state.consume(ch) {
            return None;
        }
    }

    None
}

fn format_hash_entry_lines(entry: &str, comma: bool) -> Vec<String> {
    let mut lines = fold_hash_entry_call_value(entry).unwrap_or_else(|| vec![format!("  {entry}")]);

    if comma && let Some(last) = lines.last_mut() {
        last.push(',');
    }

    lines
}

fn fold_hash_entry_call_value(entry: &str) -> Option<Vec<String>> {
    let (prefix, value) = split_hash_entry_value_argument(entry)?;

    if !value.contains('(') || (!value.contains('\n') && value.len() <= 80) {
        return None;
    }

    let mut lines = fold_command_call(value)?;
    if lines.len() < 2 {
        return None;
    }

    lines[0] = format!("  {prefix}{}", lines[0]);
    for line in lines.iter_mut().skip(1) {
        *line = format!("  {line}");
    }

    Some(lines)
}

fn split_hash_entry_value_argument(entry: &str) -> Option<(String, &str)> {
    if let Some((keyword, value)) = split_label_value_argument(entry) {
        return Some((format!("{keyword}: "), value));
    }

    let mut state = RubyScanState::default();

    for (index, ch) in entry.char_indices() {
        if state.is_top_level() && ch == '=' && entry[index..].starts_with("=>") {
            let key = entry[..index].trim();
            let value = entry[index + "=>".len()..].trim();

            return (!key.is_empty() && !value.is_empty()).then(|| (format!("{key} => "), value));
        }

        if !state.consume(ch) {
            return None;
        }
    }

    None
}

fn is_label_argument_name(value: &str) -> bool {
    is_keyword_argument_name(value) || is_quoted_label_argument_name(value)
}

fn is_quoted_label_argument_name(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !matches!(first, '\'' | '"') {
        return false;
    }

    value.len() >= 2 && value.ends_with(first)
}

fn is_keyword_argument_name(value: &str) -> bool {
    let Some(first) = value.chars().next() else {
        return false;
    };

    matches!(first, 'a'..='z' | 'A'..='Z' | '_')
        && value
            .chars()
            .all(|ch| matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_'))
}

fn normalize_multiline_argument(argument: &str) -> Vec<String> {
    let mut lines = trim_blank_edges(argument.lines().collect());
    let common_indent = common_argument_indent(&lines);

    lines
        .drain(..)
        .map(|line| {
            strip_leading_whitespace(line, common_indent)
                .trim_end()
                .to_string()
        })
        .collect()
}

fn common_argument_indent(lines: &[&str]) -> usize {
    let non_empty_lines = lines.iter().copied().filter(|line| !line.trim().is_empty());

    if lines
        .first()
        .is_some_and(|line| leading_whitespace_count(line) == 0)
    {
        let skipped_first = lines
            .iter()
            .copied()
            .skip(1)
            .filter(|line| !line.trim().is_empty())
            .map(leading_whitespace_count)
            .min();

        if let Some(indent) = skipped_first {
            return indent;
        }
    }

    non_empty_lines
        .map(leading_whitespace_count)
        .min()
        .unwrap_or(0)
}

fn split_call(code: &str) -> Option<(&str, &str)> {
    split_parenthesized_call(code).or_else(|| split_command_call(code))
}

fn split_parenthesized_call(code: &str) -> Option<(&str, &str)> {
    let open_at = code.find('(')?;
    let callee = &code[..open_at];

    if !is_foldable_callee(callee) || !code.ends_with(')') {
        return None;
    }

    let mut state = RubyScanState::default();

    for (offset, ch) in code[open_at..].char_indices() {
        if offset > 0 && state.is_top_level() {
            return None;
        }

        if !state.consume(ch) {
            return None;
        }
    }

    state
        .is_balanced()
        .then(|| (callee, &code[open_at + 1..code.len() - 1]))
}

fn split_command_call(code: &str) -> Option<(&str, &str)> {
    let split_at = code
        .char_indices()
        .find_map(|(index, ch)| ch.is_whitespace().then_some(index))?;
    let callee = code[..split_at].trim();
    let arguments = code[split_at..].trim();

    if callee.is_empty()
        || arguments.is_empty()
        || arguments.starts_with('(')
        || contains_top_level_newline(arguments)
        || !is_foldable_callee(callee)
    {
        return None;
    }

    Some((callee, arguments))
}

fn split_top_level_block_suffix(code: &str) -> Option<(&str, Option<&str>)> {
    let mut state = RubyScanState::default();

    for (index, ch) in code.char_indices() {
        if state.is_top_level() && ch.is_whitespace() {
            let rest = &code[index..];
            let trimmed = rest.trim_start();

            if let Some(after_do) = trimmed.strip_prefix("do")
                && (after_do.is_empty() || after_do.chars().next().is_some_and(char::is_whitespace))
            {
                return state
                    .is_balanced()
                    .then(|| (code[..index].trim_end(), Some(trimmed.trim())));
            }

            let before = code[..index].trim_end();
            if before.ends_with(')')
                && trimmed.starts_with('{')
                && is_single_balanced_brace_suffix(trimmed)
            {
                return state.is_balanced().then(|| (before, Some(trimmed.trim())));
            }
        }

        if !state.consume(ch) {
            return None;
        }
    }

    state.is_balanced().then_some((code, None))
}

fn split_top_level_modifier_suffix(code: &str) -> Option<(&str, Option<&str>)> {
    let mut state = RubyScanState::default();

    for (index, ch) in code.char_indices() {
        if state.is_top_level() && ch.is_whitespace() {
            let trimmed = code[index..].trim_start();

            if is_modifier_suffix(trimmed) {
                return state
                    .is_balanced()
                    .then(|| (code[..index].trim_end(), Some(trimmed)));
            }
        }

        if !state.consume(ch) {
            return None;
        }
    }

    state.is_balanced().then_some((code, None))
}

fn is_modifier_suffix(value: &str) -> bool {
    modifier_suffix_condition(value).is_some()
}

fn modifier_suffix_condition(value: &str) -> Option<&str> {
    ["if", "unless"].iter().find_map(|keyword| {
        value
            .strip_prefix(keyword)
            .and_then(|condition| {
                condition
                    .chars()
                    .next()
                    .is_some_and(char::is_whitespace)
                    .then_some(condition.trim())
            })
            .filter(|condition| !condition.is_empty())
    })
}

fn modifier_suffix_condition_is_foldable(value: &str) -> bool {
    let Some(condition) = modifier_suffix_condition(value) else {
        return false;
    };
    let Ok(program) = parse(condition) else {
        return false;
    };

    matches!(program.statements.as_slice(), [expression] if expression_is_foldable(expression))
}

fn is_foldable_callee(callee: &str) -> bool {
    let Some(first) = callee.chars().next() else {
        return false;
    };

    if !matches!(first, 'a'..='z' | 'A'..='Z' | '_' | '@') {
        return false;
    }

    if is_ruby_keyword(callee)
        || callee.ends_with('.')
        || callee.ends_with(':')
        || callee.contains("..")
        || callee.contains(":::")
    {
        return false;
    }

    callee.chars().all(|ch| {
        matches!(
            ch,
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.' | ':' | '?' | '!' | '@'
        )
    })
}

fn is_ruby_keyword(value: &str) -> bool {
    matches!(
        value,
        "BEGIN"
            | "END"
            | "alias"
            | "and"
            | "begin"
            | "break"
            | "case"
            | "class"
            | "def"
            | "defined?"
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
            | "retry"
            | "return"
            | "self"
            | "super"
            | "then"
            | "true"
            | "undef"
            | "unless"
            | "until"
            | "when"
            | "while"
            | "yield"
    )
}
