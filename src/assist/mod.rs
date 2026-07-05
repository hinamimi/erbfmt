use std::{collections::BTreeMap, ops::Range};

use crate::syntax::{
    erb::{self, Token},
    html::{self, HtmlToken},
};

pub const USE_DOUBLE_QUOTED_ATTRIBUTES: &str = "html.useDoubleQuotedAttributes";
pub const FIX_SELF_CLOSING_TAGS: &str = "html.fixSelfClosingTags";
pub const SIMPLIFY_BOOLEAN_ATTRIBUTES: &str = "html.simplifyBooleanAttributes";
pub const REMOVE_EMPTY_ERB_TAGS: &str = "erb.removeEmptyTags";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistOptions {
    pub enabled: bool,
    pub actions: BTreeMap<String, bool>,
}

impl AssistOptions {
    fn action_enabled(&self, id: &str) -> bool {
        self.enabled && self.actions.get(id).copied().unwrap_or(true)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistAction {
    pub id: &'static str,
    pub title: &'static str,
    pub range: TextRange,
    pub edit: TextEdit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    pub range: TextRange,
    pub replacement: String,
}

pub fn assist(input: &str, options: &AssistOptions) -> Vec<AssistAction> {
    let mut actions = Vec::new();

    if !options.enabled {
        return actions;
    }

    collect_html_assists(input, options, &mut actions);
    collect_erb_assists(input, options, &mut actions);
    actions.sort_by_key(|action| (action.range.start, action.range.end, action.id));
    actions
}

pub fn apply_actions(input: &str, actions: &[AssistAction]) -> String {
    let mut output = input.to_string();
    let mut last_start = input.len();

    for action in actions.iter().rev() {
        if action.edit.range.end > last_start {
            continue;
        }

        output.replace_range(
            action.edit.range.start..action.edit.range.end,
            &action.edit.replacement,
        );
        last_start = action.edit.range.start;
    }

    output
}

fn collect_html_assists(input: &str, options: &AssistOptions, actions: &mut Vec<AssistAction>) {
    let mut foreign_stack: Vec<String> = Vec::new();

    for spanned in html::tokenize_with_spans(input) {
        let is_self_closing = matches!(spanned.token, HtmlToken::SelfClosingTag(_));

        match &spanned.token {
            HtmlToken::OpenTag(tag) => {
                let in_foreign_content =
                    html_tag_is_foreign_content(&tag.name) || foreign_stack.last().is_some();

                if options.action_enabled(USE_DOUBLE_QUOTED_ATTRIBUTES) {
                    collect_double_quote_attribute_assists(spanned.span.start, tag, actions);
                }

                if options.action_enabled(SIMPLIFY_BOOLEAN_ATTRIBUTES) {
                    collect_simplify_boolean_attribute_assists(spanned.span.start, tag, actions);
                }

                if options.action_enabled(FIX_SELF_CLOSING_TAGS)
                    && is_self_closing
                    && !in_foreign_content
                    && let Some(replacement) = fixed_self_closing_tag(tag)
                {
                    actions.push(AssistAction {
                        id: FIX_SELF_CLOSING_TAGS,
                        title: "Fix self-closing HTML tag",
                        range: TextRange {
                            start: spanned.span.start,
                            end: spanned.span.end,
                        },
                        edit: TextEdit {
                            range: TextRange {
                                start: spanned.span.start,
                                end: spanned.span.end,
                            },
                            replacement,
                        },
                    });
                }

                if html_tag_is_foreign_content(&tag.name) || foreign_stack.last().is_some() {
                    foreign_stack.push(tag.name.to_ascii_lowercase());
                }
            }
            HtmlToken::VoidTag(tag) | HtmlToken::SelfClosingTag(tag) => {
                let in_foreign_content =
                    html_tag_is_foreign_content(&tag.name) || foreign_stack.last().is_some();

                if options.action_enabled(USE_DOUBLE_QUOTED_ATTRIBUTES) {
                    collect_double_quote_attribute_assists(spanned.span.start, tag, actions);
                }

                if options.action_enabled(SIMPLIFY_BOOLEAN_ATTRIBUTES) {
                    collect_simplify_boolean_attribute_assists(spanned.span.start, tag, actions);
                }

                if options.action_enabled(FIX_SELF_CLOSING_TAGS)
                    && is_self_closing
                    && !in_foreign_content
                    && let Some(replacement) = fixed_self_closing_tag(tag)
                {
                    actions.push(AssistAction {
                        id: FIX_SELF_CLOSING_TAGS,
                        title: "Fix self-closing HTML tag",
                        range: TextRange {
                            start: spanned.span.start,
                            end: spanned.span.end,
                        },
                        edit: TextEdit {
                            range: TextRange {
                                start: spanned.span.start,
                                end: spanned.span.end,
                            },
                            replacement,
                        },
                    });
                }
            }
            HtmlToken::CloseTag(tag) => {
                if let Some(index) = foreign_stack
                    .iter()
                    .rposition(|name| name.eq_ignore_ascii_case(&tag.name))
                {
                    foreign_stack.truncate(index);
                }
            }
            HtmlToken::Text(_) | HtmlToken::Comment(_) | HtmlToken::Doctype(_) => {}
        }
    }
}

fn collect_erb_assists(input: &str, options: &AssistOptions, actions: &mut Vec<AssistAction>) {
    if !options.action_enabled(REMOVE_EMPTY_ERB_TAGS) {
        return;
    }

    let Ok(tokens) = erb::tokenize_with_spans(input) else {
        return;
    };

    for spanned in tokens {
        let empty = match &spanned.token {
            Token::ErbCode(tag) | Token::ErbOutput(tag) => tag.code.trim().is_empty(),
            Token::Html(_)
            | Token::ErbComment(_)
            | Token::ErbBlockStart { .. }
            | Token::ErbBranch { .. }
            | Token::ErbBlockEnd(_) => false,
        };

        if empty {
            actions.push(AssistAction {
                id: REMOVE_EMPTY_ERB_TAGS,
                title: "Remove empty ERB tag",
                range: TextRange {
                    start: spanned.span.start,
                    end: spanned.span.end,
                },
                edit: TextEdit {
                    range: TextRange {
                        start: spanned.span.start,
                        end: spanned.span.end,
                    },
                    replacement: String::new(),
                },
            });
        }
    }
}

fn collect_double_quote_attribute_assists(
    tag_start: usize,
    tag: &html::HtmlTag,
    actions: &mut Vec<AssistAction>,
) {
    for attribute in html_attributes(tag) {
        let Some(value) = &attribute.value else {
            continue;
        };

        if value.quote == Some('"') || value.raw.contains('"') {
            continue;
        }

        let range = TextRange {
            start: tag_start + value.range.start,
            end: tag_start + value.range.end,
        };
        let replacement = format!("\"{}\"", &tag.raw[value.value_range()]);

        actions.push(AssistAction {
            id: USE_DOUBLE_QUOTED_ATTRIBUTES,
            title: "Use double quotes for HTML attribute value",
            range,
            edit: TextEdit { range, replacement },
        });
    }
}

fn collect_simplify_boolean_attribute_assists(
    tag_start: usize,
    tag: &html::HtmlTag,
    actions: &mut Vec<AssistAction>,
) {
    if tag.raw.contains("<%") {
        return;
    }

    for attribute in html_attributes(tag) {
        if !is_html_boolean_attribute(&attribute.name) {
            continue;
        }

        let Some(value) = &attribute.value else {
            continue;
        };

        if !value.raw.eq_ignore_ascii_case(&attribute.name) {
            continue;
        }

        let range = TextRange {
            start: tag_start + attribute.range.start,
            end: tag_start + attribute.range.end,
        };

        actions.push(AssistAction {
            id: SIMPLIFY_BOOLEAN_ATTRIBUTES,
            title: "Simplify redundant HTML boolean attribute",
            range,
            edit: TextEdit {
                range,
                replacement: tag.raw[attribute.name_range()].to_string(),
            },
        });
    }
}

fn fixed_self_closing_tag(tag: &html::HtmlTag) -> Option<String> {
    let body = tag.raw.strip_prefix('<')?.strip_suffix('>')?.trim();
    let open_body = body.strip_suffix('/')?.trim_end();
    let open = format!("<{open_body}>");

    if is_void_html_tag(&tag.name) {
        Some(open)
    } else {
        Some(format!("{open}</{}>", tag.name))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HtmlAttribute {
    name: String,
    range: TextRange,
    name_end: usize,
    value: Option<HtmlAttributeValue>,
}

impl HtmlAttribute {
    fn name_range(&self) -> Range<usize> {
        self.range.start..self.name_end
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HtmlAttributeValue {
    raw: String,
    quote: Option<char>,
    range: TextRange,
    value_start: usize,
    value_end: usize,
}

impl HtmlAttributeValue {
    fn value_range(&self) -> Range<usize> {
        self.value_start..self.value_end
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

        cursor = skip_html_attribute_spacing(raw, name_end);
        let value = if raw[cursor..].starts_with('=') {
            let (next_cursor, value) = read_html_attribute_value(raw, cursor + '='.len_utf8());
            cursor = next_cursor;
            value
        } else {
            None
        };

        attributes.push(HtmlAttribute {
            name: raw[name_start..name_end].to_ascii_lowercase(),
            range: TextRange {
                start: name_start,
                end: cursor,
            },
            name_end,
            value,
        });
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
        let quote_start = cursor;
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
                        range: TextRange {
                            start: quote_start,
                            end: cursor,
                        },
                        value_start,
                        value_end,
                    }),
                );
            }
        }

        return (
            cursor,
            Some(HtmlAttributeValue {
                raw: raw[value_start..cursor].to_string(),
                quote: Some(first),
                range: TextRange {
                    start: quote_start,
                    end: cursor,
                },
                value_start,
                value_end: cursor,
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
                range: TextRange {
                    start: value_start,
                    end: cursor,
                },
                value_start,
                value_end: cursor,
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

fn is_void_html_tag(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn html_tag_is_foreign_content(name: &str) -> bool {
    matches!(name.to_ascii_lowercase().as_str(), "svg" | "math")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled_options() -> AssistOptions {
        AssistOptions {
            enabled: true,
            actions: BTreeMap::new(),
        }
    }

    #[test]
    fn assists_safe_html_normalizations() {
        let input = "<button class='primary' checked=\"checked\"><br /><div /></button>\n";
        let actions = assist(input, &enabled_options());

        assert_eq!(actions.len(), 4);
        assert_eq!(
            apply_actions(input, &actions),
            "<button class=\"primary\" checked><br><div></div></button>\n"
        );
    }

    #[test]
    fn preserves_self_closing_tags_in_foreign_content() {
        let input = concat!(
            "<svg><path d=\"M0 0\" /><g><use href=\"#icon\" /></g></svg>\n",
            "<math><mi /></math>\n",
            "<div />\n",
        );
        let actions = assist(input, &enabled_options());

        assert_eq!(
            apply_actions(input, &actions),
            concat!(
                "<svg><path d=\"M0 0\" /><g><use href=\"#icon\" /></g></svg>\n",
                "<math><mi /></math>\n",
                "<div></div>\n",
            )
        );
    }

    #[test]
    fn assists_unquoted_erb_attribute_values() {
        let input = "<div class=<%= class_name %>></div>\n";
        let actions = assist(input, &enabled_options());

        assert_eq!(
            apply_actions(input, &actions),
            "<div class=\"<%= class_name %>\"></div>\n"
        );
    }

    #[test]
    fn skips_single_quoted_values_containing_double_quotes() {
        let input = r#"<div data-json='{"key":"value"}'></div>"#;
        let actions = assist(input, &enabled_options());

        assert!(actions.is_empty());
    }

    #[test]
    fn removes_empty_erb_tags() {
        let input = "<p>Before</p>\n<% %>\n<%= %>\n";
        let actions = assist(input, &enabled_options());

        assert_eq!(actions.len(), 2);
        assert_eq!(apply_actions(input, &actions), "<p>Before</p>\n\n\n");
    }
}
