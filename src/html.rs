#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlToken {
    Text(String),
    OpenTag(HtmlTag),
    CloseTag(HtmlTag),
    SelfClosingTag(HtmlTag),
    VoidTag(HtmlTag),
    Comment(String),
    Doctype(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlTag {
    pub name: String,
    pub raw: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HtmlSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedHtmlToken {
    pub token: HtmlToken,
    pub span: HtmlSpan,
}

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

fn spanned_html_token(start: usize, end: usize, token: HtmlToken) -> SpannedHtmlToken {
    SpannedHtmlToken {
        token,
        span: HtmlSpan { start, end },
    }
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

fn tag_name(body: &str) -> &str {
    body.split(|c: char| c.is_whitespace() || c == '/')
        .next()
        .unwrap_or("")
}

fn is_doctype(body: &str) -> bool {
    body.eq_ignore_ascii_case("!doctype html")
        || body
            .get(0.."!doctype".len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("!doctype"))
}

fn is_void_tag(name: &str) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_text_and_basic_tags() {
        let tokens = tokenize("<div>Hello</div>");

        assert_eq!(
            tokens,
            vec![
                HtmlToken::OpenTag(HtmlTag {
                    name: "div".to_string(),
                    raw: "<div>".to_string()
                }),
                HtmlToken::Text("Hello".to_string()),
                HtmlToken::CloseTag(HtmlTag {
                    name: "div".to_string(),
                    raw: "</div>".to_string()
                })
            ]
        );
    }

    #[test]
    fn tokenizes_attributes_without_parsing_them() {
        let tokens = tokenize(r#"<article class="card" data-id="1">"#);

        assert_eq!(
            tokens,
            vec![HtmlToken::OpenTag(HtmlTag {
                name: "article".to_string(),
                raw: r#"<article class="card" data-id="1">"#.to_string()
            })]
        );
    }

    #[test]
    fn tokenizes_attributes_with_erb_output() {
        let tokens = tokenize(r#"<a href="/users/<%= user.id %>">Profile</a>"#);

        assert_eq!(
            tokens,
            vec![
                HtmlToken::OpenTag(HtmlTag {
                    name: "a".to_string(),
                    raw: r#"<a href="/users/<%= user.id %>">"#.to_string()
                }),
                HtmlToken::Text("Profile".to_string()),
                HtmlToken::CloseTag(HtmlTag {
                    name: "a".to_string(),
                    raw: "</a>".to_string()
                })
            ]
        );
    }

    #[test]
    fn tokenizes_self_closing_and_void_tags() {
        let tokens = tokenize(r#"<img src="avatar.png"><custom />"#);

        assert_eq!(
            tokens,
            vec![
                HtmlToken::VoidTag(HtmlTag {
                    name: "img".to_string(),
                    raw: r#"<img src="avatar.png">"#.to_string()
                }),
                HtmlToken::SelfClosingTag(HtmlTag {
                    name: "custom".to_string(),
                    raw: "<custom />".to_string()
                })
            ]
        );
    }

    #[test]
    fn tokenizes_with_relative_spans() {
        let tokens = tokenize_with_spans("Hi <center>Legacy</center>");

        assert_eq!(
            tokens,
            vec![
                SpannedHtmlToken {
                    token: HtmlToken::Text("Hi ".to_string()),
                    span: HtmlSpan { start: 0, end: 3 }
                },
                SpannedHtmlToken {
                    token: HtmlToken::OpenTag(HtmlTag {
                        name: "center".to_string(),
                        raw: "<center>".to_string()
                    }),
                    span: HtmlSpan { start: 3, end: 11 }
                },
                SpannedHtmlToken {
                    token: HtmlToken::Text("Legacy".to_string()),
                    span: HtmlSpan { start: 11, end: 17 }
                },
                SpannedHtmlToken {
                    token: HtmlToken::CloseTag(HtmlTag {
                        name: "center".to_string(),
                        raw: "</center>".to_string()
                    }),
                    span: HtmlSpan { start: 17, end: 26 }
                }
            ]
        );
    }

    #[test]
    fn tokenizes_comments_and_doctype() {
        let tokens = tokenize("<!DOCTYPE html><!-- greeting --><p>Hello</p>");

        assert_eq!(
            tokens,
            vec![
                HtmlToken::Doctype("<!DOCTYPE html>".to_string()),
                HtmlToken::Comment("<!-- greeting -->".to_string()),
                HtmlToken::OpenTag(HtmlTag {
                    name: "p".to_string(),
                    raw: "<p>".to_string()
                }),
                HtmlToken::Text("Hello".to_string()),
                HtmlToken::CloseTag(HtmlTag {
                    name: "p".to_string(),
                    raw: "</p>".to_string()
                })
            ]
        );
    }

    #[test]
    fn treats_erb_like_tags_as_text() {
        let tokens = tokenize("<p>Hello, <%= user.name %></p>");

        assert_eq!(
            tokens,
            vec![
                HtmlToken::OpenTag(HtmlTag {
                    name: "p".to_string(),
                    raw: "<p>".to_string()
                }),
                HtmlToken::Text("Hello, ".to_string()),
                HtmlToken::Text("<%= user.name %>".to_string()),
                HtmlToken::CloseTag(HtmlTag {
                    name: "p".to_string(),
                    raw: "</p>".to_string()
                })
            ]
        );
    }

    #[test]
    fn treats_unterminated_tags_as_text() {
        let tokens = tokenize("<div");

        assert_eq!(tokens, vec![HtmlToken::Text("<div".to_string())]);
    }
}
