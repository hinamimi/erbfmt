pub(super) fn is_inside_html_tag(input: &str, position: usize) -> bool {
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

pub(super) fn is_inside_raw_text_element(input: &str, position: usize) -> bool {
    let mut cursor = 0;
    let mut raw_element: Option<String> = None;

    while cursor < position {
        if let Some(name) = raw_element.as_deref() {
            let Some((close_start, close_end)) = find_raw_text_close_tag(input, cursor, name)
            else {
                return true;
            };

            if close_start >= position {
                return true;
            }

            raw_element = None;
            cursor = close_end;
            continue;
        }

        let Some(relative_start) = input[cursor..position].find('<') else {
            return false;
        };
        let start = cursor + relative_start;

        if input[start..].starts_with("<%") {
            let Some(relative_end) = input[start + "<%".len()..].find("%>") else {
                return false;
            };
            cursor = start + "<%".len() + relative_end + "%>".len();
            continue;
        }

        if input[start..].starts_with("<!--") {
            let Some(relative_end) = input[start + "<!--".len()..].find("-->") else {
                return false;
            };
            cursor = start + "<!--".len() + relative_end + "-->".len();
            continue;
        }

        let Some(end) = find_tag_end(input, start) else {
            return false;
        };

        if end > position {
            return false;
        }

        let body = input[start + '<'.len_utf8()..end - '>'.len_utf8()].trim();
        if body.starts_with('/') {
            cursor = end;
            continue;
        }

        let name = tag_name(body);
        if is_raw_text_element(name) {
            raw_element = Some(name.to_ascii_lowercase());
        }

        cursor = end;
    }

    raw_element.is_some()
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

fn find_tag_end(input: &str, start: usize) -> Option<usize> {
    let mut cursor = start + '<'.len_utf8();
    let mut quote = None;

    while cursor < input.len() {
        if input[cursor..].starts_with("<%") {
            let erb_code_start = cursor + "<%".len();
            let relative_erb_end = input[erb_code_start..].find("%>")?;
            cursor = erb_code_start + relative_erb_end + "%>".len();
            continue;
        }

        let ch = input[cursor..].chars().next()?;

        match quote {
            Some(active_quote) if ch == active_quote => quote = None,
            Some(_) => {}
            None if ch == '"' || ch == '\'' => quote = Some(ch),
            None if ch == '>' => return Some(cursor + ch.len_utf8()),
            None => {}
        }

        cursor += ch.len_utf8();
    }

    None
}

fn tag_name(body: &str) -> &str {
    body.split(|c: char| c.is_whitespace() || matches!(c, '/' | '<'))
        .next()
        .unwrap_or("")
}

fn is_raw_text_element(name: &str) -> bool {
    matches!(name.to_ascii_lowercase().as_str(), "script" | "style")
}

fn find_raw_text_close_tag(input: &str, cursor: usize, name: &str) -> Option<(usize, usize)> {
    let lower = input[cursor..].to_ascii_lowercase();
    let pattern = format!("</{}", name.to_ascii_lowercase());
    let mut relative_cursor = 0;

    while let Some(relative_start) = lower[relative_cursor..].find(&pattern) {
        let close_start = cursor + relative_cursor + relative_start;
        let name_end = close_start + pattern.len();

        if is_close_tag_name_boundary(input, name_end)
            && let Some(close_end) = find_tag_end(input, close_start)
        {
            return Some((close_start, close_end));
        }

        relative_cursor += relative_start + '<'.len_utf8();
    }

    None
}

fn is_close_tag_name_boundary(input: &str, index: usize) -> bool {
    input[index..]
        .chars()
        .next()
        .is_some_and(|ch| ch == '>' || ch.is_ascii_whitespace())
}
