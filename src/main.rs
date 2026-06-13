mod lexer;
mod parser;

use anyhow::Result;
use clap::Parser;
use std::fs;

#[derive(Parser)]
struct Args {
    file: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let content = fs::read_to_string(args.file)?;

    let tokens = lexer::tokenize(&content)?;
    let document = parser::parse(&tokens)?;

    println!("{document:#?}");

    Ok(())
}
