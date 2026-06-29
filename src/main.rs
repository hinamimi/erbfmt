mod config;
mod formatter;
mod html;
mod ignore;
mod lexer;
mod linter;
mod mixed_parser;

use anyhow::{Context, Result, bail};
use clap::{ArgGroup, Parser, ValueEnum};
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const CONFIG_FILE: &str = "erbfmt.json";
const DEFAULT_CONFIG: &str = r#"{
  "$schema": "https://raw.githubusercontent.com/hinamimi/erbfmt/refs/heads/main/docs/schema/erbfmt.schema.json",
  "files": {
    "includes": ["**/*.html.erb"]
  },
  "parser": {
    "allowHtmlOptionalClosingTags": false
  },
  "formatter": {
    "enabled": true,
    "indentStyle": "space",
    "indentWidth": 2,
    "indentHtml": true,
    "lineEnding": "lf",
    "lineWidth": 80,
    "trailingNewline": true
  },
  "linter": {
    "enabled": true,
    "rules": {
      "recommended": true,
      "emptyErbBranch": "error",
      "emptyErbCodeTag": "error",
      "emptyErbControlBlock": "error",
      "noDeprecatedHtmlTag": "error",
      "noDuplicateHtmlAttribute": "error",
      "noInvalidHtmlBooleanAttribute": "error",
      "noInvalidHtmlNesting": "error",
      "noNonDoubleQuotedHtmlAttributeValue": "error",
      "noSelfClosingHtmlTag": "error",
      "unsupportedErbBlockStarter": "error"
    }
  }
}
"#;

#[derive(Parser)]
#[command(
    name = "erbfmt",
    version,
    about = "Format and lint Ruby ERB templates",
    after_help = "Commands:\n  init  Create an erbfmt.json config file\n",
    group(
    ArgGroup::new("mode")
        .args(["write", "check", "lint"])
        .multiple(false)
    )
)]
struct Args {
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    #[arg(long, help = "Write formatted output back to files")]
    write: bool,

    #[arg(long, help = "Check whether files are already formatted")]
    check: bool,

    #[arg(long, help = "Run lint diagnostics instead of formatting")]
    lint: bool,

    #[arg(
        long,
        value_enum,
        default_value = "pretty",
        help = "Set lint output format"
    )]
    lint_format: LintFormat,

    #[arg(long, value_name = "PATH", help = "Path to erbfmt.json")]
    config: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum LintFormat {
    Pretty,
    Plain,
}

#[derive(Parser)]
#[command(name = "erbfmt init", about = "Create an erbfmt.json config file")]
struct InitArgs {
    #[arg(long, help = "Overwrite an existing erbfmt.json")]
    force: bool,
}

fn main() -> Result<ExitCode> {
    if let Some(init_args) = parse_init_args() {
        return run_init(init_args.force);
    }

    let args = Args::parse();

    if args.files.is_empty() {
        bail!("at least one file is required unless a subcommand is used");
    }

    if args.files.len() > 1 && !args.write && !args.check && !args.lint {
        bail!("multiple files require --write, --check, or --lint");
    }

    if args.files.iter().any(|file| file.is_dir()) && !args.write && !args.check && !args.lint {
        bail!("directory inputs require --write, --check, or --lint");
    }

    let config = config::Config::load(args.config.as_deref())?;
    let files = expand_input_files(&args.files, &config)?;

    let mut failed = false;
    for file in &files {
        match run_file(&args, &config, file) {
            Ok(FileStatus::Passed) => {}
            Ok(FileStatus::Failed) => failed = true,
            Err(error) if args.write || args.check || args.lint => {
                eprintln!("Error: {error:#}");
                failed = true;
            }
            Err(error) => return Err(error),
        }
    }

    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

fn parse_init_args() -> Option<InitArgs> {
    let mut args = std::env::args_os();
    let binary = args.next()?;

    if args.next().as_deref() != Some(std::ffi::OsStr::new("init")) {
        return None;
    }

    let init_args = std::iter::once(OsString::from(format!(
        "{} init",
        PathBuf::from(binary)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("erbfmt")
    )))
    .chain(args);

    Some(InitArgs::parse_from(init_args))
}

fn expand_input_files(inputs: &[PathBuf], config: &config::Config) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for input in inputs {
        if input.is_dir() {
            collect_directory_files(input, config, &mut files)?;
        } else if input.is_file() {
            if config.includes_file(input) {
                files.push(input.clone());
            }
        } else {
            bail!("`{}` is not a file or directory", input.display());
        }
    }

    files.sort();
    files.dedup();

    Ok(files)
}

fn collect_directory_files(
    directory: &Path,
    config: &config::Config,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    let mut entries = fs::read_dir(directory)
        .with_context(|| format!("failed to read directory `{}`", directory.display()))?
        .collect::<std::io::Result<Vec<_>>>()
        .with_context(|| format!("failed to read directory `{}`", directory.display()))?;

    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to inspect `{}`", path.display()))?;

        if file_type.is_dir() {
            collect_directory_files(&path, config, files)?;
        } else if file_type.is_file() && config.includes_discovered_file(&path) {
            files.push(path);
        }
    }

    Ok(())
}

fn run_init(force: bool) -> Result<ExitCode> {
    let path = PathBuf::from(CONFIG_FILE);

    if path.exists() && !force {
        bail!(
            "{} already exists. Use `erbfmt init --force` to overwrite it.",
            CONFIG_FILE
        );
    }

    fs::write(&path, DEFAULT_CONFIG)
        .with_context(|| format!("failed to write `{}`", path.display()))?;
    println!("{}: created config file.", path.display());

    Ok(ExitCode::SUCCESS)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileStatus {
    Passed,
    Failed,
}

fn run_file(args: &Args, config: &config::Config, file: &Path) -> Result<FileStatus> {
    let content =
        fs::read_to_string(file).with_context(|| format!("failed to read `{}`", file.display()))?;

    if args.lint {
        return run_lint(file, &content, config, args.lint_format);
    }

    let formatted = format_content(file, &content, config)?;

    if args.write {
        fs::write(file, formatted)
            .with_context(|| format!("failed to write `{}`", file.display()))?;
        println!("{}: wrote formatted file.", file.display());
        return Ok(FileStatus::Passed);
    }

    if args.check {
        return Ok(if formatted == content {
            println!("{}: file is formatted.", file.display());
            FileStatus::Passed
        } else {
            eprintln!("{}: file is not formatted.", file.display());
            FileStatus::Failed
        });
    }

    if args.files.len() == 1 {
        print!("{formatted}");
        return Ok(FileStatus::Passed);
    }

    unreachable!("multiple files without a mode are rejected before processing")
}

fn run_lint(
    file: &Path,
    content: &str,
    config: &config::Config,
    lint_format: LintFormat,
) -> Result<FileStatus> {
    let diagnostics = linter::lint_with_options(content, config.lint_options());

    if diagnostics.is_empty() {
        println!("{}: no lint issues found.", file.display());
        return Ok(FileStatus::Passed);
    }

    let has_errors = diagnostics.iter().any(linter::Diagnostic::is_error);

    match lint_format {
        LintFormat::Pretty => print_pretty_lint_diagnostics(file, content, &diagnostics),
        LintFormat::Plain => print_plain_lint_diagnostics(file, &diagnostics),
    }

    Ok(if has_errors {
        FileStatus::Failed
    } else {
        FileStatus::Passed
    })
}

fn print_plain_lint_diagnostics(file: &Path, diagnostics: &[linter::Diagnostic]) {
    for diagnostic in diagnostics {
        if diagnostic.severity == linter::DiagnosticSeverity::Warning {
            eprintln!(
                "{}: warning: {}",
                file.display(),
                diagnostic.message_with_location()
            );
        } else {
            eprintln!("{}: {}", file.display(), diagnostic.message_with_location());
        }
    }
}

fn print_pretty_lint_diagnostics(file: &Path, content: &str, diagnostics: &[linter::Diagnostic]) {
    eprintln!("{}:", file.display());

    for diagnostic in diagnostics {
        eprintln!(
            "  {}: {}",
            lint_severity_label(diagnostic.severity),
            diagnostic.message
        );

        if let Some(location) = diagnostic.location {
            eprintln!("    --> {}", location);

            if let Some(line) = source_line(content, location.line) {
                eprintln!("     |");
                eprintln!("{:>4} | {}", location.line, line);
                eprintln!("     | {}^", caret_padding(line, location.column));
            }
        }

        eprintln!();
    }

    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == linter::DiagnosticSeverity::Error)
        .count();
    let warnings = diagnostics.len() - errors;

    eprintln!(
        "{} lint {} found ({} {}, {} {}).",
        diagnostics.len(),
        plural(diagnostics.len(), "issue", "issues"),
        errors,
        plural(errors, "error", "errors"),
        warnings,
        plural(warnings, "warning", "warnings")
    );
}

fn lint_severity_label(severity: linter::DiagnosticSeverity) -> &'static str {
    match severity {
        linter::DiagnosticSeverity::Warning => "warning",
        linter::DiagnosticSeverity::Error => "error",
    }
}

fn source_line(content: &str, line: usize) -> Option<&str> {
    line.checked_sub(1)
        .and_then(|index| content.lines().nth(index))
}

fn caret_padding(line: &str, column: usize) -> String {
    let column = column.saturating_sub(1);

    line.chars()
        .take(column)
        .map(|ch| if ch == '\t' { '\t' } else { ' ' })
        .collect()
}

fn plural(count: usize, singular: &'static str, plural: &'static str) -> &'static str {
    if count == 1 { singular } else { plural }
}

fn format_content(file: &Path, content: &str, config: &config::Config) -> Result<String> {
    if !config.formatter.enabled {
        return Ok(content.to_string());
    }

    let tokens = lexer::tokenize_with_spans(content)
        .with_context(|| format!("failed to lex `{}`", file.display()))?;
    let document = mixed_parser::parse_spanned_with_options(&tokens, config.parser_options())
        .with_context(|| format!("failed to parse `{}`", file.display()))?;

    Ok(formatter::format_document_with_source(
        &document,
        content,
        config.format_options(),
    ))
}
