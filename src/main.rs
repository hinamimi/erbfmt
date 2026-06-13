mod lexer;

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

    println!("{tokens:#?}");

    Ok(())
}
