use crate::mixed_parser::{Document, ErbBranch, Node};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatOptions {
    pub indent_html: bool,
    pub indent_style: IndentStyle,
    pub indent_width: usize,
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
                self.write_indented_line(depth, raw);
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
                children,
                branches,
                ..
            } => {
                self.write_indented_line(depth, &format!("<% {code} %>"));
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
            self.write_indented_line(depth, open);
            self.format_nodes(children, self.html_child_depth(depth));
            self.write_indented_line(depth, close);
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

    fn stability_fixture() -> &'static str {
        "<!DOCTYPE html>\n<div class=\"page <%= page_class %>\">\n<!-- profile card -->\n<img src=\"<%= avatar_url %>\" alt=\"<%= user.name %>\">\n<input type=\"checkbox\" checked=\"<%= checked %>\">\n<% if user %>\n<section>\n<a href=\"/users/<%= user.id %>\"><%= user.name %></a>\n<br>\n<% unless notifications.empty? %>\n<ul>\n<% notifications.each do |notification| %>\n<li><%= notification.title %></li>\n<% end %>\n</ul>\n<% end %>\n</section>\n<% else %>\n<p>Please sign in</p>\n<% end %>\n</div>\n"
    }
}
