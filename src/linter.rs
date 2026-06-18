use crate::{
    html::{self, HtmlToken},
    lexer,
    lexer::SourceLocation,
    mixed_parser,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub location: Option<SourceLocation>,
}

impl Diagnostic {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            location: None,
        }
    }

    fn located(message: impl Into<String>, location: SourceLocation) -> Self {
        Self {
            message: message.into(),
            location: Some(location),
        }
    }

    pub fn message_with_location(&self) -> String {
        match self.location {
            Some(location) => format!("{} at {}", self.message, location),
            None => self.message.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LintOptions {
    pub enabled: bool,
    pub rules: LintRules,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            rules: LintRules::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LintRules {
    pub empty_erb_branch: bool,
    pub empty_erb_code_tag: bool,
    pub empty_erb_control_block: bool,
    pub unsupported_erb_block_starter: bool,
}

impl Default for LintRules {
    fn default() -> Self {
        Self {
            empty_erb_branch: true,
            empty_erb_code_tag: true,
            empty_erb_control_block: true,
            unsupported_erb_block_starter: true,
        }
    }
}

#[allow(dead_code)]
pub fn lint(input: &str) -> Vec<Diagnostic> {
    lint_with_options(input, LintOptions::default())
}

pub fn lint_with_options(input: &str, options: LintOptions) -> Vec<Diagnostic> {
    if !options.enabled {
        return Vec::new();
    }

    let tokens = match lexer::tokenize_with_spans(input) {
        Ok(tokens) => tokens,
        Err(error) => {
            return vec![Diagnostic::new(error.to_string())];
        }
    };

    match mixed_parser::parse_spanned(&tokens) {
        Ok(_) => lint_tokens(&tokens, options),
        Err(error) => vec![Diagnostic::new(error.to_string())],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ErbBlockLintFrame {
    code: String,
    output: bool,
    location: SourceLocation,
    has_meaningful_content: bool,
    active_branch: Option<ErbBranchLintFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ErbBranchLintFrame {
    code: String,
    location: SourceLocation,
    has_meaningful_content: bool,
}

fn lint_tokens(tokens: &[lexer::SpannedToken], options: LintOptions) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut stack: Vec<ErbBlockLintFrame> = Vec::new();

    for spanned in tokens {
        match &spanned.token {
            lexer::Token::Html(fragment) => {
                if html_fragment_has_meaningful_content(fragment) {
                    mark_current_block_meaningful(&mut stack);
                }
            }
            lexer::Token::ErbCode(code) => {
                lint_empty_erb_code_tag(
                    ErbCodeTagKind::Code,
                    code,
                    spanned.span.location,
                    options,
                    &mut diagnostics,
                );
                lint_erb_code(code, spanned.span.location, options, &mut diagnostics);
                if !code.trim().is_empty() {
                    mark_current_block_meaningful(&mut stack);
                }
            }
            lexer::Token::ErbOutput(code) => {
                lint_empty_erb_code_tag(
                    ErbCodeTagKind::Output,
                    code,
                    spanned.span.location,
                    options,
                    &mut diagnostics,
                );
                if !code.trim().is_empty() {
                    mark_current_block_meaningful(&mut stack);
                }
            }
            lexer::Token::ErbBlockStart { code, output, .. } => {
                mark_current_block_meaningful(&mut stack);
                stack.push(ErbBlockLintFrame {
                    code: code.clone(),
                    output: *output,
                    location: spanned.span.location,
                    has_meaningful_content: false,
                    active_branch: None,
                });
            }
            lexer::Token::ErbBranch { code, .. } => {
                if let Some(frame) = stack.last_mut() {
                    finish_active_branch(frame, options, &mut diagnostics);
                    frame.active_branch = Some(ErbBranchLintFrame {
                        code: code.clone(),
                        location: spanned.span.location,
                        has_meaningful_content: false,
                    });
                }
            }
            lexer::Token::ErbBlockEnd(_) => {
                let Some(mut frame) = stack.pop() else {
                    continue;
                };

                finish_active_branch(&mut frame, options, &mut diagnostics);

                if options.rules.empty_erb_control_block && !frame.has_meaningful_content {
                    diagnostics.push(Diagnostic::located(
                        format!(
                            "empty ERB control block `{}`",
                            format_erb_block_open(frame.output, &frame.code)
                        ),
                        frame.location,
                    ));
                }
            }
        }
    }

    diagnostics
}

fn mark_current_block_meaningful(stack: &mut [ErbBlockLintFrame]) {
    if let Some(frame) = stack.last_mut() {
        frame.has_meaningful_content = true;

        if let Some(branch) = &mut frame.active_branch {
            branch.has_meaningful_content = true;
        }
    }
}

fn finish_active_branch(
    frame: &mut ErbBlockLintFrame,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(branch) = frame.active_branch.take() else {
        return;
    };

    if options.rules.empty_erb_branch && !branch.has_meaningful_content {
        diagnostics.push(Diagnostic::located(
            format!("empty ERB branch `<% {} %>`", branch.code.trim()),
            branch.location,
        ));
    }
}

fn html_fragment_has_meaningful_content(fragment: &str) -> bool {
    html::tokenize(fragment)
        .into_iter()
        .any(|token| match token {
            HtmlToken::Text(text) => !text.trim().is_empty(),
            HtmlToken::Comment(_) => false,
            HtmlToken::OpenTag(_)
            | HtmlToken::CloseTag(_)
            | HtmlToken::SelfClosingTag(_)
            | HtmlToken::VoidTag(_)
            | HtmlToken::Doctype(_) => true,
        })
}

fn format_erb_block_open(output: bool, code: &str) -> String {
    if output {
        format!("<%= {} %>", code.trim())
    } else {
        format!("<% {} %>", code.trim())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErbCodeTagKind {
    Code,
    Output,
}

fn lint_empty_erb_code_tag(
    kind: ErbCodeTagKind,
    code: &str,
    location: SourceLocation,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !options.rules.empty_erb_code_tag || !code.trim().is_empty() {
        return;
    }

    let message = match kind {
        ErbCodeTagKind::Code => "empty ERB code tag `<% %>`",
        ErbCodeTagKind::Output => "empty ERB output tag `<%= %>`",
    };

    diagnostics.push(Diagnostic::located(message, location));
}

fn lint_erb_code(
    code: &str,
    location: SourceLocation,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if options.rules.unsupported_erb_block_starter
        && let Some("while" | "for" | "until") = first_keyword(code)
    {
        diagnostics.push(Diagnostic::located(
            format!("unsupported ERB block starter `<% {} %>`", code.trim()),
            location,
        ));
    }
}

fn first_keyword(code: &str) -> Option<&str> {
    code.split_whitespace().next()
}

#[cfg(test)]
mod tests {
    use super::*;

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
                "mismatched HTML close tag `</div>`, expected closing tag for `span`, found `div`"
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
        let diagnostics =
            lint("<% if current_user %>\n<% else %>\n<p>Please sign in</p>\n<% end %>");

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
        let diagnostics =
            lint("<% if current_user %>\n<p>Hello</p>\n<% else %>\n  <% %>\n<% end %>\n");

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
                "unsupported ERB block starter `<% while job.running? %>`",
                SourceLocation { line: 1, column: 1 }
            )]
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
}
