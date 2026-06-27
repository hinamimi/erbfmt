pub(super) fn fold_command_call(code: &str) -> Option<Vec<String>> {
    let code = code.trim();

    if code.is_empty() || code.contains('#') || code.contains(';') {
        return None;
    }

    let (call, block_suffix) = split_top_level_do_block(code)?;
    let (callee, arguments) = split_call(call)?;
    let arguments = split_top_level_arguments(arguments)?;

    let mut argument_lines = Vec::new();
    let mut has_multiline_argument = false;

    for (index, argument) in arguments.iter().enumerate() {
        let comma = index + 1 < arguments.len();
        let lines = format_argument_lines(argument, comma);

        has_multiline_argument |= lines.len() > 1;
        argument_lines.extend(lines);
    }

    if arguments.len() < 2 && !has_multiline_argument {
        return None;
    }

    let mut lines = vec![format!("{callee}(")];
    lines.extend(argument_lines);

    match block_suffix {
        Some(suffix) => lines.push(format!(") {suffix}")),
        None => lines.push(")".to_string()),
    }

    Some(lines)
}

fn format_argument_lines(argument: &str, comma: bool) -> Vec<String> {
    let mut lines = fold_keyword_hash_argument(argument)
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
        let mut line = format!("  {entry}");

        if index + 1 < entries.len() {
            line.push(',');
        }

        lines.push(line);
    }

    lines.push("}".to_string());
    Some(lines)
}

fn split_keyword_hash_argument(argument: &str) -> Option<(&str, &str)> {
    let mut state = RubyScanState::default();

    for (index, ch) in argument.char_indices() {
        if state.is_top_level() && ch == ':' {
            let keyword = argument[..index].trim();
            let hash = argument[index + ch.len_utf8()..].trim();

            return (is_keyword_argument_name(keyword) && is_single_hash_literal(hash))
                .then_some((keyword, hash));
        }

        if !state.consume(ch) {
            return None;
        }
    }

    None
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

fn is_single_hash_literal(value: &str) -> bool {
    if !value.starts_with('{') || !value.ends_with('}') {
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

fn trim_blank_edges(mut lines: Vec<&str>) -> Vec<&str> {
    while lines.first().is_some_and(|line| line.trim().is_empty()) {
        lines.remove(0);
    }

    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }

    lines
}

fn leading_whitespace_count(line: &str) -> usize {
    line.chars().take_while(|ch| ch.is_whitespace()).count()
}

fn strip_leading_whitespace(line: &str, count: usize) -> &str {
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
        || !is_foldable_callee(callee)
    {
        return None;
    }

    Some((callee, arguments))
}

fn split_top_level_do_block(code: &str) -> Option<(&str, Option<&str>)> {
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
        }

        if !state.consume(ch) {
            return None;
        }
    }

    state.is_balanced().then_some((code, None))
}

fn split_top_level_arguments(arguments: &str) -> Option<Vec<String>> {
    let mut state = RubyScanState::default();
    let mut start = 0;
    let mut result = Vec::new();

    for (index, ch) in arguments.char_indices() {
        if state.is_top_level() && ch == ',' {
            result.push(arguments[start..index].trim().to_string());
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

    result.push(arguments[start..].trim().to_string());

    if result.iter().any(String::is_empty) {
        return None;
    }

    Some(result)
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

#[derive(Default)]
struct RubyScanState {
    stack: Vec<char>,
    string: Option<char>,
    escaped: bool,
}

impl RubyScanState {
    fn is_top_level(&self) -> bool {
        self.string.is_none() && self.stack.is_empty()
    }

    fn is_balanced(&self) -> bool {
        self.string.is_none() && self.stack.is_empty() && !self.escaped
    }

    fn consume(&mut self, ch: char) -> bool {
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

#[cfg(test)]
mod tests {
    use super::fold_command_call;

    #[test]
    fn folds_command_call_arguments() {
        assert_eq!(
            fold_command_call(r#"link_to "Edit profile", edit_user_path(user), class: "button""#),
            Some(vec![
                "link_to(".to_string(),
                r#"  "Edit profile","#.to_string(),
                "  edit_user_path(user),".to_string(),
                r#"  class: "button""#.to_string(),
                ")".to_string(),
            ])
        );
    }

    #[test]
    fn folds_command_call_with_do_block() {
        assert_eq!(
            fold_command_call(r#"form_with model: user, url: user_path(user) do |form|"#),
            Some(vec![
                "form_with(".to_string(),
                "  model: user,".to_string(),
                "  url: user_path(user)".to_string(),
                ") do |form|".to_string(),
            ])
        );
    }

    #[test]
    fn folds_parenthesized_call_arguments() {
        assert_eq!(
            fold_command_call(
                r#"video_tag(["intro.mp4", "intro.webm"], controls: true, class: "hero-video")"#
            ),
            Some(vec![
                "video_tag(".to_string(),
                r#"  ["intro.mp4", "intro.webm"],"#.to_string(),
                "  controls: true,".to_string(),
                r#"  class: "hero-video""#.to_string(),
                ")".to_string(),
            ])
        );
    }

    #[test]
    fn folds_parenthesized_call_with_do_block() {
        assert_eq!(
            fold_command_call("form_with(model: user, url: user_path(user)) do |form|"),
            Some(vec![
                "form_with(".to_string(),
                "  model: user,".to_string(),
                "  url: user_path(user)".to_string(),
                ") do |form|".to_string(),
            ])
        );
    }

    #[test]
    fn folds_keyword_hash_arguments() {
        assert_eq!(
            fold_command_call(
                r#"render partial: "profile", locals: { current_user: current_user, account: account, selected_status: selected_status }"#
            ),
            Some(vec![
                "render(".to_string(),
                r#"  partial: "profile","#.to_string(),
                "  locals: {".to_string(),
                "    current_user: current_user,".to_string(),
                "    account: account,".to_string(),
                "    selected_status: selected_status".to_string(),
                "  }".to_string(),
                ")".to_string(),
            ])
        );
    }

    #[test]
    fn folds_single_keyword_hash_argument() {
        assert_eq!(
            fold_command_call(
                r#"render locals: { current_user: current_user, account: account, selected_status: selected_status }"#
            ),
            Some(vec![
                "render(".to_string(),
                "  locals: {".to_string(),
                "    current_user: current_user,".to_string(),
                "    account: account,".to_string(),
                "    selected_status: selected_status".to_string(),
                "  }".to_string(),
                ")".to_string(),
            ])
        );
    }

    #[test]
    fn folds_existing_multiline_parenthesized_call_arguments() {
        assert_eq!(
            fold_command_call(
                "react_component(\"ReactComponent\",\n  props: {\n    key1: \"value1\",\n    key2: \"value2\"\n  }\n)"
            ),
            Some(vec![
                "react_component(".to_string(),
                r#"  "ReactComponent","#.to_string(),
                "  props: {".to_string(),
                r#"    key1: "value1","#.to_string(),
                r#"    key2: "value2""#.to_string(),
                "  }".to_string(),
                ")".to_string(),
            ])
        );
    }

    #[test]
    fn ignores_commas_inside_strings_and_nested_values() {
        assert_eq!(
            fold_command_call(r#"link_to "Edit, profile", user_path(user, anchor: "top")"#),
            Some(vec![
                "link_to(".to_string(),
                r#"  "Edit, profile","#.to_string(),
                r#"  user_path(user, anchor: "top")"#.to_string(),
                ")".to_string(),
            ])
        );
    }

    #[test]
    fn does_not_fold_control_flow() {
        assert_eq!(
            fold_command_call("if current_user.admin? && account.active?"),
            None
        );
    }

    #[test]
    fn does_not_fold_single_argument_calls() {
        assert_eq!(
            fold_command_call(r#"cache ["profile-card", user.cache_key_with_version]"#),
            None
        );
    }

    #[test]
    fn does_not_fold_single_entry_keyword_hash_arguments() {
        assert_eq!(fold_command_call(r#"render locals: { user: user }"#), None);
    }

    #[test]
    fn does_not_fold_unbalanced_code() {
        assert_eq!(
            fold_command_call(r#"link_to "Edit", edit_user_path(user"#),
            None
        );
    }

    #[test]
    fn does_not_fold_parenthesized_call_with_trailing_expression() {
        assert_eq!(
            fold_command_call(r#"image_tag("profile.png", alt: "Profile") || fallback"#),
            None
        );
    }
}
