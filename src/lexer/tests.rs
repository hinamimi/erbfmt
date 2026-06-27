use super::*;

fn erb_tag(code: &str, open: ErbTagOpen, close: ErbTagClose) -> ErbTag {
    ErbTag::new(code.to_string(), ErbTagSyntax { open, close })
}

fn code_tag(code: &str) -> ErbTag {
    erb_tag(code, ErbTagOpen::Code, ErbTagClose::Normal)
}

fn output_tag(code: &str) -> ErbTag {
    erb_tag(code, ErbTagOpen::Output, ErbTagClose::Normal)
}

fn comment_tag(code: &str) -> ErbTag {
    erb_tag(code, ErbTagOpen::Comment, ErbTagClose::Normal)
}

#[test]
fn tokenize_html() {
    let tokens = tokenize("<div>Hello</div>").unwrap();

    assert_eq!(tokens, vec![Token::Html("<div>Hello</div>".to_string())]);
}

#[test]
fn tokenizes_empty_erb_code_tag() {
    let tokens = tokenize("<% %>").unwrap();

    assert_eq!(tokens, vec![Token::ErbCode(code_tag(""))]);
}

#[test]
fn tokenizes_erb_output_tag() {
    let tokens = tokenize("<%= user.name %>").unwrap();

    assert_eq!(tokens, vec![Token::ErbOutput(output_tag("user.name"))]);
}

#[test]
fn tokenizes_erb_comment_tag() {
    let tokens = tokenize("<%# erbfmt-ignore format: generated %>").unwrap();

    assert_eq!(
        tokens,
        vec![Token::ErbComment(comment_tag(
            "erbfmt-ignore format: generated"
        ))]
    );
}

#[test]
fn tokenizes_html_fragments_around_erb() {
    let tokens = tokenize("<p>Hello <%= user.name %></p>").unwrap();

    assert_eq!(
        tokens,
        vec![
            Token::Html("<p>Hello ".to_string()),
            Token::ErbOutput(output_tag("user.name")),
            Token::Html("</p>".to_string())
        ]
    );
}

#[test]
fn keeps_erb_output_inside_html_tag_attributes_as_html() {
    let tokens = tokenize(
        r#"<a href="/users/<%= user.id %>" aria-label="<%= user.name %>"><%= user.name %></a>"#,
    )
    .unwrap();

    assert_eq!(
        tokens,
        vec![
            Token::Html(
                r#"<a href="/users/<%= user.id %>" aria-label="<%= user.name %>">"#.to_string()
            ),
            Token::ErbOutput(output_tag("user.name")),
            Token::Html("</a>".to_string())
        ]
    );
}

#[test]
fn tokenizes_erb_after_non_tag_less_than_sign() {
    let tokens = tokenize("2 < 3 <%= result %>").unwrap();

    assert_eq!(
        tokens,
        vec![
            Token::Html("2 < 3 ".to_string()),
            Token::ErbOutput(output_tag("result"))
        ]
    );
}

#[test]
fn tokenizes_supported_erb_control_tags() {
    let cases = [
        ("<% if user %>", ErbBlockKind::If, "if user"),
        (
            "<% unless user.guest? %>",
            ErbBlockKind::Unless,
            "unless user.guest?",
        ),
        ("<% case user.role %>", ErbBlockKind::Case, "case user.role"),
        ("<% do %>", ErbBlockKind::Do, "do"),
    ];

    for (input, kind, code) in cases {
        assert_eq!(
            tokenize(input).unwrap(),
            vec![Token::ErbBlockStart {
                kind,
                tag: code_tag(code),
                output: false
            }]
        );
    }
}

#[test]
fn tokenizes_erb_block_end_tag() {
    let tokens = tokenize("<% end %>").unwrap();

    assert_eq!(tokens, vec![Token::ErbBlockEnd(code_tag("end"))]);
}

#[test]
fn tokenizes_erb_branch_tags() {
    let cases = [
        ("<% else %>", ErbBranchKind::Else, "else"),
        ("<% elsif admin? %>", ErbBranchKind::Elsif, "elsif admin?"),
        (
            "<% when \"admin\" %>",
            ErbBranchKind::When,
            "when \"admin\"",
        ),
        (
            "<% rescue => error %>",
            ErbBranchKind::Rescue,
            "rescue => error",
        ),
        ("<% ensure %>", ErbBranchKind::Ensure, "ensure"),
    ];

    for (input, kind, code) in cases {
        assert_eq!(
            tokenize(input).unwrap(),
            vec![Token::ErbBranch {
                kind,
                tag: code_tag(code)
            }]
        );
    }
}

#[test]
fn tokenizes_begin_control_tag() {
    let tokens = tokenize("<% begin %>").unwrap();

    assert_eq!(
        tokens,
        vec![Token::ErbBlockStart {
            kind: ErbBlockKind::Begin,
            tag: code_tag("begin"),
            output: false
        }]
    );
}

#[test]
fn tokenizes_do_block_expression() {
    let tokens = tokenize("<% users.each do |user| %>").unwrap();

    assert_eq!(
        tokens,
        vec![Token::ErbBlockStart {
            kind: ErbBlockKind::Do,
            tag: code_tag("users.each do |user|"),
            output: false
        }]
    );
}

#[test]
fn tokenizes_erb_output_do_block_expression() {
    let tokens = tokenize("<%= form_with model: user do |form| %>").unwrap();

    assert_eq!(
        tokens,
        vec![Token::ErbBlockStart {
            kind: ErbBlockKind::Do,
            tag: output_tag("form_with model: user do |form|"),
            output: true
        }]
    );
}

#[test]
fn reports_unterminated_erb_tag() {
    let error = tokenize("<div><% if user").unwrap_err();

    assert_eq!(
        error.to_string(),
        "unterminated ERB tag at line 1, column 6"
    );
}

#[test]
fn tokenizes_supported_erb_marker_variants() {
    assert_eq!(
        tokenize("<%- if user -%>").unwrap(),
        vec![Token::ErbBlockStart {
            kind: ErbBlockKind::If,
            tag: erb_tag("if user", ErbTagOpen::TrimCode, ErbTagClose::Trim),
            output: false
        }]
    );
    assert_eq!(
        tokenize("<%== raw_html %>").unwrap(),
        vec![Token::ErbOutput(erb_tag(
            "raw_html",
            ErbTagOpen::RawOutput,
            ErbTagClose::Normal
        ))]
    );
    assert_eq!(
        tokenize("<% foo -%>").unwrap(),
        vec![Token::ErbCode(erb_tag(
            "foo",
            ErbTagOpen::Code,
            ErbTagClose::Trim
        ))]
    );
    assert_eq!(
        tokenize("<% -%>").unwrap(),
        vec![Token::ErbCode(erb_tag(
            "",
            ErbTagOpen::Code,
            ErbTagClose::Trim
        ))]
    );
}

#[test]
fn rejects_literal_erb_marker_without_rewriting_it() {
    let error = tokenize("<%%= literal %>").unwrap_err();

    assert_eq!(
        error.to_string(),
        "unsupported ERB marker `<%%` at line 1, column 1"
    );
}
