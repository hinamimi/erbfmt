pub(super) struct ParsedTag {
    pub(super) name: String,
    pub(super) attributes: Vec<String>,
    pub(super) self_closing: bool,
}

impl ParsedTag {
    pub(super) fn parse(raw: &str) -> Option<Self> {
        let body = raw.strip_prefix('<')?.strip_suffix('>')?.trim();

        if body.is_empty()
            || body.starts_with('/')
            || body.starts_with('!')
            || body.starts_with('?')
            || body.starts_with('%')
        {
            return None;
        }

        let self_closing = body.ends_with('/');
        let body = if self_closing {
            body.strip_suffix('/')?.trim_end()
        } else {
            body
        };

        let name_end = body
            .char_indices()
            .find_map(|(index, ch)| ch.is_whitespace().then_some(index))
            .unwrap_or(body.len());
        let name = body[..name_end].to_string();
        let attributes = split_attributes(body[name_end..].trim());

        Some(Self {
            name,
            attributes,
            self_closing,
        })
    }

    pub(super) fn closing_marker(&self) -> &'static str {
        if self.self_closing { "/>" } else { ">" }
    }

    pub(super) fn inline(&self) -> String {
        let attributes = self.normalized_attributes();

        if !attributes.is_empty() {
            let attributes = attributes.join(" ");

            return if self.self_closing {
                format!("<{} {attributes} />", self.name)
            } else {
                format!("<{} {attributes}>", self.name)
            };
        }

        if self.self_closing {
            format!("<{} />", self.name)
        } else {
            format!("<{}>", self.name)
        }
    }

    fn normalized_attributes(&self) -> Vec<String> {
        self.attributes
            .iter()
            .map(|attribute| normalize_attribute_quotes(attribute))
            .collect()
    }
}

pub(super) struct TagRenderContext<'a> {
    pub(super) indent: &'a str,
    pub(super) child_indent: &'a str,
    pub(super) line_ending: &'a str,
    pub(super) line_width: usize,
}

pub(super) fn render_tag(raw: &str, suffix: &str, context: TagRenderContext<'_>) -> Option<String> {
    let trimmed = raw.trim();
    let suffix = suffix.trim_end();
    if trimmed.is_empty() {
        return None;
    }

    let Some(tag) = ParsedTag::parse(trimmed) else {
        let is_multiline = trimmed.contains('\n');
        if !is_multiline && fits_on_line(context.indent, trimmed, context.line_width) {
            return Some(format!("{}{trimmed}{suffix}", context.indent));
        }

        return Some(format!("{}{trimmed}{suffix}", context.indent));
    };
    let inline = tag.inline();
    let is_multiline = trimmed.contains('\n');
    if !is_multiline && fits_on_line(context.indent, &inline, context.line_width) {
        return Some(format!("{}{inline}{suffix}", context.indent));
    }

    if tag.attributes.is_empty() {
        return Some(format!("{}{}{}", context.indent, tag.inline(), suffix));
    }

    let mut rendered = format!("{}<{}{}", context.indent, tag.name, context.line_ending);

    for attribute in tag.normalized_attributes() {
        rendered.push_str(context.child_indent);
        rendered.push_str(&attribute);
        rendered.push_str(context.line_ending);
    }

    rendered.push_str(context.indent);
    rendered.push_str(tag.closing_marker());
    rendered.push_str(suffix);
    Some(rendered)
}

fn fits_on_line(indent: &str, text: &str, line_width: usize) -> bool {
    indent.chars().count() + text.chars().count() <= line_width
}

fn split_attributes(input: &str) -> Vec<String> {
    let mut attributes = Vec::new();
    let mut start = None;
    let mut quote = None;
    let mut cursor = 0;

    while cursor < input.len() {
        if input[cursor..].starts_with("<%") {
            let Some(relative_end) = input[cursor + "<%".len()..].find("%>") else {
                break;
            };
            cursor += "<%".len() + relative_end + "%>".len();
            continue;
        }

        let ch = input[cursor..]
            .chars()
            .next()
            .expect("cursor is inside input");

        if start.is_none() && !ch.is_whitespace() {
            start = Some(cursor);
        }

        match quote {
            Some(active_quote) if ch == active_quote => quote = None,
            Some(_) => {}
            None if ch == '"' || ch == '\'' => quote = Some(ch),
            None if ch.is_whitespace() => {
                if let Some(attribute_start) = start.take() {
                    attributes.push(input[attribute_start..cursor].to_string());
                }
            }
            None => {}
        }

        cursor += ch.len_utf8();
    }

    if let Some(attribute_start) = start {
        attributes.push(input[attribute_start..].trim_end().to_string());
    }

    attributes
}

pub(super) fn normalize_tag(raw: &str) -> Option<String> {
    ParsedTag::parse(raw.trim()).map(|tag| tag.inline())
}

pub(super) fn normalize_close_tag(raw: &str) -> Option<String> {
    let name = raw.trim().strip_prefix("</")?.strip_suffix('>')?.trim();

    if name.is_empty()
        || name
            .chars()
            .any(|ch| ch.is_whitespace() || matches!(ch, '<' | '>' | '/' | '"' | '\''))
    {
        return None;
    }

    Some(format!("</{name}>"))
}

fn normalize_attribute_quotes(attribute: &str) -> String {
    let Some((name, value)) = attribute.split_once('=') else {
        return attribute.to_string();
    };
    let value = value.trim();

    if !value.starts_with('\'') || !value.ends_with('\'') {
        return attribute.to_string();
    }

    let inner = &value['\''.len_utf8()..value.len() - '\''.len_utf8()];
    if inner.contains('"') {
        return attribute.to_string();
    }

    format!("{}=\"{}\"", name.trim(), inner)
}

pub(super) fn attribute_name(attribute: &str) -> &str {
    attribute
        .split_once('=')
        .map_or(attribute, |(name, _)| name)
        .trim()
}

pub(super) fn attribute_value(attribute: &str) -> Option<&str> {
    let (_, value) = attribute.split_once('=')?;
    Some(value.trim().trim_matches(['"', '\'']))
}
