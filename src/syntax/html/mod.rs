mod tag;
mod token;
mod tokenizer;

#[cfg(test)]
mod tests;

pub use token::{HtmlTag, HtmlToken, SpannedHtmlToken};
pub use tokenizer::{tokenize, tokenize_with_spans};
