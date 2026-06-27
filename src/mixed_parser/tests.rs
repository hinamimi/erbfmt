use super::*;
use crate::lexer::{
    ErbBlockKind, ErbTag, ErbTagClose, ErbTagOpen, ErbTagSyntax, tokenize, tokenize_with_spans,
};

fn tag(code: &str) -> ErbTag {
    ErbTag::new(
        code.to_string(),
        ErbTagSyntax {
            open: ErbTagOpen::Code,
            close: ErbTagClose::Normal,
        },
    )
}

fn output_tag(code: &str) -> ErbTag {
    ErbTag::new(
        code.to_string(),
        ErbTagSyntax {
            open: ErbTagOpen::Output,
            close: ErbTagClose::Normal,
        },
    )
}

fn end_tag() -> ErbTag {
    tag("end")
}

#[test]
fn keeps_absolute_ranges_for_spanned_html_subtrees() {
    let input = "<section>\n  <p>Hello</p>\n</section>\n";
    let tokens = tokenize_with_spans(input).unwrap();
    let document = parse_spanned(&tokens).unwrap();
    let section = &document.children[0];

    assert_eq!(
        section.source_range(),
        Some(SourceRange { start: 0, end: 35 })
    );

    let Node::HtmlElement { children, .. } = section.unspanned() else {
        panic!("section element expected");
    };
    let paragraph = children
        .iter()
        .find(|node| matches!(node.unspanned(), Node::HtmlElement { .. }))
        .expect("paragraph element exists");

    assert_eq!(
        paragraph.source_range(),
        Some(SourceRange { start: 12, end: 24 })
    );
}

#[test]
fn keeps_absolute_ranges_for_spanned_erb_blocks_and_comments() {
    let input = "<%# note %>\n<% if user %>\n<p>Hello</p>\n<% end %>\n";
    let tokens = tokenize_with_spans(input).unwrap();
    let document = parse_spanned(&tokens).unwrap();
    let comment = &document.children[0];
    let block = document
        .children
        .iter()
        .find(|node| matches!(node.unspanned(), Node::ErbBlock { .. }))
        .expect("ERB block exists");

    assert_eq!(
        comment.source_range(),
        Some(SourceRange { start: 0, end: 11 })
    );
    assert!(matches!(comment.unspanned(), Node::ErbComment(comment) if comment.code == "note"));
    assert_eq!(
        block.source_range(),
        Some(SourceRange { start: 12, end: 48 })
    );
}

#[test]
fn parses_nested_html_elements() {
    let tokens = tokenize("<div><p>Hello</p></div>").unwrap();
    let document = parse(&tokens).unwrap();

    assert_eq!(
        document,
        Document {
            children: vec![Node::HtmlElement {
                name: "div".to_string(),
                open: "<div>".to_string(),
                close: "</div>".to_string(),
                children: vec![Node::HtmlElement {
                    name: "p".to_string(),
                    open: "<p>".to_string(),
                    close: "</p>".to_string(),
                    children: vec![Node::HtmlText("Hello".to_string())]
                }]
            }]
        }
    );
}

#[test]
fn allows_html_optional_closing_tags_when_configured() {
    let tokens = tokenize("<ul><li>one<li>two</ul>").unwrap();
    let document = parse_with_options(
        &tokens,
        ParserOptions {
            allow_html_optional_closing_tags: true,
        },
    )
    .unwrap();

    assert_eq!(
        document,
        Document {
            children: vec![Node::HtmlElement {
                name: "ul".to_string(),
                open: "<ul>".to_string(),
                close: "</ul>".to_string(),
                children: vec![
                    Node::HtmlElement {
                        name: "li".to_string(),
                        open: "<li>".to_string(),
                        close: String::new(),
                        children: vec![Node::HtmlText("one".to_string())]
                    },
                    Node::HtmlElement {
                        name: "li".to_string(),
                        open: "<li>".to_string(),
                        close: String::new(),
                        children: vec![Node::HtmlText("two".to_string())]
                    }
                ]
            }]
        }
    );
}

#[test]
fn allows_paragraph_optional_close_before_block_elements_when_configured() {
    let tokens = tokenize("<section><p>Hello<div>Block</div></section>").unwrap();
    let document = parse_with_options(
        &tokens,
        ParserOptions {
            allow_html_optional_closing_tags: true,
        },
    )
    .unwrap();

    assert_eq!(
        document,
        Document {
            children: vec![Node::HtmlElement {
                name: "section".to_string(),
                open: "<section>".to_string(),
                close: "</section>".to_string(),
                children: vec![
                    Node::HtmlElement {
                        name: "p".to_string(),
                        open: "<p>".to_string(),
                        close: String::new(),
                        children: vec![Node::HtmlText("Hello".to_string())]
                    },
                    Node::HtmlElement {
                        name: "div".to_string(),
                        open: "<div>".to_string(),
                        close: "</div>".to_string(),
                        children: vec![Node::HtmlText("Block".to_string())]
                    }
                ]
            }]
        }
    );
}

#[test]
fn still_rejects_invalid_close_tags_when_optional_close_is_allowed() {
    let tokens = tokenize("<p>Hello</span>").unwrap();
    let error = parse_with_options(
        &tokens,
        ParserOptions {
            allow_html_optional_closing_tags: true,
        },
    )
    .unwrap_err();

    assert_eq!(
        error,
        ParseError::MismatchedHtmlCloseTag {
            expected: "p".to_string(),
            found: "span".to_string(),
            raw: "</span>".to_string(),
            location: None
        }
    );
}

#[test]
fn preserves_inline_erb_output_inside_html() {
    let tokens = tokenize("<p>Hello, <%= user.name %></p>").unwrap();
    let document = parse(&tokens).unwrap();

    assert_eq!(
        document,
        Document {
            children: vec![Node::HtmlElement {
                name: "p".to_string(),
                open: "<p>".to_string(),
                close: "</p>".to_string(),
                children: vec![
                    Node::HtmlText("Hello, ".to_string()),
                    Node::ErbOutput(output_tag("user.name"))
                ]
            }]
        }
    );
}

#[test]
fn preserves_erb_output_inside_html_attributes() {
    let tokens = tokenize(r#"<a href="/users/<%= user.id %>"><%= user.name %></a>"#).unwrap();
    let document = parse(&tokens).unwrap();

    assert_eq!(
        document,
        Document {
            children: vec![Node::HtmlElement {
                name: "a".to_string(),
                open: r#"<a href="/users/<%= user.id %>">"#.to_string(),
                close: "</a>".to_string(),
                children: vec![Node::ErbOutput(output_tag("user.name"))]
            }]
        }
    );
}

#[test]
fn keeps_erb_blocks_with_html_children() {
    let tokens = tokenize("<% if user %><ul><li>Hello</li></ul><% end %>").unwrap();
    let document = parse(&tokens).unwrap();

    assert_eq!(
        document,
        Document {
            children: vec![Node::ErbBlock {
                kind: ErbBlockKind::If,
                tag: tag("if user"),
                output: false,
                end_tag: end_tag(),
                children: vec![Node::HtmlElement {
                    name: "ul".to_string(),
                    open: "<ul>".to_string(),
                    close: "</ul>".to_string(),
                    children: vec![Node::HtmlElement {
                        name: "li".to_string(),
                        open: "<li>".to_string(),
                        close: "</li>".to_string(),
                        children: vec![Node::HtmlText("Hello".to_string())]
                    }]
                }],
                branches: vec![]
            }]
        }
    );
}

#[test]
fn keeps_erb_branches_with_children() {
    let tokens = tokenize(
        "<% if admin? %><p>Admin</p><% elsif user? %><p>User</p><% else %><p>Guest</p><% end %>",
    )
    .unwrap();
    let document = parse(&tokens).unwrap();

    assert_eq!(
        document,
        Document {
            children: vec![Node::ErbBlock {
                kind: ErbBlockKind::If,
                tag: tag("if admin?"),
                output: false,
                end_tag: end_tag(),
                children: vec![Node::HtmlElement {
                    name: "p".to_string(),
                    open: "<p>".to_string(),
                    close: "</p>".to_string(),
                    children: vec![Node::HtmlText("Admin".to_string())]
                }],
                branches: vec![
                    ErbBranch {
                        kind: ErbBranchKind::Elsif,
                        tag: tag("elsif user?"),
                        children: vec![Node::HtmlElement {
                            name: "p".to_string(),
                            open: "<p>".to_string(),
                            close: "</p>".to_string(),
                            children: vec![Node::HtmlText("User".to_string())]
                        }]
                    },
                    ErbBranch {
                        kind: ErbBranchKind::Else,
                        tag: tag("else"),
                        children: vec![Node::HtmlElement {
                            name: "p".to_string(),
                            open: "<p>".to_string(),
                            close: "</p>".to_string(),
                            children: vec![Node::HtmlText("Guest".to_string())]
                        }]
                    }
                ]
            }]
        }
    );
}

#[test]
fn keeps_case_when_branches() {
    let tokens = tokenize(
        "<% case role %><% when \"admin\" %><p>Admin</p><% when \"user\" %><p>User</p><% end %>",
    )
    .unwrap();
    let document = parse(&tokens).unwrap();

    assert_eq!(
        document,
        Document {
            children: vec![Node::ErbBlock {
                kind: ErbBlockKind::Case,
                tag: tag("case role"),
                output: false,
                end_tag: end_tag(),
                children: vec![],
                branches: vec![
                    ErbBranch {
                        kind: ErbBranchKind::When,
                        tag: tag("when \"admin\""),
                        children: vec![Node::HtmlElement {
                            name: "p".to_string(),
                            open: "<p>".to_string(),
                            close: "</p>".to_string(),
                            children: vec![Node::HtmlText("Admin".to_string())]
                        }]
                    },
                    ErbBranch {
                        kind: ErbBranchKind::When,
                        tag: tag("when \"user\""),
                        children: vec![Node::HtmlElement {
                            name: "p".to_string(),
                            open: "<p>".to_string(),
                            close: "</p>".to_string(),
                            children: vec![Node::HtmlText("User".to_string())]
                        }]
                    }
                ]
            }]
        }
    );
}

#[test]
fn keeps_output_erb_do_blocks() {
    let tokens = tokenize("<%= form_with model: user do |form| %><p>Hello</p><% end %>").unwrap();
    let document = parse(&tokens).unwrap();

    assert_eq!(
        document,
        Document {
            children: vec![Node::ErbBlock {
                kind: ErbBlockKind::Do,
                tag: output_tag("form_with model: user do |form|"),
                output: true,
                end_tag: end_tag(),
                children: vec![Node::HtmlElement {
                    name: "p".to_string(),
                    open: "<p>".to_string(),
                    close: "</p>".to_string(),
                    children: vec![Node::HtmlText("Hello".to_string())]
                }],
                branches: vec![]
            }]
        }
    );
}

#[test]
fn keeps_begin_rescue_ensure_branches() {
    let tokens = tokenize(
            "<% begin %><p>Saving</p><% rescue => error %><p>Failed</p><% ensure %><p>Done</p><% end %>",
        )
        .unwrap();
    let document = parse(&tokens).unwrap();

    assert_eq!(
        document,
        Document {
            children: vec![Node::ErbBlock {
                kind: ErbBlockKind::Begin,
                tag: tag("begin"),
                output: false,
                end_tag: end_tag(),
                children: vec![Node::HtmlElement {
                    name: "p".to_string(),
                    open: "<p>".to_string(),
                    close: "</p>".to_string(),
                    children: vec![Node::HtmlText("Saving".to_string())]
                }],
                branches: vec![
                    ErbBranch {
                        kind: ErbBranchKind::Rescue,
                        tag: tag("rescue => error"),
                        children: vec![Node::HtmlElement {
                            name: "p".to_string(),
                            open: "<p>".to_string(),
                            close: "</p>".to_string(),
                            children: vec![Node::HtmlText("Failed".to_string())]
                        }]
                    },
                    ErbBranch {
                        kind: ErbBranchKind::Ensure,
                        tag: tag("ensure"),
                        children: vec![Node::HtmlElement {
                            name: "p".to_string(),
                            open: "<p>".to_string(),
                            close: "</p>".to_string(),
                            children: vec![Node::HtmlText("Done".to_string())]
                        }]
                    }
                ]
            }]
        }
    );
}

#[test]
fn preserves_void_comments_and_doctype() {
    let tokens = tokenize("<!DOCTYPE html><!-- hi --><img src=\"x.png\">").unwrap();
    let document = parse(&tokens).unwrap();

    assert_eq!(
        document,
        Document {
            children: vec![
                Node::HtmlDoctype("<!DOCTYPE html>".to_string()),
                Node::HtmlComment("<!-- hi -->".to_string()),
                Node::HtmlVoid {
                    name: "img".to_string(),
                    raw: "<img src=\"x.png\">".to_string()
                }
            ]
        }
    );
}

#[test]
fn reports_unexpected_html_close_tag() {
    let tokens = tokenize("</div>").unwrap();
    let error = parse(&tokens).unwrap_err();

    assert_eq!(
        error,
        ParseError::UnexpectedHtmlCloseTag {
            name: "div".to_string(),
            raw: "</div>".to_string(),
            location: None
        }
    );
}

#[test]
fn reports_mismatched_html_close_tag() {
    let tokens = tokenize("<div></span>").unwrap();
    let error = parse(&tokens).unwrap_err();

    assert_eq!(
        error,
        ParseError::MismatchedHtmlCloseTag {
            expected: "div".to_string(),
            found: "span".to_string(),
            raw: "</span>".to_string(),
            location: None
        }
    );
}

#[test]
fn reports_unclosed_html_tag() {
    let tokens = tokenize("<div><p>Hello</p>").unwrap();
    let error = parse(&tokens).unwrap_err();

    assert_eq!(
        error,
        ParseError::UnclosedHtmlTag {
            name: "div".to_string(),
            raw: "<div>".to_string(),
            location: None
        }
    );
}

#[test]
fn reports_unclosed_html_before_erb_block_end() {
    let tokens = tokenize("<% if user %><div><% end %>").unwrap();
    let error = parse(&tokens).unwrap_err();

    assert_eq!(
        error,
        ParseError::UnclosedHtmlTag {
            name: "div".to_string(),
            raw: "<div>".to_string(),
            location: None
        }
    );
}
