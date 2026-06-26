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
        if self.self_closing {
            format!("<{} />", self.name)
        } else {
            format!("<{}>", self.name)
        }
    }
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
