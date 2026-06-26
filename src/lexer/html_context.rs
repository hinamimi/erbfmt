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
