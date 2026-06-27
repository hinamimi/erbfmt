use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::{
    formatter::{FormatOptions, IndentStyle, LineEnding},
    linter::{DiagnosticSeverity, LintOptions, LintRuleSeverities, LintRules},
};

const CONFIG_FILE: &str = "erbfmt.json";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    pub formatter: FormatterConfig,
    pub linter: LinterConfig,
}

impl Config {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let Some(path) = path.map(Path::to_path_buf).or_else(find_config) else {
            return Ok(Self::default());
        };

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read config `{}`", path.display()))?;
        let raw: RawConfig = serde_json::from_str(&content)
            .with_context(|| format!("failed to parse config `{}`", path.display()))?;

        raw.into_config()
            .with_context(|| format!("failed to load config `{}`", path.display()))
    }

    pub fn format_options(&self) -> FormatOptions {
        self.formatter.options
    }

    pub fn lint_options(&self) -> LintOptions {
        self.linter.options
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatterConfig {
    pub enabled: bool,
    pub options: FormatOptions,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            options: FormatOptions::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LinterConfig {
    pub options: LintOptions,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawConfig {
    formatter: Option<RawFormatterConfig>,
    linter: Option<RawLinterConfig>,
}

impl RawConfig {
    fn into_config(self) -> Result<Config> {
        let mut config = Config::default();

        if let Some(formatter) = self.formatter {
            config.formatter = formatter.into_config()?;
        }

        if let Some(linter) = self.linter {
            config.linter = linter.into_config();
        }

        Ok(config)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawFormatterConfig {
    enabled: Option<bool>,
    indent_style: Option<IndentStyle>,
    indent_width: Option<usize>,
    indent_html: Option<bool>,
    line_ending: Option<LineEnding>,
    line_width: Option<usize>,
    trailing_newline: Option<bool>,
}

impl RawFormatterConfig {
    fn into_config(self) -> Result<FormatterConfig> {
        let mut config = FormatterConfig::default();

        if let Some(enabled) = self.enabled {
            config.enabled = enabled;
        }

        if let Some(indent_style) = self.indent_style {
            config.options.indent_style = indent_style;
        }

        if let Some(indent_width) = self.indent_width {
            if indent_width == 0 {
                bail!("formatter.indentWidth must be greater than 0");
            }

            config.options.indent_width = indent_width;
        }

        if let Some(indent_html) = self.indent_html {
            config.options.indent_html = indent_html;
        }

        if let Some(line_ending) = self.line_ending {
            config.options.line_ending = line_ending;
        }

        if let Some(line_width) = self.line_width {
            if line_width == 0 {
                bail!("formatter.lineWidth must be greater than 0");
            }

            config.options.line_width = line_width;
        }

        if let Some(trailing_newline) = self.trailing_newline {
            config.options.trailing_newline = trailing_newline;
        }

        Ok(config)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLinterConfig {
    enabled: Option<bool>,
    rules: Option<RawLintRules>,
}

impl RawLinterConfig {
    fn into_config(self) -> LinterConfig {
        let mut config = LinterConfig::default();

        if let Some(enabled) = self.enabled {
            config.options.enabled = enabled;
        }

        if let Some(rules) = self.rules {
            let rule_config = rules.into_rule_config();
            config.options.rules = rule_config.rules;
            config.options.rule_severities = rule_config.severities;
        }

        config
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLintRules {
    recommended: Option<bool>,
    empty_erb_branch: Option<RuleSetting>,
    empty_erb_code_tag: Option<RuleSetting>,
    empty_erb_control_block: Option<RuleSetting>,
    no_deprecated_html_tag: Option<RuleSetting>,
    no_duplicate_html_attribute: Option<RuleSetting>,
    no_invalid_html_boolean_attribute: Option<RuleSetting>,
    no_invalid_html_nesting: Option<RuleSetting>,
    no_non_double_quoted_html_attribute_value: Option<RuleSetting>,
    no_self_closing_html_tag: Option<RuleSetting>,
    unsupported_erb_block_starter: Option<RuleSetting>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LintRuleConfig {
    rules: LintRules,
    severities: LintRuleSeverities,
}

impl RawLintRules {
    fn into_rule_config(self) -> LintRuleConfig {
        let recommended = self.recommended.unwrap_or(true);

        LintRuleConfig {
            rules: LintRules {
                empty_erb_branch: rule_enabled(self.empty_erb_branch, recommended),
                empty_erb_code_tag: rule_enabled(self.empty_erb_code_tag, recommended),
                empty_erb_control_block: rule_enabled(self.empty_erb_control_block, recommended),
                no_deprecated_html_tag: rule_enabled(self.no_deprecated_html_tag, recommended),
                no_duplicate_html_attribute: rule_enabled(
                    self.no_duplicate_html_attribute,
                    recommended,
                ),
                no_invalid_html_boolean_attribute: rule_enabled(
                    self.no_invalid_html_boolean_attribute,
                    recommended,
                ),
                no_invalid_html_nesting: rule_enabled(self.no_invalid_html_nesting, recommended),
                no_non_double_quoted_html_attribute_value: rule_enabled(
                    self.no_non_double_quoted_html_attribute_value,
                    recommended,
                ),
                no_self_closing_html_tag: rule_enabled(self.no_self_closing_html_tag, recommended),
                unsupported_erb_block_starter: rule_enabled(
                    self.unsupported_erb_block_starter,
                    recommended,
                ),
            },
            severities: LintRuleSeverities {
                empty_erb_branch: rule_severity(self.empty_erb_branch),
                empty_erb_code_tag: rule_severity(self.empty_erb_code_tag),
                empty_erb_control_block: rule_severity(self.empty_erb_control_block),
                no_deprecated_html_tag: rule_severity(self.no_deprecated_html_tag),
                no_duplicate_html_attribute: rule_severity(self.no_duplicate_html_attribute),
                no_invalid_html_boolean_attribute: rule_severity(
                    self.no_invalid_html_boolean_attribute,
                ),
                no_invalid_html_nesting: rule_severity(self.no_invalid_html_nesting),
                no_non_double_quoted_html_attribute_value: rule_severity(
                    self.no_non_double_quoted_html_attribute_value,
                ),
                no_self_closing_html_tag: rule_severity(self.no_self_closing_html_tag),
                unsupported_erb_block_starter: rule_severity(self.unsupported_erb_block_starter),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
enum RuleSetting {
    Off,
    Warn,
    Error,
}

impl RuleSetting {
    fn is_enabled(self) -> bool {
        !matches!(self, Self::Off)
    }

    fn diagnostic_severity(self) -> DiagnosticSeverity {
        match self {
            Self::Warn => DiagnosticSeverity::Warning,
            Self::Error | Self::Off => DiagnosticSeverity::Error,
        }
    }
}

fn rule_enabled(setting: Option<RuleSetting>, recommended: bool) -> bool {
    setting.map(RuleSetting::is_enabled).unwrap_or(recommended)
}

fn rule_severity(setting: Option<RuleSetting>) -> DiagnosticSeverity {
    setting
        .map(RuleSetting::diagnostic_severity)
        .unwrap_or(DiagnosticSeverity::Error)
}

fn find_config() -> Option<PathBuf> {
    let mut directory = std::env::current_dir().ok()?;

    loop {
        let candidate = directory.join(CONFIG_FILE);
        if candidate.is_file() {
            return Some(candidate);
        }

        if !directory.pop() {
            return None;
        }
    }
}
