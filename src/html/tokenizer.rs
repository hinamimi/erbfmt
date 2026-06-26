use super::{
    HtmlTag, HtmlToken, SpannedHtmlToken,
    tag::{find_tag_end, is_doctype, is_void_tag, tag_name},
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

            if body.ends_with('/') {
                tokens.push(spanned_html_token(
                    start,
                    end,
                    HtmlToken::SelfClosingTag(tag),
                ));
            } else if is_void_tag(&tag.name) {
                tokens.push(spanned_html_token(start, end, HtmlToken::VoidTag(tag)));
            } else {
                tokens.push(spanned_html_token(start, end, HtmlToken::OpenTag(tag)));
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
