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

    #[arg(long, conflicts_with = "lint", help = "Disable HTML tag indentation")]
    no_html_indent: bool,

    #[arg(long, help = "Write formatted output back to files")]
    write: bool,

    #[arg(long, help = "Check whether files are already formatted")]
    check: bool,

    #[arg(long, help = "Run lint diagnostics instead of formatting")]
    lint: bool,
}

fn main() -> Result<ExitCode> {
    let args = Args::parse();

    if args.files.len() > 1 && !args.write && !args.check && !args.lint {
        bail!("multiple files require --write, --check, or --lint");
    }

    let mut failed = false;
    for file in &args.files {
        if run_file(&args, file)? == FileStatus::Failed {
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

fn run_file(args: &Args, file: &Path) -> Result<FileStatus> {
    let content =
        fs::read_to_string(file).with_context(|| format!("failed to read `{}`", file.display()))?;

    if args.lint {
        return run_lint(file, &content);
    }

    let formatted = format_content(file, &content, args.no_html_indent)?;

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

fn run_lint(file: &Path, content: &str) -> Result<FileStatus> {
    let diagnostics = linter::lint(content);

    if diagnostics.is_empty() {
        println!("{}: no lint issues found.", file.display());
        return Ok(FileStatus::Passed);
    }

    for diagnostic in diagnostics {
        eprintln!("{}: {}", file.display(), diagnostic.message);
    }

    Ok(FileStatus::Failed)
}

fn format_content(file: &Path, content: &str, no_html_indent: bool) -> Result<String> {
    let tokens =
        lexer::tokenize(content).with_context(|| format!("failed to lex `{}`", file.display()))?;
    let document = mixed_parser::parse(&tokens)
        .with_context(|| format!("failed to parse `{}`", file.display()))?;

    Ok(if no_html_indent {
        formatter::format_document_with_options(
            &document,
            formatter::FormatOptions { indent_html: false },
        )
    } else {
        formatter::format_document(&document)
    })
}
