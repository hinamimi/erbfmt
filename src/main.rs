mod formatter;
mod lexer;
mod parser;

use anyhow::Result;
use clap::Parser;
use std::fs;

#[derive(Parser)]
struct Args {
    file: String,

    #[arg(long)]
    no_html_indent: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let content = fs::read_to_string(args.file)?;

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

    print!("{formatted}");

    Ok(())
}
