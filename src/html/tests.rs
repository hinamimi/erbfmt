use super::token::HtmlSpan;
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
