mod formatter;
mod lexer;
mod linter;
mod parser;

use anyhow::Result;
use clap::Parser;
use std::fs;
use std::process::ExitCode;

#[derive(Parser)]
struct Args {
    file: String,

    #[arg(long)]
    no_html_indent: bool,

    #[arg(long)]
    write: bool,

    #[arg(long)]
    lint: bool,
}

fn main() -> Result<ExitCode> {
    let args = Args::parse();

    let content = fs::read_to_string(&args.file)?;

    if args.lint {
        let diagnostics = linter::lint(&content);

        if diagnostics.is_empty() {
            println!("No lint issues found.");
            return Ok(ExitCode::SUCCESS);
        }

        for diagnostic in diagnostics {
            eprintln!("{}", diagnostic.message);
        }

        return Ok(ExitCode::FAILURE);
    }

    let tokens = lexer::tokenize(&content)?;
    let document = parser::parse(&tokens)?;
    let formatted = if args.no_html_indent {
        formatter::format_document_with_options(
            &document,
            formatter::FormatOptions { indent_html: false },
        )
    } else {
        formatter::format_document(&document)
    };

    if args.write {
        fs::write(&args.file, formatted)?;
    } else {
        print!("{formatted}");
    }

    Ok(ExitCode::SUCCESS)
}
