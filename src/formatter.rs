use crate::mixed_parser::{Document, ErbBranch, Node};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatOptions {
    pub indent_html: bool,
    pub indent_style: IndentStyle,
    pub indent_width: usize,
    pub line_width: usize,
    pub line_ending: LineEnding,
    pub trailing_newline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IndentStyle {
    Space,
    Tab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LineEnding {
    Lf,
    Crlf,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent_html: true,
            indent_style: IndentStyle::Space,
            indent_width: 2,
            line_width: 80,
            line_ending: LineEnding::Lf,
            trailing_newline: true,
        }
    }
}

impl LineEnding {
    fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::Crlf => "\r\n",
        }
    }
}

#[allow(dead_code)]
pub fn format_document(document: &Document) -> String {
    format_document_with_options(document, FormatOptions::default())
}

pub fn format_document_with_options(document: &Document, options: FormatOptions) -> String {
    let mut formatter = Formatter::new(options);
    formatter.format_nodes(&document.children, 0);
    formatter.finish()
}

struct Formatter {
    options: FormatOptions,
    output: String,
}

impl Formatter {
    fn new(options: FormatOptions) -> Self {
        Self {
            options,
            output: String::new(),
        }
    }

    fn format_nodes(&mut self, nodes: &[Node], depth: usize) {
        for node in nodes {
            self.format_node(node, depth);
        }
    }

    fn format_node(&mut self, node: &Node, depth: usize) {
        match node {
            Node::HtmlText(text) => self.write_text(text, depth),
            Node::HtmlElement {
                open,
                close,
                children,
                ..
            } => self.write_html_element(open, close, children, depth),
            Node::HtmlSelfClosing { raw, .. } | Node::HtmlVoid { raw, .. } => {
                self.write_tag(raw, depth)
            }
            Node::HtmlComment(comment) | Node::HtmlDoctype(comment) => {
                self.write_indented_line(depth, comment);
            }
            Node::ErbCode(code) => {
                self.write_indented_line(depth, &format!("<% {} %>", code.trim()))
            }
            Node::ErbOutput(code) => {
                self.write_indented_line(depth, &format!("<%= {} %>", code.trim()));
            }
            Node::ErbBlock {
                code,
                output,
                children,
                branches,
                ..
            } => {
                self.write_indented_line(depth, &format_erb_block_open(*output, code));
                self.format_nodes(children, depth + 1);
                self.format_erb_branches(branches, depth);
                self.write_indented_line(depth, "<% end %>");
            }
        }
    }

    fn format_erb_branches(&mut self, branches: &[ErbBranch], depth: usize) {
        for branch in branches {
            self.write_indented_line(depth, &format!("<% {} %>", branch.code));
            self.format_nodes(&branch.children, depth + 1);
        }
    }

    fn write_text(&mut self, text: &str, depth: usize) {
        for line in text.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                self.write_indented_line(depth, trimmed);
            }
        }
    }

    fn write_html_element(&mut self, open: &str, close: &str, children: &[Node], depth: usize) {
        if can_render_inline(children) {
            let content = render_inline_nodes(children);
            self.write_indented_line(depth, &format!("{open}{content}{close}"));
        } else {
            self.write_tag(open, depth);
            self.format_nodes(children, self.html_child_depth(depth));
            self.write_indented_line(depth, close);
        }
    }

    fn write_tag(&mut self, raw: &str, depth: usize) {
        let trimmed = raw.trim();
        let line_width = self.options.line_width;

        if self.indent(depth).chars().count() + trimmed.chars().count() <= line_width {
            self.write_indented_line(depth, trimmed);
            return;
        }

        let Some(tag) = ParsedTag::parse(trimmed) else {
            self.write_indented_line(depth, trimmed);
            return;
        };

        if tag.attributes.is_empty() {
            self.write_indented_line(depth, trimmed);
            return;
        }

        self.write_indented_line(depth, &format!("<{}", tag.name));

        let last_index = tag.attributes.len() - 1;
        for (index, attribute) in tag.attributes.iter().enumerate() {
            if index == last_index {
                self.write_indented_line(
                    depth + 1,
                    &format!("{}{}", attribute, tag.closing_suffix()),
                );
            } else {
                self.write_indented_line(depth + 1, attribute);
            }
        }
    }

    fn html_child_depth(&self, depth: usize) -> usize {
        depth + usize::from(self.options.indent_html)
    }

    fn write_indented_line(&mut self, depth: usize, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }

        self.output.push_str(&self.indent(depth));
        self.output.push_str(trimmed);
        self.output.push('\n');
    }

    fn indent(&self, depth: usize) -> String {
        match self.options.indent_style {
            IndentStyle::Space => " ".repeat(self.options.indent_width * depth),
            IndentStyle::Tab => "\t".repeat(depth),
        }
    }

    fn finish(mut self) -> String {
        if !self.options.trailing_newline {
            self.output = self.output.trim_end_matches('\n').to_string();
        }

        match self.options.line_ending {
            LineEnding::Lf => self.output,
            LineEnding::Crlf => self.output.replace('\n', self.options.line_ending.as_str()),
        }
    }
}

struct ParsedTag {
    name: String,
    attributes: Vec<String>,
    self_closing: bool,
}

impl ParsedTag {
    fn parse(raw: &str) -> Option<Self> {
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

    fn closing_suffix(&self) -> &'static str {
        if self.self_closing { " />" } else { ">" }
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

fn can_render_inline(nodes: &[Node]) -> bool {
    nodes.iter().all(is_inline_node)
}

fn is_inline_node(node: &Node) -> bool {
    match node {
        Node::HtmlText(text) => !text.contains('\n'),
        Node::HtmlSelfClosing { .. }
        | Node::HtmlVoid { .. }
        | Node::ErbCode(_)
        | Node::ErbOutput(_) => true,
        Node::HtmlElement { .. }
        | Node::HtmlComment(_)
        | Node::HtmlDoctype(_)
        | Node::ErbBlock { .. } => false,
    }
}

fn render_inline_nodes(nodes: &[Node]) -> String {
    nodes
        .iter()
        .map(render_inline_node)
        .collect::<String>()
        .trim()
        .to_string()
}

fn render_inline_node(node: &Node) -> String {
    match node {
        Node::HtmlText(text) => text.clone(),
        Node::HtmlSelfClosing { raw, .. } | Node::HtmlVoid { raw, .. } => raw.clone(),
        Node::ErbCode(code) => format!("<% {} %>", code.trim()),
        Node::ErbOutput(code) => format!("<%= {} %>", code.trim()),
        Node::HtmlElement { .. }
        | Node::HtmlComment(_)
        | Node::HtmlDoctype(_)
        | Node::ErbBlock { .. } => unreachable!("node cannot render inline"),
    }
}

fn format_erb_block_open(output: bool, code: &str) -> String {
    if output {
        format!("<%= {} %>", code.trim())
    } else {
        format!("<% {} %>", code.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::tokenize, mixed_parser::parse};

    fn format(input: &str) -> String {
        let tokens = tokenize(input).unwrap();
        let document = parse(&tokens).unwrap();

        format_document(&document)
    }

    fn format_without_html_indent(input: &str) -> String {
        let tokens = tokenize(input).unwrap();
        let document = parse(&tokens).unwrap();

        format_document_with_options(
            &document,
            FormatOptions {
                indent_html: false,
                ..FormatOptions::default()
            },
        )
    }

    fn format_with_options(input: &str, options: FormatOptions) -> String {
        let tokens = tokenize(input).unwrap();
        let document = parse(&tokens).unwrap();

        format_document_with_options(&document, options)
    }

    #[test]
    fn formats_plain_html_lines() {
        assert_eq!(
            format("<div>\n<p>Hello</p>\n</div>\n"),
            "<div>\n  <p>Hello</p>\n</div>\n"
        );
    }

    #[test]
    fn preserves_inline_erb_output() {
        assert_eq!(
            format("<h1><%= page_title %></h1>\n<p>Hello, <%= user.name %></p>\n"),
            "<h1><%= page_title %></h1>\n<p>Hello, <%= user.name %></p>\n"
        );
    }

    #[test]
    fn indents_erb_block_children() {
        assert_eq!(
            format("<% if user %>\n<p>Hello</p>\n<% end %>\n"),
            "<% if user %>\n  <p>Hello</p>\n<% end %>\n"
        );
    }

    #[test]
    fn indents_nested_erb_blocks() {
        assert_eq!(
            format(
                "<% if user %>\n<ul>\n<% Objects.map do |obj| %>\n<li><%= obj.name %></li>\n<% end %>\n</ul>\n<% end %>\n"
            ),
            "<% if user %>\n  <ul>\n    <% Objects.map do |obj| %>\n      <li><%= obj.name %></li>\n    <% end %>\n  </ul>\n<% end %>\n"
        );
    }

    #[test]
    fn can_disable_html_indentation() {
        assert_eq!(
            format_without_html_indent(
                "<% if user %>\n<ul>\n<% Objects.map do |obj| %>\n<li><%= obj.name %></li>\n<% end %>\n</ul>\n<% end %>\n"
            ),
            "<% if user %>\n  <ul>\n  <% Objects.map do |obj| %>\n    <li><%= obj.name %></li>\n  <% end %>\n  </ul>\n<% end %>\n"
        );
    }

    #[test]
    fn can_configure_indent_width() {
        assert_eq!(
            format_with_options(
                "<div>\n<p>Hello</p>\n</div>\n",
                FormatOptions {
                    indent_width: 4,
                    ..FormatOptions::default()
                }
            ),
            "<div>\n    <p>Hello</p>\n</div>\n"
        );
    }

    #[test]
    fn can_configure_tab_indentation() {
        assert_eq!(
            format_with_options(
                "<div>\n<p>Hello</p>\n</div>\n",
                FormatOptions {
                    indent_style: IndentStyle::Tab,
                    ..FormatOptions::default()
                }
            ),
            "<div>\n\t<p>Hello</p>\n</div>\n"
        );
    }

    #[test]
    fn can_configure_line_ending_and_trailing_newline() {
        assert_eq!(
            format_with_options(
                "<div>\n<p>Hello</p>\n</div>\n",
                FormatOptions {
                    line_ending: LineEnding::Crlf,
                    trailing_newline: false,
                    ..FormatOptions::default()
                }
            ),
            "<div>\r\n  <p>Hello</p>\r\n</div>"
        );
    }

    #[test]
    fn wraps_long_html_opening_tags_by_attribute() {
        assert_eq!(
            format_with_options(
                r#"<article class="card" data-user-id="<%= user.id %>" aria-label="Current user profile"><p>Hello</p></article>"#,
                FormatOptions {
                    line_width: 48,
                    ..FormatOptions::default()
                }
            ),
            "<article\n  class=\"card\"\n  data-user-id=\"<%= user.id %>\"\n  aria-label=\"Current user profile\">\n  <p>Hello</p>\n</article>\n"
        );
    }

    #[test]
    fn wraps_long_void_tags_by_attribute() {
        assert_eq!(
            format_with_options(
                r#"<img src="<%= avatar_url %>" alt="<%= user.name %>" data-controller="avatar-preview">"#,
                FormatOptions {
                    line_width: 48,
                    ..FormatOptions::default()
                }
            ),
            "<img\n  src=\"<%= avatar_url %>\"\n  alt=\"<%= user.name %>\"\n  data-controller=\"avatar-preview\">\n"
        );
    }

    #[test]
    fn formats_if_elsif_else_branches() {
        assert_eq!(
            format(
                "<% if admin? %>\n<p>Admin</p>\n<% elsif user? %>\n<p>User</p>\n<% else %>\n<p>Guest</p>\n<% end %>\n"
            ),
            "<% if admin? %>\n  <p>Admin</p>\n<% elsif user? %>\n  <p>User</p>\n<% else %>\n  <p>Guest</p>\n<% end %>\n"
        );
    }

    #[test]
    fn formats_case_when_branches() {
        assert_eq!(
            format(
                "<% case role %>\n<% when \"admin\" %>\n<p>Admin</p>\n<% when \"user\" %>\n<p>User</p>\n<% end %>\n"
            ),
            "<% case role %>\n<% when \"admin\" %>\n  <p>Admin</p>\n<% when \"user\" %>\n  <p>User</p>\n<% end %>\n"
        );
    }

    #[test]
    fn formats_output_erb_do_blocks() {
        assert_eq!(
            format(
                "<%= form_with model: user do |form| %>\n<div>\n<%= form.text_field :name %>\n</div>\n<% end %>\n"
            ),
            "<%= form_with model: user do |form| %>\n  <div>\n    <%= form.text_field :name %>\n  </div>\n<% end %>\n"
        );
    }

    #[test]
    fn formats_begin_rescue_ensure_branches() {
        assert_eq!(
            format(
                "<% begin %>\n<p>Saving</p>\n<% rescue => error %>\n<p>Failed</p>\n<% ensure %>\n<p>Done</p>\n<% end %>\n"
            ),
            "<% begin %>\n  <p>Saving</p>\n<% rescue => error %>\n  <p>Failed</p>\n<% ensure %>\n  <p>Done</p>\n<% end %>\n"
        );
    }

    #[test]
    fn snapshots_default_html_indentation() {
        insta::assert_snapshot!(
            "default_html_indentation",
            format(
                "<div>\n<h1><%= page_title %></h1>\n<% if user %>\n<p>Hello, <%= user.name %></p>\n<ul>\n<% Objects.map do |obj| %>\n<li><%= obj.name %></li>\n<% end %>\n</ul>\n<% end %>\n</div>\n"
            )
        );
    }

    #[test]
    fn snapshots_without_html_indentation() {
        insta::assert_snapshot!(
            "without_html_indentation",
            format_without_html_indent(
                "<div>\n<h1><%= page_title %></h1>\n<% if user %>\n<p>Hello, <%= user.name %></p>\n<ul>\n<% Objects.map do |obj| %>\n<li><%= obj.name %></li>\n<% end %>\n</ul>\n<% end %>\n</div>\n"
            )
        );
    }

    #[test]
    fn snapshots_branch_formatting() {
        insta::assert_snapshot!(
            "branch_formatting",
            format(
                "<% if admin? %>\n<p>Admin</p>\n<% elsif user? %>\n<p>User</p>\n<% else %>\n<p>Guest</p>\n<% end %>\n<% case role %>\n<% when \"admin\" %>\n<p>Admin tools</p>\n<% when \"user\" %>\n<p>User dashboard</p>\n<% end %>\n"
            )
        );
    }

    #[test]
    fn snapshots_stability_fixture() {
        insta::assert_snapshot!("stability_fixture", format(stability_fixture()));
    }

    #[test]
    fn snapshots_stability_fixture_without_html_indentation() {
        insta::assert_snapshot!(
            "stability_fixture_without_html_indentation",
            format_without_html_indent(stability_fixture())
        );
    }

    #[test]
    fn snapshots_formatter_audit_fixture() {
        insta::assert_snapshot!("formatter_audit_fixture", format(formatter_audit_fixture()));
    }

    fn stability_fixture() -> &'static str {
        "<!DOCTYPE html>\n<div class=\"page <%= page_class %>\">\n<!-- profile card -->\n<img src=\"<%= avatar_url %>\" alt=\"<%= user.name %>\">\n<input type=\"checkbox\" checked=\"<%= checked %>\">\n<% if user %>\n<section>\n<a href=\"/users/<%= user.id %>\"><%= user.name %></a>\n<br>\n<% unless notifications.empty? %>\n<ul>\n<% notifications.each do |notification| %>\n<li><%= notification.title %></li>\n<% end %>\n</ul>\n<% end %>\n</section>\n<% else %>\n<p>Please sign in</p>\n<% end %>\n</div>\n"
    }

    fn formatter_audit_fixture() -> &'static str {
        include_str!("../samples/formatter-audit.html.erb")
    }
}
