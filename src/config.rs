use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::{
    formatter::{FormatOptions, IndentStyle, LineEnding},
    linter::{LintOptions, LintRules},
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
            config.options.rules = rules.into_rules();
        }

        config
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawLintRules {
    recommended: Option<bool>,
    empty_erb_control_block: Option<RuleSetting>,
    unsupported_erb_block_starter: Option<RuleSetting>,
}

impl RawLintRules {
    fn into_rules(self) -> LintRules {
        let recommended = self.recommended.unwrap_or(true);

        LintRules {
            empty_erb_control_block: self
                .empty_erb_control_block
                .map(RuleSetting::is_enabled)
                .unwrap_or(recommended),
            unsupported_erb_block_starter: self
                .unsupported_erb_block_starter
                .map(RuleSetting::is_enabled)
                .unwrap_or(recommended),
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
