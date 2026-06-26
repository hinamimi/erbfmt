use crate::lexer::SourceLocation;

use super::{Diagnostic, LintOptions};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ErbBlockLintFrame {
    pub(super) code: String,
    pub(super) output: bool,
    pub(super) location: SourceLocation,
    pub(super) has_meaningful_content: bool,
    pub(super) active_branch: Option<ErbBranchLintFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ErbBranchLintFrame {
    pub(super) code: String,
    pub(super) location: SourceLocation,
    pub(super) has_meaningful_content: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ErbCodeTagKind {
    Code,
    Output,
}

pub(super) fn mark_current_block_meaningful(stack: &mut [ErbBlockLintFrame]) {
    if let Some(frame) = stack.last_mut() {
        frame.has_meaningful_content = true;

        if let Some(branch) = &mut frame.active_branch {
            branch.has_meaningful_content = true;
        }
    }
}

pub(super) fn finish_active_branch(
    frame: &mut ErbBlockLintFrame,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(branch) = frame.active_branch.take() else {
        return;
    };

    if options.rules.empty_erb_branch && !branch.has_meaningful_content {
        diagnostics.push(Diagnostic::located_with_severity(
            format!("empty ERB branch `<% {} %>`", branch.code.trim()),
            branch.location,
            options.rule_severities.empty_erb_branch,
        ));
    }
}

pub(super) fn lint_empty_erb_control_block(
    frame: &ErbBlockLintFrame,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if options.rules.empty_erb_control_block && !frame.has_meaningful_content {
        diagnostics.push(Diagnostic::located_with_severity(
            format!(
                "empty ERB control block `{}`",
                format_erb_block_open(frame.output, &frame.code)
            ),
            frame.location,
            options.rule_severities.empty_erb_control_block,
        ));
    }
}

fn format_erb_block_open(output: bool, code: &str) -> String {
    if output {
        format!("<%= {} %>", code.trim())
    } else {
        format!("<% {} %>", code.trim())
    }
}

pub(super) fn lint_empty_erb_code_tag(
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

    diagnostics.push(Diagnostic::located_with_severity(
        message,
        location,
        options.rule_severities.empty_erb_code_tag,
    ));
}

pub(super) fn lint_erb_code(
    code: &str,
    location: SourceLocation,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if options.rules.unsupported_erb_block_starter
        && let Some(keyword @ ("while" | "for" | "until")) = first_keyword(code)
    {
        diagnostics.push(Diagnostic::located_with_severity(
            format!("unsupported ERB block starter `{keyword}`"),
            location,
            options.rule_severities.unsupported_erb_block_starter,
        ));
    }
}

fn first_keyword(code: &str) -> Option<&str> {
    code.split_whitespace().next()
}
