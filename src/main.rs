mod formatter;
mod html;
mod lexer;
mod linter;
mod mixed_parser;
#[cfg(test)]
mod parser;

use anyhow::{Context, Result};
use clap::{ArgGroup, Parser};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(group(
    ArgGroup::new("mode")
        .args(["write", "check", "lint"])
        .multiple(false)
))]
struct Args {
    file: PathBuf,

    #[arg(long, conflicts_with = "lint")]
    no_html_indent: bool,

    #[arg(long)]
    write: bool,

    #[arg(long)]
    check: bool,

    #[arg(long)]
    lint: bool,
}

fn main() -> Result<ExitCode> {
    let args = Args::parse();

    let content = fs::read_to_string(&args.file)
        .with_context(|| format!("failed to read `{}`", args.file.display()))?;

    if args.lint {
        let diagnostics = linter::lint(&content);

        if diagnostics.is_empty() {
            println!("{}: no lint issues found.", args.file.display());
            return Ok(ExitCode::SUCCESS);
        }

        for diagnostic in diagnostics {
            eprintln!("{}: {}", args.file.display(), diagnostic.message);
        }

        return Ok(ExitCode::FAILURE);
    }

    let tokens = lexer::tokenize(&content)
        .with_context(|| format!("failed to lex `{}`", args.file.display()))?;
    let document = mixed_parser::parse(&tokens)
        .with_context(|| format!("failed to parse `{}`", args.file.display()))?;
    let formatted = if args.no_html_indent {
        formatter::format_document_with_options(
            &document,
            formatter::FormatOptions { indent_html: false },
        )
    } else {
        formatter::format_document(&document)
    };

    if args.write {
        fs::write(&args.file, formatted)
            .with_context(|| format!("failed to write `{}`", args.file.display()))?;
    } else if args.check {
        if formatted == content {
            println!("{}: file is formatted.", args.file.display());
        } else {
            eprintln!("{}: file is not formatted.", args.file.display());
            return Ok(ExitCode::FAILURE);
        }
    } else {
        print!("{formatted}");
    }

    Ok(ExitCode::SUCCESS)
}
