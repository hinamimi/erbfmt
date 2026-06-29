use super::{
    HtmlTag, HtmlToken, SpannedHtmlToken,
    tag::{find_tag_end, is_doctype, is_self_closing_tag_body, is_void_tag, tag_name},
    token::spanned_html_token,
};

pub fn tokenize(input: &str) -> Vec<HtmlToken> {
    tokenize_with_spans(input)
        .into_iter()
        .map(|spanned| spanned.token)
        .collect()
}

pub fn tokenize_with_spans(input: &str) -> Vec<SpannedHtmlToken> {
    let mut tokens = Vec::new();
    let mut cursor = 0;

    while let Some(relative_start) = input[cursor..].find('<') {
        let start = cursor + relative_start;

        if start > cursor {
            tokens.push(spanned_html_token(
                cursor,
                start,
                HtmlToken::Text(input[cursor..start].to_string()),
            ));
        }

        if input[start..].starts_with("<!--") {
            let Some(relative_end) = input[start + "<!--".len()..].find("-->") else {
                tokens.push(spanned_html_token(
                    start,
                    input.len(),
                    HtmlToken::Text(input[start..].to_string()),
                ));
                return tokens;
            };

            let end = start + "<!--".len() + relative_end + "-->".len();
            tokens.push(spanned_html_token(
                start,
                end,
                HtmlToken::Comment(input[start..end].to_string()),
            ));
            cursor = end;
            continue;
        }

        if input[start..].starts_with("<%") {
            let Some(relative_end) = input[start + "<%".len()..].find("%>") else {
                tokens.push(spanned_html_token(
                    start,
                    input.len(),
                    HtmlToken::Text(input[start..].to_string()),
                ));
                return tokens;
            };

            let end = start + "<%".len() + relative_end + "%>".len();
            tokens.push(spanned_html_token(
                start,
                end,
                HtmlToken::Text(input[start..end].to_string()),
            ));
            cursor = end;
            continue;
        }

        let Some(end) = find_tag_end(input, start) else {
            tokens.push(spanned_html_token(
                start,
                input.len(),
                HtmlToken::Text(input[start..].to_string()),
            ));
            return tokens;
        };

        let raw = &input[start..end];
        let body = input[start + '<'.len_utf8()..end - '>'.len_utf8()].trim();

        if body.starts_with('%') || body.starts_with('?') || body.starts_with('!') {
            if is_doctype(body) {
                tokens.push(spanned_html_token(
                    start,
                    end,
                    HtmlToken::Doctype(raw.to_string()),
                ));
            } else {
                tokens.push(spanned_html_token(
                    start,
                    end,
                    HtmlToken::Text(raw.to_string()),
                ));
            }

            cursor = end;
            continue;
        }

        if let Some(close_body) = body.strip_prefix('/') {
            tokens.push(spanned_html_token(
                start,
                end,
                HtmlToken::CloseTag(HtmlTag {
                    name: tag_name(close_body).to_string(),
                    raw: raw.to_string(),
                }),
            ));
        } else {
            let name = tag_name(body).to_string();
            let tag = HtmlTag {
                name,
                raw: raw.to_string(),
            };

            if is_self_closing_tag_body(body) {
                tokens.push(spanned_html_token(
                    start,
                    end,
                    HtmlToken::SelfClosingTag(tag),
                ));
            } else if is_void_tag(&tag.name) {
                tokens.push(spanned_html_token(start, end, HtmlToken::VoidTag(tag)));
            } else {
                let raw_text_tag_name = tag.name.clone();
                tokens.push(spanned_html_token(start, end, HtmlToken::OpenTag(tag)));

                if is_raw_text_element(&raw_text_tag_name) {
                    let Some((close_start, close_end)) =
                        find_raw_text_close_tag(input, end, &raw_text_tag_name)
                    else {
                        if end < input.len() {
                            tokens.push(spanned_html_token(
                                end,
                                input.len(),
                                HtmlToken::Text(input[end..].to_string()),
                            ));
                        }
                        return tokens;
                    };

                    if end < close_start {
                        tokens.push(spanned_html_token(
                            end,
                            close_start,
                            HtmlToken::Text(input[end..close_start].to_string()),
                        ));
                    }

                    let close_raw = &input[close_start..close_end];
                    let close_body =
                        input[close_start + '<'.len_utf8()..close_end - '>'.len_utf8()].trim();
                    let close_name = close_body
                        .strip_prefix('/')
                        .map(tag_name)
                        .unwrap_or_default()
                        .to_string();
                    tokens.push(spanned_html_token(
                        close_start,
                        close_end,
                        HtmlToken::CloseTag(HtmlTag {
                            name: close_name,
                            raw: close_raw.to_string(),
                        }),
                    ));
                    cursor = close_end;
                    continue;
                }
            }
        }

        cursor = end;
    }

    if cursor < input.len() {
        tokens.push(spanned_html_token(
            cursor,
            input.len(),
            HtmlToken::Text(input[cursor..].to_string()),
        ));
    }

    tokens
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
