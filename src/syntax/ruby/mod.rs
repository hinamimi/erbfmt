#![allow(dead_code)]

mod ast;
mod error;
mod lexer;
mod parser;
mod parser_support;

#[cfg(test)]
mod tests;

pub(crate) use ast::*;
pub(crate) use parser::parse;
