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

pub fn tokenize(input: &str) -> Vec<HtmlToken> {
    let mut tokens = Vec::new();
    let mut cursor = 0;

    while let Some(relative_start) = input[cursor..].find('<') {
        let start = cursor + relative_start;

        if start > cursor {
            tokens.push(HtmlToken::Text(input[cursor..start].to_string()));
        }

        if input[start..].starts_with("<!--") {
            let Some(relative_end) = input[start + "<!--".len()..].find("-->") else {
                tokens.push(HtmlToken::Text(input[start..].to_string()));
                return tokens;
            };

            let end = start + "<!--".len() + relative_end + "-->".len();
            tokens.push(HtmlToken::Comment(input[start..end].to_string()));
            cursor = end;
            continue;
        }

        if input[start..].starts_with("<%") {
            let Some(relative_end) = input[start + "<%".len()..].find("%>") else {
                tokens.push(HtmlToken::Text(input[start..].to_string()));
                return tokens;
            };

            let end = start + "<%".len() + relative_end + "%>".len();
            tokens.push(HtmlToken::Text(input[start..end].to_string()));
            cursor = end;
            continue;
        }

        let Some(end) = find_tag_end(input, start) else {
            tokens.push(HtmlToken::Text(input[start..].to_string()));
            return tokens;
        };

        let raw = &input[start..end];
        let body = input[start + '<'.len_utf8()..end - '>'.len_utf8()].trim();

        if body.starts_with('%') || body.starts_with('?') || body.starts_with('!') {
            if is_doctype(body) {
                tokens.push(HtmlToken::Doctype(raw.to_string()));
            } else {
                tokens.push(HtmlToken::Text(raw.to_string()));
            }

            cursor = end;
            continue;
        }

        if let Some(close_body) = body.strip_prefix('/') {
            tokens.push(HtmlToken::CloseTag(HtmlTag {
                name: tag_name(close_body).to_string(),
                raw: raw.to_string(),
            }));
        } else {
            let name = tag_name(body).to_string();
            let tag = HtmlTag {
                name,
                raw: raw.to_string(),
            };

            if body.ends_with('/') {
                tokens.push(HtmlToken::SelfClosingTag(tag));
            } else if is_void_tag(&tag.name) {
                tokens.push(HtmlToken::VoidTag(tag));
            } else {
                tokens.push(HtmlToken::OpenTag(tag));
            }
        }

        cursor = end;
    }

    if cursor < input.len() {
        tokens.push(HtmlToken::Text(input[cursor..].to_string()));
    }

    tokens
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
