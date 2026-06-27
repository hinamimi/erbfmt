use crate::{
    html::{self, HtmlToken},
    lexer,
};

use super::{Diagnostic, LintOptions};

pub(super) struct HtmlElementLintFrame {
    name: String,
}

pub(super) fn html_tokens_have_meaningful_content(tokens: &[html::SpannedHtmlToken]) -> bool {
    tokens.iter().any(|spanned| match &spanned.token {
        HtmlToken::Text(text) => !text.trim().is_empty(),
        HtmlToken::Comment(_) => false,
        HtmlToken::OpenTag(_)
        | HtmlToken::CloseTag(_)
        | HtmlToken::SelfClosingTag(_)
        | HtmlToken::VoidTag(_)
        | HtmlToken::Doctype(_) => true,
    })
}

pub(super) fn lint_html_tokens(
    input: &str,
    fragment_start: usize,
    tokens: &[html::SpannedHtmlToken],
    stack: &mut Vec<HtmlElementLintFrame>,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for spanned in tokens {
        match &spanned.token {
            HtmlToken::OpenTag(tag) => {
                lint_html_content_model(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    stack,
                    options,
                    diagnostics,
                );
                lint_deprecated_html_tag(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                lint_duplicate_html_attributes(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                lint_invalid_html_boolean_attributes(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                lint_non_double_quoted_html_attribute_values(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                stack.push(HtmlElementLintFrame {
                    name: tag.name.clone(),
                });
            }
            HtmlToken::VoidTag(tag) => {
                lint_html_content_model(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    stack,
                    options,
                    diagnostics,
                );
                lint_deprecated_html_tag(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                lint_duplicate_html_attributes(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                lint_invalid_html_boolean_attributes(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                lint_non_double_quoted_html_attribute_values(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
            }
            HtmlToken::SelfClosingTag(tag) => {
                lint_html_content_model(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    stack,
                    options,
                    diagnostics,
                );
                lint_self_closing_html_tag(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                lint_deprecated_html_tag(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                lint_duplicate_html_attributes(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                lint_invalid_html_boolean_attributes(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                lint_non_double_quoted_html_attribute_values(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
            }
            HtmlToken::CloseTag(tag) => close_html_lint_frame(stack, &tag.name),
            HtmlToken::Text(text) => {
                lint_html_text_content_model(
                    input,
                    fragment_start,
                    spanned.span.start,
                    text,
                    stack,
                    options,
                    diagnostics,
                );
            }
            HtmlToken::Comment(_) | HtmlToken::Doctype(_) => {}
        }
    }
}

fn close_html_lint_frame(stack: &mut Vec<HtmlElementLintFrame>, name: &str) {
    let Some(frame) = stack.pop() else {
        return;
    };

    if !frame.name.eq_ignore_ascii_case(name) {
        stack.push(frame);
    }
}

fn lint_html_content_model(
    input: &str,
    fragment_start: usize,
    html_token_start: usize,
    tag: &html::HtmlTag,
    stack: &[HtmlElementLintFrame],
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !options.rules.no_invalid_html_nesting {
        return;
    }

    let Some(parent) = stack.last() else {
        return;
    };

    let parent_name = parent.name.to_ascii_lowercase();
    let child_name = tag.name.to_ascii_lowercase();
    let Some(message) = invalid_html_child_message(&parent_name, &child_name) else {
        return;
    };

    diagnostics.push(Diagnostic::located_with_severity(
        message,
        lexer::source_location(input, fragment_start + html_token_start),
        options.rule_severities.no_invalid_html_nesting,
    ));
}

fn lint_html_text_content_model(
    input: &str,
    fragment_start: usize,
    html_token_start: usize,
    text: &str,
    stack: &[HtmlElementLintFrame],
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !options.rules.no_invalid_html_nesting || text.trim().is_empty() {
        return;
    }

    let Some(parent) = stack.last() else {
        return;
    };

    let parent_name = parent.name.to_ascii_lowercase();

    if !matches!(
        parent_name.as_str(),
        "ul" | "ol" | "menu" | "table" | "thead" | "tbody" | "tfoot" | "tr" | "colgroup"
    ) {
        return;
    }

    diagnostics.push(Diagnostic::located_with_severity(
        format!("invalid HTML nesting: <{parent_name}> cannot have text as a direct child"),
        lexer::source_location(
            input,
            fragment_start + html_token_start + first_non_whitespace_offset(text),
        ),
        options.rule_severities.no_invalid_html_nesting,
    ));
}

fn invalid_html_child_message(parent: &str, child: &str) -> Option<String> {
    if parent == "p" && !is_phrasing_html_tag(child) {
        return Some(format!(
            "invalid HTML nesting: <p> cannot contain <{child}>"
        ));
    }

    if matches!(parent, "ul" | "ol" | "menu") && !matches!(child, "li" | "script" | "template") {
        return Some(format!(
            "invalid HTML nesting: <{parent}> cannot have <{child}> as a direct child"
        ));
    }

    if parent == "table"
        && !matches!(
            child,
            "caption" | "colgroup" | "thead" | "tbody" | "tfoot" | "tr" | "script" | "template"
        )
    {
        return Some(format!(
            "invalid HTML nesting: <table> cannot have <{child}> as a direct child"
        ));
    }

    if matches!(parent, "thead" | "tbody" | "tfoot")
        && !matches!(child, "tr" | "script" | "template")
    {
        return Some(format!(
            "invalid HTML nesting: <{parent}> cannot have <{child}> as a direct child"
        ));
    }

    if parent == "tr" && !matches!(child, "td" | "th" | "script" | "template") {
        return Some(format!(
            "invalid HTML nesting: <tr> cannot have <{child}> as a direct child"
        ));
    }

    if parent == "colgroup" && !matches!(child, "col" | "template") {
        return Some(format!(
            "invalid HTML nesting: <colgroup> cannot have <{child}> as a direct child"
        ));
    }

    None
}

fn is_phrasing_html_tag(name: &str) -> bool {
    matches!(
        name,
        "a" | "abbr"
            | "area"
            | "audio"
            | "b"
            | "bdi"
            | "bdo"
            | "br"
            | "button"
            | "canvas"
            | "cite"
            | "code"
            | "data"
            | "datalist"
            | "del"
            | "dfn"
            | "em"
            | "embed"
            | "i"
            | "iframe"
            | "img"
            | "input"
            | "ins"
            | "kbd"
            | "label"
            | "link"
            | "map"
            | "mark"
            | "math"
            | "meta"
            | "meter"
            | "noscript"
            | "object"
            | "output"
            | "picture"
            | "progress"
            | "q"
            | "ruby"
            | "s"
            | "samp"
            | "script"
            | "select"
            | "slot"
            | "small"
            | "span"
            | "strong"
            | "sub"
            | "sup"
            | "svg"
            | "template"
            | "textarea"
            | "time"
            | "u"
            | "var"
            | "video"
            | "wbr"
    )
}

fn first_non_whitespace_offset(text: &str) -> usize {
    text.char_indices()
        .find_map(|(index, ch)| (!ch.is_whitespace()).then_some(index))
        .unwrap_or(0)
}

fn lint_self_closing_html_tag(
    input: &str,
    fragment_start: usize,
    html_token_start: usize,
    tag: &html::HtmlTag,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !options.rules.no_self_closing_html_tag {
        return;
    }

    diagnostics.push(Diagnostic::located_with_severity(
        format!("self-closing HTML tag `{}` is not valid HTML5", tag.raw),
        lexer::source_location(input, fragment_start + html_token_start),
        options.rule_severities.no_self_closing_html_tag,
    ));
}

fn lint_deprecated_html_tag(
    input: &str,
    fragment_start: usize,
    html_token_start: usize,
    tag: &html::HtmlTag,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !options.rules.no_deprecated_html_tag || !is_deprecated_html_tag(&tag.name) {
        return;
    }

    diagnostics.push(Diagnostic::located_with_severity(
        format!("deprecated HTML tag `{}`", tag.raw),
        lexer::source_location(input, fragment_start + html_token_start),
        options.rule_severities.no_deprecated_html_tag,
    ));
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HtmlAttribute {
    name: String,
    offset: usize,
    value: Option<HtmlAttributeValue>,
    raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HtmlAttributeValue {
    raw: String,
    quote: Option<char>,
}

fn lint_duplicate_html_attributes(
    input: &str,
    fragment_start: usize,
    html_token_start: usize,
    tag: &html::HtmlTag,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !options.rules.no_duplicate_html_attribute || tag.raw.contains("<%") {
        return;
    }

    let attributes = html_attributes(tag);
    let mut seen: Vec<HtmlAttribute> = Vec::new();

    for attribute in attributes {
        if seen
            .iter()
            .any(|seen_attribute| seen_attribute.name == attribute.name)
        {
            diagnostics.push(Diagnostic::located_with_severity(
                format!("duplicate HTML attribute `{}`", attribute.name),
                lexer::source_location(input, fragment_start + html_token_start + attribute.offset),
                options.rule_severities.no_duplicate_html_attribute,
            ));
        } else {
            seen.push(attribute);
        }
    }
}

fn lint_invalid_html_boolean_attributes(
    input: &str,
    fragment_start: usize,
    html_token_start: usize,
    tag: &html::HtmlTag,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !options.rules.no_invalid_html_boolean_attribute || tag.raw.contains("<%") {
        return;
    }

    for attribute in html_attributes(tag) {
        if !is_html_boolean_attribute(&attribute.name) {
            continue;
        }

        let Some(value) = attribute.value else {
            continue;
        };

        let message = if value.raw.eq_ignore_ascii_case("false") {
            Some(format!(
                "invalid HTML boolean attribute value `{}=\"{}\"`",
                attribute.name, value.raw
            ))
        } else if value.raw.eq_ignore_ascii_case(&attribute.name) {
            Some(format!(
                "redundant HTML boolean attribute value `{}=\"{}\"`",
                attribute.name, value.raw
            ))
        } else {
            None
        };

        if let Some(message) = message {
            diagnostics.push(Diagnostic::located_with_severity(
                message,
                lexer::source_location(input, fragment_start + html_token_start + attribute.offset),
                options.rule_severities.no_invalid_html_boolean_attribute,
            ));
        }
    }
}

fn lint_non_double_quoted_html_attribute_values(
    input: &str,
    fragment_start: usize,
    html_token_start: usize,
    tag: &html::HtmlTag,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !options.rules.no_non_double_quoted_html_attribute_value {
        return;
    }

    for attribute in html_attributes(tag) {
        let Some(value) = attribute.value else {
            continue;
        };

        if value.quote == Some('"') {
            continue;
        }

        diagnostics.push(Diagnostic::located_with_severity(
            format!(
                "HTML attribute value must use double quotes `{}`",
                attribute.raw
            ),
            lexer::source_location(input, fragment_start + html_token_start + attribute.offset),
            options
                .rule_severities
                .no_non_double_quoted_html_attribute_value,
        ));
    }
}

fn html_attributes(tag: &html::HtmlTag) -> Vec<HtmlAttribute> {
    let Some(mut cursor) = tag.raw.find(&tag.name).map(|index| index + tag.name.len()) else {
        return Vec::new();
    };

    let mut attributes = Vec::new();
    let raw = tag.raw.as_str();

    while cursor < raw.len() {
        cursor = skip_html_attribute_spacing(raw, cursor);

        if cursor >= raw.len() || raw[cursor..].starts_with('>') || raw[cursor..].starts_with("/>")
        {
            break;
        }

        let name_start = cursor;
        let Some(name_end) = read_html_attribute_name_end(raw, name_start) else {
            break;
        };

        if name_end == name_start {
            break;
        }

        let mut attribute = HtmlAttribute {
            name: raw[name_start..name_end].to_ascii_lowercase(),
            offset: name_start,
            value: None,
            raw: raw[name_start..name_end].to_string(),
        };

        cursor = skip_html_attribute_spacing(raw, name_end);

        if raw[cursor..].starts_with('=') {
            let (next_cursor, value) = read_html_attribute_value(raw, cursor + '='.len_utf8());
            attribute.value = value;
            cursor = next_cursor;
        }

        attribute.raw = raw[name_start..cursor].to_string();
        attributes.push(attribute);
    }

    attributes
}

fn skip_html_attribute_spacing(raw: &str, mut cursor: usize) -> usize {
    while cursor < raw.len() {
        let ch = raw[cursor..]
            .chars()
            .next()
            .expect("cursor is inside raw tag");

        if !ch.is_whitespace() {
            break;
        }

        cursor += ch.len_utf8();
    }

    cursor
}

fn read_html_attribute_name_end(raw: &str, start: usize) -> Option<usize> {
    let mut cursor = start;

    while cursor < raw.len() {
        let ch = raw[cursor..]
            .chars()
            .next()
            .expect("cursor is inside raw tag");

        if ch.is_whitespace() || matches!(ch, '=' | '>' | '/') {
            break;
        }

        if matches!(ch, '"' | '\'' | '<') {
            return None;
        }

        cursor += ch.len_utf8();
    }

    Some(cursor)
}

fn read_html_attribute_value(raw: &str, cursor: usize) -> (usize, Option<HtmlAttributeValue>) {
    let mut cursor = skip_html_attribute_spacing(raw, cursor);

    let Some(first) = raw[cursor..].chars().next() else {
        return (cursor, None);
    };

    if first == '"' || first == '\'' {
        cursor += first.len_utf8();
        let value_start = cursor;

        while cursor < raw.len() {
            let ch = raw[cursor..]
                .chars()
                .next()
                .expect("cursor is inside raw tag");
            cursor += ch.len_utf8();

            if ch == first {
                let value_end = cursor - ch.len_utf8();
                return (
                    cursor,
                    Some(HtmlAttributeValue {
                        raw: raw[value_start..value_end].to_string(),
                        quote: Some(first),
                    }),
                );
            }
        }

        return (
            cursor,
            Some(HtmlAttributeValue {
                raw: raw[value_start..cursor].to_string(),
                quote: Some(first),
            }),
        );
    }

    let value_start = cursor;

    while cursor < raw.len() {
        if raw[cursor..].starts_with("<%") {
            let Some(relative_end) = raw[cursor + "<%".len()..].find("%>") else {
                break;
            };
            cursor += "<%".len() + relative_end + "%>".len();
            continue;
        }

        let ch = raw[cursor..]
            .chars()
            .next()
            .expect("cursor is inside raw tag");

        if ch.is_whitespace() || ch == '>' {
            break;
        }

        cursor += ch.len_utf8();
    }

    if cursor == value_start {
        (cursor, None)
    } else {
        (
            cursor,
            Some(HtmlAttributeValue {
                raw: raw[value_start..cursor].to_string(),
                quote: None,
            }),
        )
    }
}

fn is_html_boolean_attribute(name: &str) -> bool {
    matches!(
        name,
        "allowfullscreen"
            | "async"
            | "autofocus"
            | "autoplay"
            | "checked"
            | "controls"
            | "default"
            | "defer"
            | "disabled"
            | "formnovalidate"
            | "hidden"
            | "inert"
            | "ismap"
            | "itemscope"
            | "loop"
            | "multiple"
            | "muted"
            | "nomodule"
            | "novalidate"
            | "open"
            | "playsinline"
            | "readonly"
            | "required"
            | "reversed"
            | "selected"
    )
}

fn is_deprecated_html_tag(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "acronym"
            | "applet"
            | "basefont"
            | "big"
            | "center"
            | "dir"
            | "font"
            | "frame"
            | "frameset"
            | "isindex"
            | "marquee"
            | "noframes"
            | "strike"
            | "tt"
            | "xmp"
    )
}
