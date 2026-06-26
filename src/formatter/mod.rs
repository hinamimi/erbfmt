mod engine;
mod erb;
mod erb_block;
mod ignore_directive;
mod inline;
mod options;
mod preserve;
mod ruby_wrap;
mod tag;

use crate::mixed_parser::Document;

pub use options::{FormatOptions, IndentStyle, LineEnding};

#[allow(dead_code)]
pub fn format_document(document: &Document) -> String {
    engine::format_document(document)
}

#[allow(dead_code)]
pub fn format_document_with_options(document: &Document, options: FormatOptions) -> String {
    engine::format_document_with_options(document, options)
}

pub fn format_document_with_source(
    document: &Document,
    source: &str,
    options: FormatOptions,
) -> String {
    engine::format_document_with_source(document, source, options)
}

#[cfg(test)]
mod tests;
