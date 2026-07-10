use crate::syntax::{
    erb as lexer,
    html::{self, HtmlToken},
    template as mixed_parser,
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
                close_optional_html_lint_frames_before_open(stack, &tag.name, options);
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
                lint_unknown_html_attributes(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    stack,
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
                lint_unknown_html_attributes(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    stack,
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
                lint_unknown_html_attributes(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    stack,
                    options,
                    diagnostics,
                );
            }
            HtmlToken::CloseTag(tag) => {
                close_optional_html_lint_frames_before_close(stack, &tag.name, options);
                close_html_lint_frame(stack, &tag.name);
            }
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

fn close_optional_html_lint_frames_before_open(
    stack: &mut Vec<HtmlElementLintFrame>,
    next_name: &str,
    options: LintOptions,
) {
    if !options.parser.allow_html_optional_closing_tags {
        return;
    }

    while stack
        .last()
        .is_some_and(|frame| mixed_parser::optional_html_close_before_open(&frame.name, next_name))
    {
        stack.pop();
    }
}

fn close_optional_html_lint_frames_before_close(
    stack: &mut Vec<HtmlElementLintFrame>,
    close_name: &str,
    options: LintOptions,
) {
    if !options.parser.allow_html_optional_closing_tags {
        return;
    }

    while stack.last().is_some_and(|frame| {
        mixed_parser::optional_html_close_before_close(&frame.name, close_name)
    }) {
        stack.pop();
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
    stack: &[HtmlElementLintFrame],
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !options.rules.no_self_closing_html_tag || is_inside_foreign_html_content(tag, stack) {
        return;
    }

    diagnostics.push(Diagnostic::located_with_severity(
        format!("self-closing HTML tag `{}` is not valid HTML5", tag.raw),
        lexer::source_location(input, fragment_start + html_token_start),
        options.rule_severities.no_self_closing_html_tag,
    ));
}

fn is_inside_foreign_html_content(tag: &html::HtmlTag, stack: &[HtmlElementLintFrame]) -> bool {
    html_tag_is_foreign_content(&tag.name)
        || stack
            .iter()
            .any(|frame| html_tag_is_foreign_content(&frame.name))
}

fn html_tag_is_foreign_content(name: &str) -> bool {
    matches!(name.to_ascii_lowercase().as_str(), "svg" | "math")
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
    raw_name: String,
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

fn lint_unknown_html_attributes(
    input: &str,
    fragment_start: usize,
    html_token_start: usize,
    tag: &html::HtmlTag,
    stack: &[HtmlElementLintFrame],
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !options.rules.no_unknown_html_attribute
        || tag.raw.contains("<%")
        || is_inside_foreign_html_content(tag, stack)
        || is_custom_html_element(&tag.name)
    {
        return;
    }

    let Some(attribute_set) = known_html_attributes_for_tag(&tag.name) else {
        return;
    };

    for attribute in html_attributes(tag) {
        if let Some(candidate) = unknown_attribute_typo_candidate(&attribute.raw_name)
            && attribute_has_typo_shape(&attribute.raw_name)
            && attribute.raw_name != candidate
        {
            diagnostics.push(Diagnostic::located_with_severity(
                format!(
                    "unknown HTML attribute `{}`; did you mean `{candidate}`?",
                    attribute.raw_name
                ),
                lexer::source_location(input, fragment_start + html_token_start + attribute.offset),
                options.rule_severities.no_unknown_html_attribute,
            ));
            continue;
        }

        if is_known_html_attribute(&attribute.name, attribute_set)
            || is_framework_html_attribute(&attribute.name)
        {
            continue;
        }

        let message = if let Some(candidate) = unknown_attribute_typo_candidate(&attribute.raw_name)
        {
            format!(
                "unknown HTML attribute `{}`; did you mean `{candidate}`?",
                attribute.raw_name
            )
        } else {
            format!("unknown HTML attribute `{}`", attribute.raw_name)
        };

        diagnostics.push(Diagnostic::located_with_severity(
            message,
            lexer::source_location(input, fragment_start + html_token_start + attribute.offset),
            options.rule_severities.no_unknown_html_attribute,
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
            raw_name: raw[name_start..name_end].to_string(),
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

fn is_known_html_attribute(name: &str, tag_specific_attributes: &[&str]) -> bool {
    is_global_html_attribute(name)
        || is_data_html_attribute(name)
        || is_aria_html_attribute(name)
        || is_event_handler_html_attribute(name)
        || tag_specific_attributes.contains(&name)
}

fn known_html_attributes_for_tag(name: &str) -> Option<&'static [&'static str]> {
    match name.to_ascii_lowercase().as_str() {
        "a" => Some(&[
            "download",
            "href",
            "hreflang",
            "ping",
            "referrerpolicy",
            "rel",
            "target",
            "type",
        ]),
        "button" => Some(&[
            "command",
            "commandfor",
            "disabled",
            "form",
            "formaction",
            "formenctype",
            "formmethod",
            "formnovalidate",
            "formtarget",
            "name",
            "popovertarget",
            "popovertargetaction",
            "type",
            "value",
        ]),
        "input" => Some(&[
            "accept",
            "alt",
            "autocomplete",
            "capture",
            "checked",
            "dirname",
            "disabled",
            "form",
            "formaction",
            "formenctype",
            "formmethod",
            "formnovalidate",
            "formtarget",
            "height",
            "list",
            "max",
            "maxlength",
            "min",
            "minlength",
            "multiple",
            "name",
            "pattern",
            "placeholder",
            "readonly",
            "required",
            "size",
            "src",
            "step",
            "type",
            "value",
            "width",
        ]),
        _ => None,
    }
}

fn is_global_html_attribute(name: &str) -> bool {
    matches!(
        name,
        "accesskey"
            | "autocapitalize"
            | "autofocus"
            | "class"
            | "contenteditable"
            | "dir"
            | "draggable"
            | "enterkeyhint"
            | "hidden"
            | "id"
            | "inert"
            | "inputmode"
            | "is"
            | "itemid"
            | "itemprop"
            | "itemref"
            | "itemscope"
            | "itemtype"
            | "lang"
            | "nonce"
            | "part"
            | "popover"
            | "slot"
            | "spellcheck"
            | "style"
            | "tabindex"
            | "title"
            | "translate"
    )
}

fn is_data_html_attribute(name: &str) -> bool {
    name.starts_with("data-") && name.len() > "data-".len()
}

fn is_aria_html_attribute(name: &str) -> bool {
    name.starts_with("aria-") && name.len() > "aria-".len()
}

fn is_event_handler_html_attribute(name: &str) -> bool {
    name.starts_with("on") && name.len() > "on".len()
}

fn is_framework_html_attribute(raw_name: &str) -> bool {
    raw_name.starts_with("v-")
        || raw_name.starts_with(':')
        || raw_name.starts_with('@')
        || raw_name.starts_with("x-")
        || raw_name.starts_with("hx-")
        || raw_name.starts_with("ng-")
        || raw_name.starts_with("phx-")
        || raw_name.starts_with("turbo-")
}

fn is_custom_html_element(name: &str) -> bool {
    name.contains('-')
}

fn unknown_attribute_typo_candidate(raw_name: &str) -> Option<&'static str> {
    match normalized_html_attribute_typo_key(raw_name).as_str() {
        "arialabel" => Some("aria-label"),
        "autocomplete" => Some("autocomplete"),
        "formaction" => Some("formaction"),
        "formmethod" => Some("formmethod"),
        "formtarget" => Some("formtarget"),
        "maxlength" => Some("maxlength"),
        "minlength" => Some("minlength"),
        "readonly" => Some("readonly"),
        "tabindex" => Some("tabindex"),
        _ => None,
    }
}

fn attribute_has_typo_shape(raw_name: &str) -> bool {
    raw_name.contains('-') || raw_name.contains('_') || is_mixed_case(raw_name)
}

fn is_mixed_case(raw_name: &str) -> bool {
    raw_name.chars().any(|ch| ch.is_ascii_lowercase())
        && raw_name.chars().any(|ch| ch.is_ascii_uppercase())
}

fn normalized_html_attribute_typo_key(raw_name: &str) -> String {
    raw_name
        .chars()
        .filter(|ch| *ch != '-' && *ch != '_')
        .flat_map(char::to_lowercase)
        .collect()
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
