use crate::{html, lexer, mixed_parser};

mod diagnostic;
mod erb;
mod html_rules;
mod ignore_directive;
mod options;

pub use diagnostic::{Diagnostic, DiagnosticSeverity};
use erb::{
    ErbBlockLintFrame, ErbBranchLintFrame, ErbCodeTagKind, finish_active_branch,
    lint_empty_erb_code_tag, lint_empty_erb_control_block, lint_erb_code,
    mark_current_block_meaningful,
};
use html_rules::{HtmlElementLintFrame, html_tokens_have_meaningful_content, lint_html_tokens};
use ignore_directive::apply_lint_ignore_directives;
pub use options::{LintOptions, LintRuleSeverities, LintRules};

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

    match mixed_parser::parse_spanned_with_options(&tokens, options.parser) {
        Ok(_) => apply_lint_ignore_directives(input, lint_tokens(input, &tokens, options)),
        Err(error) => vec![Diagnostic::new(error.to_string())],
    }
}

fn lint_tokens(
    input: &str,
    tokens: &[lexer::SpannedToken],
    options: LintOptions,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut erb_stack: Vec<ErbBlockLintFrame> = Vec::new();
    let mut html_stack: Vec<HtmlElementLintFrame> = Vec::new();

    for spanned in tokens {
        match &spanned.token {
            lexer::Token::Html(fragment) => {
                let html_tokens = html::tokenize_with_spans(fragment);
                lint_html_tokens(
                    input,
                    spanned.span.start,
                    &html_tokens,
                    &mut html_stack,
                    options,
                    &mut diagnostics,
                );

                if html_tokens_have_meaningful_content(&html_tokens) {
                    mark_current_block_meaningful(&mut erb_stack);
                }
            }
            lexer::Token::ErbCode(tag) => {
                lint_empty_erb_code_tag(
                    ErbCodeTagKind::Code,
                    &tag.code,
                    spanned.span.location,
                    options,
                    &mut diagnostics,
                );
                lint_erb_code(&tag.code, spanned.span.location, options, &mut diagnostics);
                if !tag.code.trim().is_empty() {
                    mark_current_block_meaningful(&mut erb_stack);
                }
            }
            lexer::Token::ErbComment(_) => {}
            lexer::Token::ErbOutput(tag) => {
                lint_empty_erb_code_tag(
                    ErbCodeTagKind::Output,
                    &tag.code,
                    spanned.span.location,
                    options,
                    &mut diagnostics,
                );
                if !tag.code.trim().is_empty() {
                    mark_current_block_meaningful(&mut erb_stack);
                }
            }
            lexer::Token::ErbBlockStart { tag, output, .. } => {
                mark_current_block_meaningful(&mut erb_stack);
                erb_stack.push(ErbBlockLintFrame {
                    code: tag.code.clone(),
                    output: *output,
                    location: spanned.span.location,
                    has_meaningful_content: false,
                    active_branch: None,
                });
            }
            lexer::Token::ErbBranch { tag, .. } => {
                if let Some(frame) = erb_stack.last_mut() {
                    finish_active_branch(frame, options, &mut diagnostics);
                    frame.active_branch = Some(ErbBranchLintFrame {
                        code: tag.code.clone(),
                        location: spanned.span.location,
                        has_meaningful_content: false,
                    });
                }
            }
            lexer::Token::ErbBlockEnd(_) => {
                let Some(mut frame) = erb_stack.pop() else {
                    continue;
                };

                finish_active_branch(&mut frame, options, &mut diagnostics);
                lint_empty_erb_control_block(&frame, options, &mut diagnostics);
            }
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests;
