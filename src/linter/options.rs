use super::DiagnosticSeverity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LintOptions {
    pub enabled: bool,
    pub rules: LintRules,
    pub rule_severities: LintRuleSeverities,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            rules: LintRules::default(),
            rule_severities: LintRuleSeverities::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LintRules {
    pub empty_erb_branch: bool,
    pub empty_erb_code_tag: bool,
    pub empty_erb_control_block: bool,
    pub no_deprecated_html_tag: bool,
    pub no_duplicate_html_attribute: bool,
    pub no_invalid_html_boolean_attribute: bool,
    pub no_invalid_html_nesting: bool,
    pub no_non_double_quoted_html_attribute_value: bool,
    pub no_self_closing_html_tag: bool,
    pub unsupported_erb_block_starter: bool,
}

impl Default for LintRules {
    fn default() -> Self {
        Self {
            empty_erb_branch: true,
            empty_erb_code_tag: true,
            empty_erb_control_block: true,
            no_deprecated_html_tag: true,
            no_duplicate_html_attribute: true,
            no_invalid_html_boolean_attribute: true,
            no_invalid_html_nesting: true,
            no_non_double_quoted_html_attribute_value: true,
            no_self_closing_html_tag: true,
            unsupported_erb_block_starter: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LintRuleSeverities {
    pub empty_erb_branch: DiagnosticSeverity,
    pub empty_erb_code_tag: DiagnosticSeverity,
    pub empty_erb_control_block: DiagnosticSeverity,
    pub no_deprecated_html_tag: DiagnosticSeverity,
    pub no_duplicate_html_attribute: DiagnosticSeverity,
    pub no_invalid_html_boolean_attribute: DiagnosticSeverity,
    pub no_invalid_html_nesting: DiagnosticSeverity,
    pub no_non_double_quoted_html_attribute_value: DiagnosticSeverity,
    pub no_self_closing_html_tag: DiagnosticSeverity,
    pub unsupported_erb_block_starter: DiagnosticSeverity,
}

impl Default for LintRuleSeverities {
    fn default() -> Self {
        Self {
            empty_erb_branch: DiagnosticSeverity::Error,
            empty_erb_code_tag: DiagnosticSeverity::Error,
            empty_erb_control_block: DiagnosticSeverity::Error,
            no_deprecated_html_tag: DiagnosticSeverity::Error,
            no_duplicate_html_attribute: DiagnosticSeverity::Error,
            no_invalid_html_boolean_attribute: DiagnosticSeverity::Error,
            no_invalid_html_nesting: DiagnosticSeverity::Error,
            no_non_double_quoted_html_attribute_value: DiagnosticSeverity::Error,
            no_self_closing_html_tag: DiagnosticSeverity::Error,
            unsupported_erb_block_starter: DiagnosticSeverity::Error,
        }
    }
}
