use super::*;

#[test]
fn tokenize_html() {
    let tokens = tokenize("<div>Hello</div>").unwrap();

    assert_eq!(tokens, vec![Token::Html("<div>Hello</div>".to_string())]);
}

#[test]
fn tokenizes_empty_erb_code_tag() {
    let tokens = tokenize("<% %>").unwrap();

    assert_eq!(tokens, vec![Token::ErbCode(String::new())]);
}

#[test]
fn tokenizes_erb_output_tag() {
    let tokens = tokenize("<%= user.name %>").unwrap();

    assert_eq!(tokens, vec![Token::ErbOutput("user.name".to_string())]);
}

#[test]
fn tokenizes_erb_comment_tag() {
    let tokens = tokenize("<%# erbfmt-ignore format: generated %>").unwrap();

    assert_eq!(
        tokens,
        vec![Token::ErbComment(
            "erbfmt-ignore format: generated".to_string()
        )]
    );
}

#[test]
fn tokenizes_html_fragments_around_erb() {
    let tokens = tokenize("<p>Hello <%= user.name %></p>").unwrap();

    assert_eq!(
        tokens,
        vec![
            Token::Html("<p>Hello ".to_string()),
            Token::ErbOutput("user.name".to_string()),
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
            Token::ErbOutput("user.name".to_string()),
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
            Token::ErbOutput("result".to_string())
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
                code: code.to_string(),
                output: false
            }]
        );
    }
}

#[test]
fn tokenizes_erb_block_end_tag() {
    let tokens = tokenize("<% end %>").unwrap();

    assert_eq!(tokens, vec![Token::ErbBlockEnd("end".to_string())]);
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
                code: code.to_string()
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
            code: "begin".to_string(),
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
            code: "users.each do |user|".to_string(),
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
            code: "form_with model: user do |form|".to_string(),
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
fn rejects_unsupported_erb_markers_without_rewriting_them() {
    let cases = [
        (
            "<%- if user %>",
            "unsupported ERB marker `<%-` at line 1, column 1",
        ),
        (
            "<%%= literal %>",
            "unsupported ERB marker `<%%` at line 1, column 1",
        ),
        (
            "<%== raw_html %>",
            "unsupported ERB marker `<%==` at line 1, column 1",
        ),
        (
            "<%= user -%>",
            "unsupported ERB marker `-%>` at line 1, column 10",
        ),
    ];

    for (input, expected) in cases {
        assert_eq!(tokenize(input).unwrap_err().to_string(), expected);
    }
}
