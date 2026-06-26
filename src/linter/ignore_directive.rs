use crate::ignore::{IgnoreSelector, parse_ignore_directive};

use super::Diagnostic;

#[derive(Debug, Clone, PartialEq, Eq)]
struct LintIgnoreDirective {
    target_line: usize,
    rule: Option<String>,
}

pub(super) fn apply_lint_ignore_directives(
    input: &str,
    diagnostics: Vec<Diagnostic>,
) -> Vec<Diagnostic> {
    let directives = lint_ignore_directives(input);

    if directives.is_empty() {
        return diagnostics;
    }

    diagnostics
        .into_iter()
        .filter(|diagnostic| !is_lint_diagnostic_ignored(diagnostic, &directives))
        .collect()
}

fn is_lint_diagnostic_ignored(diagnostic: &Diagnostic, directives: &[LintIgnoreDirective]) -> bool {
    let Some(location) = diagnostic.location else {
        return false;
    };

    directives.iter().any(|directive| {
        directive.target_line == location.line
            && directive.rule.as_deref().is_none_or(|rule| {
                diagnostic_rule_id(&diagnostic.message).is_some_and(|id| id == rule)
            })
    })
}

fn lint_ignore_directives(input: &str) -> Vec<LintIgnoreDirective> {
    input
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let directive = parse_ignore_directive(line)?;
            let rule = match directive.selector {
                IgnoreSelector::Lint { rule } => rule,
                IgnoreSelector::All => None,
                IgnoreSelector::Format => return None,
            };

            Some(LintIgnoreDirective {
                target_line: index + 2,
                rule,
            })
        })
        .collect()
}

fn diagnostic_rule_id(message: &str) -> Option<&'static str> {
    if message.starts_with("empty ERB branch") {
        Some("emptyErbBranch")
    } else if message.starts_with("empty ERB code tag")
        || message.starts_with("empty ERB output tag")
    {
        Some("emptyErbCodeTag")
    } else if message.starts_with("empty ERB control block") {
        Some("emptyErbControlBlock")
    } else if message.starts_with("deprecated HTML tag") {
        Some("noDeprecatedHtmlTag")
    } else if message.starts_with("duplicate HTML attribute") {
        Some("noDuplicateHtmlAttribute")
    } else if message.starts_with("invalid HTML boolean attribute value")
        || message.starts_with("redundant HTML boolean attribute value")
    {
        Some("noInvalidHtmlBooleanAttribute")
    } else if message.starts_with("invalid HTML nesting") {
        Some("noInvalidHtmlNesting")
    } else if message.starts_with("self-closing HTML tag") {
        Some("noSelfClosingHtmlTag")
    } else if message.starts_with("unsupported ERB block starter") {
        Some("unsupportedErbBlockStarter")
    } else {
        None
    }
}
