mod config;
mod formatter;
mod html;
mod lexer;
mod linter;
mod mixed_parser;
#[cfg(test)]
mod parser;

use anyhow::{Context, Result, bail};
use clap::{ArgGroup, Parser};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "erbfmt",
    version,
    about = "Format and lint Ruby ERB templates",
    group(
    ArgGroup::new("mode")
        .args(["write", "check", "lint"])
        .multiple(false)
    )
)]
struct Args {
    #[arg(required = true, value_name = "FILE")]
    files: Vec<PathBuf>,

    #[arg(long, help = "Write formatted output back to files")]
    write: bool,

    #[arg(long, help = "Check whether files are already formatted")]
    check: bool,

    #[arg(long, help = "Run lint diagnostics instead of formatting")]
    lint: bool,

    #[arg(long, value_name = "PATH", help = "Path to erbfmt.json")]
    config: Option<PathBuf>,
}

fn main() -> Result<ExitCode> {
    let args = Args::parse();

    if args.files.len() > 1 && !args.write && !args.check && !args.lint {
        bail!("multiple files require --write, --check, or --lint");
    }

    let config = config::Config::load(args.config.as_deref())?;

    let mut failed = false;
    for file in &args.files {
        if run_file(&args, &config, file)? == FileStatus::Failed {
            failed = true;
        }
    }

    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
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
        return run_lint(file, &content, config);
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

fn run_lint(file: &Path, content: &str, config: &config::Config) -> Result<FileStatus> {
    let diagnostics = linter::lint_with_options(content, config.lint_options());

    if diagnostics.is_empty() {
        println!("{}: no lint issues found.", file.display());
        return Ok(FileStatus::Passed);
    }

    let has_errors = diagnostics.iter().any(linter::Diagnostic::is_error);

    for diagnostic in &diagnostics {
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

    Ok(if has_errors {
        FileStatus::Failed
    } else {
        FileStatus::Passed
    })
}

fn format_content(file: &Path, content: &str, config: &config::Config) -> Result<String> {
    if !config.formatter.enabled {
        return Ok(content.to_string());
    }

    let tokens = lexer::tokenize_with_spans(content)
        .with_context(|| format!("failed to lex `{}`", file.display()))?;
    let document = mixed_parser::parse_spanned(&tokens)
        .with_context(|| format!("failed to parse `{}`", file.display()))?;

    Ok(formatter::format_document_with_options(
        &document,
        config.format_options(),
    ))
}
