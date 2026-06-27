pub(super) fn find_tag_end(input: &str, start: usize) -> Option<usize> {
    let mut cursor = start + '<'.len_utf8();
    let mut quote = None;

    while cursor < input.len() {
        if input[cursor..].starts_with("<%") {
            let erb_code_start = cursor + "<%".len();
            let relative_erb_end = input[erb_code_start..].find("%>")?;
            cursor = erb_code_start + relative_erb_end + "%>".len();
            continue;
        }

        let ch = input[cursor..]
            .chars()
            .next()
            .expect("cursor is inside input");

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

pub(super) fn tag_name(body: &str) -> &str {
    body.split(|c: char| c.is_whitespace() || c == '/')
        .next()
        .unwrap_or("")
}

pub(super) fn is_self_closing_tag_body(body: &str) -> bool {
    let body = body.trim_end();

    if !body.ends_with('/') {
        return false;
    }

    let before_slash = body[..body.len() - '/'.len_utf8()].trim_end();

    if before_slash.is_empty() {
        return false;
    }

    if !before_slash.chars().any(char::is_whitespace) {
        return true;
    }

    before_slash
        .chars()
        .next_back()
        .is_some_and(|ch| ch.is_whitespace() || matches!(ch, '"' | '\''))
}

pub(super) fn is_doctype(body: &str) -> bool {
    body.eq_ignore_ascii_case("!doctype html")
        || body
            .get(0.."!doctype".len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("!doctype"))
}

pub(super) fn is_void_tag(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}
