mod classifier;
mod error;
mod html_context;
mod location;
mod token;

#[cfg(test)]
mod tests;

use classifier::{classify_code, classify_output_code};
pub use error::LexError;
use html_context::is_inside_html_tag;
use location::spanned_token;
pub use location::{SourceLocation, SpannedToken, source_location};
pub use token::{
    ErbBlockKind, ErbBranchKind, ErbTag, ErbTagClose, ErbTagOpen, ErbTagSyntax, Token,
};

#[cfg(test)]
pub fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
    Ok(tokenize_with_spans(input)?
        .into_iter()
        .map(|spanned| spanned.token)
        .collect())
}

pub fn tokenize_with_spans(input: &str) -> Result<Vec<SpannedToken>, LexError> {
    let mut tokens = Vec::new();
    let mut cursor = 0;
    let mut search_cursor = 0;

    while let Some(relative_start) = input[search_cursor..].find("<%") {
        let start = search_cursor + relative_start;

        if is_inside_html_tag(input, start) {
            search_cursor = start + "<%".len();
            continue;
        }

        if start > cursor {
            tokens.push(spanned_token(
                input,
                cursor,
                start,
                Token::Html(input[cursor..start].to_string()),
            ));
        }

        let tag_content_start = start + "<%".len();

        if input[tag_content_start..].starts_with('%') {
            return Err(LexError::unsupported_erb_marker(input, start, "<%%"));
        }

        let (open, is_output, is_comment, code_start) =
            if input[tag_content_start..].starts_with("==") {
                (
                    ErbTagOpen::RawOutput,
                    true,
                    false,
                    tag_content_start + "==".len(),
                )
            } else if input[tag_content_start..].starts_with('=') {
                (
                    ErbTagOpen::Output,
                    true,
                    false,
                    tag_content_start + "=".len(),
                )
            } else if input[tag_content_start..].starts_with('#') {
                (
                    ErbTagOpen::Comment,
                    false,
                    true,
                    tag_content_start + "#".len(),
                )
            } else if input[tag_content_start..].starts_with('-') {
                (
                    ErbTagOpen::TrimCode,
                    false,
                    false,
                    tag_content_start + "-".len(),
                )
            } else {
                (ErbTagOpen::Code, false, false, tag_content_start)
            };

        let Some(relative_end) = input[code_start..].find("%>") else {
            return Err(LexError::unterminated_erb(input, start));
        };

        let mut code_end = code_start + relative_end;
        let token_end = code_end + "%>".len();
        let close = if input[..code_end].ends_with('-') {
            code_end -= '-'.len_utf8();
            ErbTagClose::Trim
        } else {
            ErbTagClose::Normal
        };
        let code = input[code_start..code_end].trim().to_string();
        let tag = ErbTag::new(code, ErbTagSyntax { open, close });
        let token = if is_comment {
            Token::ErbComment(tag)
        } else if is_output {
            classify_output_code(tag)
        } else {
            classify_code(tag)
        };

        tokens.push(spanned_token(input, start, token_end, token));
        cursor = token_end;
        search_cursor = cursor;
    }

    if cursor < input.len() {
        tokens.push(spanned_token(
            input,
            cursor,
            input.len(),
            Token::Html(input[cursor..].to_string()),
        ));
    }

    Ok(tokens)
}
