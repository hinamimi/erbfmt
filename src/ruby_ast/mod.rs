#![allow(dead_code)]

mod ast;
mod error;
mod lexer;
mod parser;

#[cfg(test)]
mod tests;

pub(crate) use ast::*;
pub(crate) use parser::parse;
