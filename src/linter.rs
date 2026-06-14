use crate::{
    lexer,
    mixed_parser::{self, Document, Node},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
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
    pub empty_erb_control_block: bool,
    pub unsupported_erb_block_starter: bool,
}

impl Default for LintRules {
    fn default() -> Self {
        Self {
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
            return vec![Diagnostic {
                message: error.to_string(),
            }];
        }
    };

    match mixed_parser::parse_spanned(&tokens) {
        Ok(document) => lint_document(&document, options),
        Err(error) => {
            vec![Diagnostic {
                message: error.to_string(),
            }]
        }
    }
}

fn lint_document(document: &Document, options: LintOptions) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    lint_nodes(&document.children, options, &mut diagnostics);
    diagnostics
}

fn lint_nodes(nodes: &[Node], options: LintOptions, diagnostics: &mut Vec<Diagnostic>) {
    for node in nodes {
        lint_node(node, options, diagnostics);
    }
}

fn lint_node(node: &Node, options: LintOptions, diagnostics: &mut Vec<Diagnostic>) {
    match node {
        Node::ErbCode(code) => lint_erb_code(code, options, diagnostics),
        Node::ErbBlock {
            code,
            output,
            children,
            branches,
            ..
        } => {
            if options.rules.empty_erb_control_block
                && !children.iter().any(is_meaningful_node)
                && !branches
                    .iter()
                    .any(|branch| branch.children.iter().any(is_meaningful_node))
            {
                diagnostics.push(Diagnostic {
                    message: format!(
                        "empty ERB control block `{}`",
                        format_erb_block_open(*output, code)
                    ),
                });
            }

            lint_nodes(children, options, diagnostics);
            for branch in branches {
                lint_nodes(&branch.children, options, diagnostics);
            }
        }
        Node::HtmlElement { children, .. } => lint_nodes(children, options, diagnostics),
        Node::HtmlText(_)
        | Node::HtmlSelfClosing { .. }
        | Node::HtmlVoid { .. }
        | Node::HtmlComment(_)
        | Node::HtmlDoctype(_)
        | Node::ErbOutput(_) => {}
    }
}

fn format_erb_block_open(output: bool, code: &str) -> String {
    if output {
        format!("<%= {} %>", code.trim())
    } else {
        format!("<% {} %>", code.trim())
    }
}

fn lint_erb_code(code: &str, options: LintOptions, diagnostics: &mut Vec<Diagnostic>) {
    if options.rules.unsupported_erb_block_starter
        && let Some("while" | "for" | "until") = first_keyword(code)
    {
        diagnostics.push(Diagnostic {
            message: format!("unsupported ERB block starter `<% {} %>`", code.trim()),
        });
    }
}

fn first_keyword(code: &str) -> Option<&str> {
    code.split_whitespace().next()
}

fn is_meaningful_node(node: &Node) -> bool {
    match node {
        Node::HtmlText(text) => !text.trim().is_empty(),
        Node::HtmlComment(_) => false,
        Node::HtmlElement { .. }
        | Node::HtmlSelfClosing { .. }
        | Node::HtmlVoid { .. }
        | Node::HtmlDoctype(_)
        | Node::ErbCode(_)
        | Node::ErbOutput(_)
        | Node::ErbBlock { .. } => true,
    }
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
            vec![Diagnostic {
                message: "unterminated ERB tag at line 1, column 6".to_string()
            }]
        );
    }

    #[test]
    fn reports_unexpected_block_end() {
        let diagnostics = lint("<% end %>");

        assert_eq!(
            diagnostics,
            vec![Diagnostic {
                message: "unexpected ERB block end `end` at line 1, column 1".to_string()
            }]
        );
    }

    #[test]
    fn reports_unclosed_block() {
        let diagnostics = lint("<% if user %>\n<p>Hello</p>\n");

        assert_eq!(
            diagnostics,
            vec![Diagnostic {
                message: "unclosed ERB block `if user` at line 1, column 1".to_string()
            }]
        );
    }

    #[test]
    fn reports_unbalanced_html_tags() {
        let diagnostics = lint("<div><span>Hello</div>");

        assert_eq!(
            diagnostics,
            vec![Diagnostic {
                message: "mismatched HTML close tag `</div>`, expected closing tag for `span`, found `div`".to_string()
            }]
        );
    }

    #[test]
    fn reports_empty_erb_control_blocks() {
        let diagnostics = lint("<% if show_empty_state %>\n<% end %>\n");

        assert_eq!(
            diagnostics,
            vec![Diagnostic {
                message: "empty ERB control block `<% if show_empty_state %>`".to_string()
            }]
        );
    }

    #[test]
    fn does_not_report_supported_erb_branches() {
        let diagnostics =
            lint("<% if current_user %>\n<% else %>\n<p>Please sign in</p>\n<% end %>");

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn reports_unsupported_erb_block_starters() {
        let diagnostics = lint("<% while job.running? %>\n<p>Waiting</p>\n");

        assert_eq!(
            diagnostics,
            vec![Diagnostic {
                message: "unsupported ERB block starter `<% while job.running? %>`".to_string()
            }]
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
}
