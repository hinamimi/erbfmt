use super::*;
use crate::syntax::erb::SourceLocation;

#[test]
fn reports_no_diagnostics_for_valid_template() {
    let diagnostics = lint("<% if user %>\n<p>Hello</p>\n<% end %>\n");

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn reports_unterminated_erb_tag() {
    let diagnostics = lint("<div><% if user");

    assert_eq!(
        diagnostics,
        vec![Diagnostic::new("unterminated ERB tag at line 1, column 6")]
    );
}

#[test]
fn reports_unexpected_block_end() {
    let diagnostics = lint("<% end %>");

    assert_eq!(
        diagnostics,
        vec![Diagnostic::new(
            "unexpected ERB block end `end` at line 1, column 1"
        )]
    );
}

#[test]
fn reports_unclosed_block() {
    let diagnostics = lint("<% if user %>\n<p>Hello</p>\n");

    assert_eq!(
        diagnostics,
        vec![Diagnostic::new(
            "unclosed ERB block `if user` at line 1, column 1"
        )]
    );
}

#[test]
fn reports_unbalanced_html_tags() {
    let diagnostics = lint("<div><span>Hello</div>");

    assert_eq!(
        diagnostics,
        vec![Diagnostic::new(
            "mismatched HTML close tag `</div>`, expected `</span>` at line 1, column 17"
        )]
    );
}

#[test]
fn reports_empty_erb_control_blocks() {
    let diagnostics = lint("<% if show_empty_state %>\n<% end %>\n");

    assert_eq!(
        diagnostics,
        vec![Diagnostic::located(
            "empty ERB control block `<% if show_empty_state %>`",
            SourceLocation { line: 1, column: 1 }
        )]
    );
}

#[test]
fn reports_empty_erb_code_tags() {
    let diagnostics = lint("<p>Before</p>\n  <% %>\n  <%=   %>\n");

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::located(
                "empty ERB code tag `<% %>`",
                SourceLocation { line: 2, column: 3 }
            ),
            Diagnostic::located(
                "empty ERB output tag `<%= %>`",
                SourceLocation { line: 3, column: 3 }
            )
        ]
    );
}

#[test]
fn reports_self_closing_html_tags() {
    let diagnostics = lint("<section>\n  <div />\n  <br />\n</section>\n");

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::located(
                "self-closing HTML tag `<div />` is not valid HTML5",
                SourceLocation { line: 2, column: 3 }
            ),
            Diagnostic::located(
                "self-closing HTML tag `<br />` is not valid HTML5",
                SourceLocation { line: 3, column: 3 }
            )
        ]
    );
}

#[test]
fn allows_self_closing_tags_in_foreign_content() {
    let diagnostics = lint(
        "<svg><path d=\"M0 0\" /><g><use href=\"#icon\" /></g></svg>\n<math><mi /></math>\n<div />\n",
    );

    assert_eq!(
        diagnostics,
        vec![Diagnostic::located(
            "self-closing HTML tag `<div />` is not valid HTML5",
            SourceLocation { line: 3, column: 1 }
        )]
    );
}

#[test]
fn reports_deprecated_html_tags() {
    let diagnostics =
        lint("<main>\n  <center>Legacy</center>\n  <font color=\"red\">Alert</font>\n</main>\n");

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::located(
                "deprecated HTML tag `<center>`",
                SourceLocation { line: 2, column: 3 }
            ),
            Diagnostic::located(
                "deprecated HTML tag `<font color=\"red\">`",
                SourceLocation { line: 3, column: 3 }
            )
        ]
    );
}

#[test]
fn reports_rule_warning_severity() {
    let diagnostics = lint_with_options(
        "<center>Legacy</center>\n",
        LintOptions {
            rule_severities: LintRuleSeverities {
                no_deprecated_html_tag: DiagnosticSeverity::Warning,
                ..LintRuleSeverities::default()
            },
            ..LintOptions::default()
        },
    );

    assert_eq!(
        diagnostics,
        vec![Diagnostic::located_with_severity(
            "deprecated HTML tag `<center>`",
            SourceLocation { line: 1, column: 1 },
            DiagnosticSeverity::Warning
        )]
    );
}

#[test]
fn reports_duplicate_html_attributes() {
    let diagnostics = lint(
        "<main>\n  <article class=\"card\" id=\"one\" class=\"wide\" data-user-id=\"1\" DATA-USER-ID=\"2\"></article>\n</main>\n",
    );

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::located(
                "duplicate HTML attribute `class`",
                SourceLocation {
                    line: 2,
                    column: 34
                }
            ),
            Diagnostic::located(
                "duplicate HTML attribute `data-user-id`",
                SourceLocation {
                    line: 2,
                    column: 64
                }
            )
        ]
    );
}

#[test]
fn does_not_report_duplicate_html_attributes_when_tag_contains_erb() {
    let diagnostics = lint(r#"<div class="card" <%= tag_options %> class="wide"></div>"#);

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn reports_invalid_html_boolean_attribute_values() {
    let diagnostics = lint(r#"<input disabled="false" checked="checked" hidden>"#);

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::located(
                "invalid HTML boolean attribute value `disabled=\"false\"`",
                SourceLocation { line: 1, column: 8 }
            ),
            Diagnostic::located(
                "redundant HTML boolean attribute value `checked=\"checked\"`",
                SourceLocation {
                    line: 1,
                    column: 25
                }
            )
        ]
    );
}

#[test]
fn does_not_report_html_boolean_attribute_values_when_tag_contains_erb() {
    let diagnostics = lint(r#"<input disabled="<%= disabled? %>" checked="checked">"#);

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn reports_non_double_quoted_html_attribute_values() {
    let diagnostics = lint("<div class=<%= foo %> data-name=foo title='Profile' hidden></div>");

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::located(
                "HTML attribute value must use double quotes `class=<%= foo %>`",
                SourceLocation { line: 1, column: 6 }
            ),
            Diagnostic::located(
                "HTML attribute value must use double quotes `data-name=foo`",
                SourceLocation {
                    line: 1,
                    column: 23
                }
            ),
            Diagnostic::located(
                "HTML attribute value must use double quotes `title='Profile'`",
                SourceLocation {
                    line: 1,
                    column: 37
                }
            )
        ]
    );
}

#[test]
fn reports_unknown_html_attributes_on_initial_supported_tags() {
    let diagnostics = lint(
        r#"<input name="email" auto-complete="email">
<a href="/home" targt="_blank">Home</a>
<button tab-index="0" onclick="save()">Save</button>"#,
    );

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::located_with_severity(
                "unknown HTML attribute `auto-complete`; did you mean `autocomplete`?",
                SourceLocation {
                    line: 1,
                    column: 21
                },
                DiagnosticSeverity::Warning
            ),
            Diagnostic::located_with_severity(
                "unknown HTML attribute `targt`",
                SourceLocation {
                    line: 2,
                    column: 17
                },
                DiagnosticSeverity::Warning
            ),
            Diagnostic::located_with_severity(
                "unknown HTML attribute `tab-index`; did you mean `tabindex`?",
                SourceLocation { line: 3, column: 9 },
                DiagnosticSeverity::Warning
            )
        ]
    );
}

#[test]
fn allows_global_framework_and_custom_attributes() {
    let diagnostics = lint(
        r#"<input id="email" class="field" data-controller="form" aria-label="Email" v-model="email" x-ref="email" hx-post="/users" phx-click="save">
<my-input tab-index="0" custom-prop="value"></my-input>"#,
    );

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn allows_unknown_html_attribute_rule_on_expanded_supported_tags() {
    let diagnostics = lint(
        r#"<form accept-charset="UTF-8" action="/users" autocomplete="off" method="post"></form>
<label for="email">Email</label>
<img src="/avatar.png" srcset="/avatar@2x.png 2x" alt="Avatar" loading="lazy" referrerpolicy="no-referrer">
<script type="module" src="/app.js" crossorigin="anonymous" defer></script>
<link rel="preload" href="/app.css" as="style" imagesizes="100vw">
<meta http-equiv="refresh" content="0">
<textarea name="bio" maxlength="200" readonly></textarea>
<select name="role" multiple required><option value="admin" selected>Admin</option></select>"#,
    );

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn reports_camel_case_attribute_typos() {
    let diagnostics = lint(r#"<button tabIndex="0">Open</button><input maxLength="20">"#);

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::located_with_severity(
                "unknown HTML attribute `tabIndex`; did you mean `tabindex`?",
                SourceLocation { line: 1, column: 9 },
                DiagnosticSeverity::Warning
            ),
            Diagnostic::located_with_severity(
                "unknown HTML attribute `maxLength`; did you mean `maxlength`?",
                SourceLocation {
                    line: 1,
                    column: 42
                },
                DiagnosticSeverity::Warning
            )
        ]
    );
}

#[test]
fn reports_typo_candidates_on_expanded_supported_tags() {
    let diagnostics = lint(
        r#"<form acceptCharset="UTF-8"></form>
<img srcSet="/avatar@2x.png 2x" referrerPolicy="no-referrer">
<script noModule src="/legacy.js"></script>
<meta httpEquiv="refresh" content="0">
<textarea readOnly maxLength="200"></textarea>"#,
    );

    assert_eq!(
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>(),
        vec![
            "unknown HTML attribute `acceptCharset`; did you mean `accept-charset`?",
            "unknown HTML attribute `srcSet`; did you mean `srcset`?",
            "unknown HTML attribute `referrerPolicy`; did you mean `referrerpolicy`?",
            "unknown HTML attribute `noModule`; did you mean `nomodule`?",
            "unknown HTML attribute `httpEquiv`; did you mean `http-equiv`?",
            "unknown HTML attribute `readOnly`; did you mean `readonly`?",
            "unknown HTML attribute `maxLength`; did you mean `maxlength`?"
        ]
    );
}

#[test]
fn skips_unknown_html_attributes_when_tag_contains_erb() {
    let diagnostics = lint(
        r#"<input <%= field_options %> auto-complete="email"><a <%= link_options %> targt="_blank">Home</a>"#,
    );

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn respects_disabled_unknown_html_attribute_rule() {
    let diagnostics = lint_with_options(
        r#"<input auto-complete="email">"#,
        LintOptions {
            rules: LintRules {
                no_unknown_html_attribute: false,
                ..LintRules::default()
            },
            ..LintOptions::default()
        },
    );

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn reports_invalid_list_children() {
    let diagnostics = lint(
        "<ul>\n  <div>Bad</div>\n  <% items.each do |item| %>\n    <li><%= item.name %></li>\n  <% end %>\n</ul>\n<ol>\n  Text\n</ol>\n",
    );

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::located(
                "invalid HTML nesting: <ul> cannot have <div> as a direct child",
                SourceLocation { line: 2, column: 3 }
            ),
            Diagnostic::located(
                "invalid HTML nesting: <ol> cannot have text as a direct child",
                SourceLocation { line: 8, column: 3 }
            )
        ]
    );
}

#[test]
fn reports_invalid_table_structure() {
    let diagnostics = lint(
        "<table>\n  <div>Bad</div>\n  <thead><td>Bad</td></thead>\n  <tr><div>Bad</div></tr>\n</table>\n",
    );

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::located(
                "invalid HTML nesting: <table> cannot have <div> as a direct child",
                SourceLocation { line: 2, column: 3 }
            ),
            Diagnostic::located(
                "invalid HTML nesting: <thead> cannot have <td> as a direct child",
                SourceLocation {
                    line: 3,
                    column: 10
                }
            ),
            Diagnostic::located(
                "invalid HTML nesting: <tr> cannot have <div> as a direct child",
                SourceLocation { line: 4, column: 7 }
            )
        ]
    );
}

#[test]
fn reports_block_html_inside_paragraphs() {
    let diagnostics = lint("<p>\n  <span>OK</span>\n  <div>Bad</div>\n</p>\n");

    assert_eq!(
        diagnostics,
        vec![Diagnostic::located(
            "invalid HTML nesting: <p> cannot contain <div>",
            SourceLocation { line: 3, column: 3 }
        )]
    );
}

#[test]
fn does_not_report_valid_list_and_table_structure() {
    let diagnostics = lint(
        "<ul>\n  <% items.each do |item| %>\n    <li><%= item.name %></li>\n  <% end %>\n</ul>\n<table>\n  <thead><tr><th>Name</th></tr></thead>\n  <tbody><tr><td>A</td></tr></tbody>\n</table>\n<p><span>OK</span><a href=\"#\">Link</a></p>\n",
    );

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn reports_html_rule_locations_after_erb_tags() {
    let diagnostics = lint("<% if user %>\n  <center>Legacy</center>\n<% end %>\n");

    assert_eq!(
        diagnostics,
        vec![Diagnostic::located(
            "deprecated HTML tag `<center>`",
            SourceLocation { line: 2, column: 3 }
        )]
    );
}

#[test]
fn ignores_lint_diagnostics_on_the_next_line() {
    let diagnostics = lint("<!-- erbfmt-ignore lint: legacy markup -->\n<center>Legacy</center>\n");

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn ignores_only_the_selected_lint_rule() {
    let diagnostics = lint(
        "<!-- erbfmt-ignore lint/noDeprecatedHtmlTag: legacy markup -->\n<center><div /></center>\n",
    );

    assert_eq!(
        diagnostics,
        vec![Diagnostic::located(
            "self-closing HTML tag `<div />` is not valid HTML5",
            SourceLocation { line: 2, column: 9 }
        )]
    );
}

#[test]
fn ignores_lint_diagnostics_from_erb_comments() {
    let diagnostics =
        lint("<%# erbfmt-ignore lint/emptyErbCodeTag: generated placeholder %>\n<% %>\n");

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn ignores_lint_diagnostics_with_combined_directives() {
    let diagnostics =
        lint("<!-- erbfmt-ignore all: generated markup -->\n<center>Legacy</center>\n");

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn empty_erb_code_tags_do_not_count_as_meaningful_block_content() {
    let diagnostics = lint("<% if show_empty_state %>\n  <% %>\n<% end %>\n");

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::located(
                "empty ERB code tag `<% %>`",
                SourceLocation { line: 2, column: 3 }
            ),
            Diagnostic::located(
                "empty ERB control block `<% if show_empty_state %>`",
                SourceLocation { line: 1, column: 1 }
            )
        ]
    );
}

#[test]
fn does_not_report_supported_erb_branches() {
    let diagnostics = lint("<% if current_user %>\n<% else %>\n<p>Please sign in</p>\n<% end %>");

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn reports_empty_erb_branches() {
    let diagnostics = lint(
        "<% if current_user %>\n<p>Hello</p>\n<% else %>\n<% end %>\n\
             <% case role %>\n<% when \"admin\" %>\n<% when \"member\" %>\n<p>Member</p>\n<% end %>\n",
    );

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::located(
                "empty ERB branch `<% else %>`",
                SourceLocation { line: 3, column: 1 }
            ),
            Diagnostic::located(
                "empty ERB branch `<% when \"admin\" %>`",
                SourceLocation { line: 6, column: 1 }
            )
        ]
    );
}

#[test]
fn empty_erb_code_tags_do_not_count_as_meaningful_branch_content() {
    let diagnostics = lint("<% if current_user %>\n<p>Hello</p>\n<% else %>\n  <% %>\n<% end %>\n");

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::located(
                "empty ERB code tag `<% %>`",
                SourceLocation { line: 4, column: 3 }
            ),
            Diagnostic::located(
                "empty ERB branch `<% else %>`",
                SourceLocation { line: 3, column: 1 }
            )
        ]
    );
}

#[test]
fn does_not_report_non_empty_erb_branches() {
    let diagnostics = lint(
        "<% if current_user %>\n<p>Hello</p>\n<% elsif guest? %>\n<p>Guest</p>\n<% else %>\n<p>Please sign in</p>\n<% end %>\n\
             <% begin %>\n<% rescue StandardError %>\n<p>Failed</p>\n<% ensure %>\n<% cleanup %>\n<% end %>\n",
    );

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn reports_unsupported_erb_block_starters() {
    let diagnostics = lint("<% while job.running? %>\n<p>Waiting</p>\n");

    assert_eq!(
        diagnostics,
        vec![Diagnostic::located(
            "unsupported ERB block starter `while`",
            SourceLocation { line: 1, column: 1 }
        )]
    );
}

#[test]
fn reports_unsupported_erb_block_starter_keywords() {
    let diagnostics = lint(
        "<% for user in users %>\n<p><%= user.name %></p>\n<% until done? %>\n<p>Waiting</p>\n",
    );

    assert_eq!(
        diagnostics,
        vec![
            Diagnostic::located(
                "unsupported ERB block starter `for`",
                SourceLocation { line: 1, column: 1 }
            ),
            Diagnostic::located(
                "unsupported ERB block starter `until`",
                SourceLocation { line: 3, column: 1 }
            )
        ]
    );
}

#[test]
fn respects_disabled_linter() {
    let diagnostics = lint_with_options(
        "<% if show_empty_state %>\n<% end %>\n",
        LintOptions {
            enabled: false,
            ..LintOptions::default()
        },
    );

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn respects_disabled_empty_block_rule() {
    let diagnostics = lint_with_options(
        "<% if show_empty_state %>\n<% end %>\n",
        LintOptions {
            rules: LintRules {
                empty_erb_control_block: false,
                ..LintRules::default()
            },
            ..LintOptions::default()
        },
    );

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn respects_disabled_empty_erb_branch_rule() {
    let diagnostics = lint_with_options(
        "<% if current_user %>\n<p>Hello</p>\n<% else %>\n<% end %>\n",
        LintOptions {
            rules: LintRules {
                empty_erb_branch: false,
                ..LintRules::default()
            },
            ..LintOptions::default()
        },
    );

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn respects_disabled_html_rules() {
    let diagnostics = lint_with_options(
        "<center><div /></center>\n",
        LintOptions {
            rules: LintRules {
                no_deprecated_html_tag: false,
                no_self_closing_html_tag: false,
                ..LintRules::default()
            },
            ..LintOptions::default()
        },
    );

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn respects_disabled_duplicate_html_attribute_rule() {
    let diagnostics = lint_with_options(
        r#"<div class="card" class="wide"></div>"#,
        LintOptions {
            rules: LintRules {
                no_duplicate_html_attribute: false,
                ..LintRules::default()
            },
            ..LintOptions::default()
        },
    );

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn respects_disabled_invalid_html_boolean_attribute_rule() {
    let diagnostics = lint_with_options(
        r#"<input disabled="false" checked="checked">"#,
        LintOptions {
            rules: LintRules {
                no_invalid_html_boolean_attribute: false,
                ..LintRules::default()
            },
            ..LintOptions::default()
        },
    );

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn respects_disabled_invalid_html_nesting_rule() {
    let diagnostics = lint_with_options(
        "<ul><div>Bad</div></ul>\n<p><div>Bad</div></p>\n",
        LintOptions {
            rules: LintRules {
                no_invalid_html_nesting: false,
                ..LintRules::default()
            },
            ..LintOptions::default()
        },
    );

    assert_eq!(diagnostics, Vec::new());
}

#[test]
fn respects_disabled_empty_erb_code_tag_rule() {
    let diagnostics = lint_with_options(
        "<% %>\n<%= %>\n",
        LintOptions {
            rules: LintRules {
                empty_erb_code_tag: false,
                ..LintRules::default()
            },
            ..LintOptions::default()
        },
    );

    assert_eq!(diagnostics, Vec::new());
}
