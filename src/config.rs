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
    pub files: FilesConfig,
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

        let mut config = raw
            .into_config()
            .with_context(|| format!("failed to load config `{}`", path.display()))?;
        config.files.base_dir = path.parent().map(Path::to_path_buf);

        Ok(config)
    }

    pub fn format_options(&self) -> FormatOptions {
        self.formatter.options
    }

    pub fn lint_options(&self) -> LintOptions {
        self.linter.options
    }

    pub fn includes_file(&self, path: &Path) -> bool {
        self.files.includes(path)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilesConfig {
    pub includes: Vec<String>,
    base_dir: Option<PathBuf>,
}

impl FilesConfig {
    fn includes(&self, path: &Path) -> bool {
        if self.includes.is_empty() {
            return true;
        }

        let normalized_path = normalize_path_for_match(path);
        let relative_path = self.base_dir.as_deref().and_then(|base_dir| {
            path.strip_prefix(base_dir)
                .ok()
                .map(normalize_path_for_match)
        });
        let has_positive = self
            .includes
            .iter()
            .any(|pattern| !pattern.trim_start().starts_with('!'));
        let mut included = !has_positive;

        for pattern in &self.includes {
            let pattern = pattern.trim();
            if pattern.is_empty() {
                continue;
            }

            let (exclude, pattern) = pattern
                .strip_prefix('!')
                .map_or((false, pattern), |pattern| (true, pattern));
            let pattern = pattern.trim();

            if pattern.is_empty()
                || !path_matches_pattern(path, relative_path.as_deref(), &normalized_path, pattern)
            {
                continue;
            }

            included = !exclude;
        }

        included
    }
}

fn path_matches_pattern(
    path: &Path,
    relative_path: Option<&str>,
    normalized_path: &str,
    pattern: &str,
) -> bool {
    let normalized_pattern = normalize_pattern(pattern);

    relative_path.is_some_and(|relative_path| glob_matches(&normalized_pattern, relative_path))
        || glob_matches(&normalized_pattern, normalized_path)
        || path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| glob_matches(&normalized_pattern, name))
}

fn normalize_pattern(pattern: &str) -> String {
    pattern.trim().replace('\\', "/")
}

fn normalize_path_for_match(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            std::path::Component::Normal(part) => part.to_str().map(str::to_string),
            std::path::Component::RootDir => Some(String::new()),
            std::path::Component::Prefix(prefix) => {
                Some(prefix.as_os_str().to_string_lossy().into_owned())
            }
            std::path::Component::CurDir | std::path::Component::ParentDir => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn glob_matches(pattern: &str, path: &str) -> bool {
    let pattern_parts = pattern
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect::<Vec<_>>();
    let path_parts = path
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect::<Vec<_>>();

    glob_parts_match(&pattern_parts, &path_parts)
}

fn glob_parts_match(pattern: &[&str], path: &[&str]) -> bool {
    match pattern.split_first() {
        None => path.is_empty(),
        Some((part, rest)) if *part == "**" => {
            glob_parts_match(rest, path)
                || (!path.is_empty() && glob_parts_match(pattern, &path[1..]))
        }
        Some((pattern_part, rest)) => {
            let Some((path_part, path_rest)) = path.split_first() else {
                return false;
            };

            glob_segment_matches(pattern_part, path_part) && glob_parts_match(rest, path_rest)
        }
    }
}

fn glob_segment_matches(pattern: &str, text: &str) -> bool {
    glob_segment_matches_chars(
        &pattern.chars().collect::<Vec<_>>(),
        &text.chars().collect::<Vec<_>>(),
    )
}

fn glob_segment_matches_chars(pattern: &[char], text: &[char]) -> bool {
    match pattern.split_first() {
        None => text.is_empty(),
        Some(('*', rest)) => {
            glob_segment_matches_chars(rest, text)
                || (!text.is_empty() && glob_segment_matches_chars(pattern, &text[1..]))
        }
        Some(('?', rest)) => !text.is_empty() && glob_segment_matches_chars(rest, &text[1..]),
        Some((expected, rest)) => text.first().is_some_and(|actual| {
            actual == expected && glob_segment_matches_chars(rest, &text[1..])
        }),
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
    files: Option<RawFilesConfig>,
    formatter: Option<RawFormatterConfig>,
    linter: Option<RawLinterConfig>,
}

impl RawConfig {
    fn into_config(self) -> Result<Config> {
        let mut config = Config::default();

        if let Some(formatter) = self.formatter {
            config.formatter = formatter.into_config()?;
        }

        if let Some(files) = self.files {
            config.files = files.into_config();
        }

        if let Some(linter) = self.linter {
            config.linter = linter.into_config();
        }

        Ok(config)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawFilesConfig {
    includes: Option<Vec<String>>,
}

impl RawFilesConfig {
    fn into_config(self) -> FilesConfig {
        FilesConfig {
            includes: self.includes.unwrap_or_default(),
            base_dir: None,
        }
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
