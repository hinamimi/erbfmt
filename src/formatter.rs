use crate::{
    html::{self, HtmlToken},
    parser::{Document, Node},
};

const INDENT: &str = "  ";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatOptions {
    pub indent_html: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self { indent_html: true }
    }
}

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
    line: String,
    html_depth: usize,
}

impl Formatter {
    fn new(options: FormatOptions) -> Self {
        Self {
            options,
            output: String::new(),
            line: String::new(),
            html_depth: 0,
        }
    }

    fn format_nodes(&mut self, nodes: &[Node], depth: usize) {
        for node in nodes {
            self.format_node(node, depth);
        }
    }

    fn format_node(&mut self, node: &Node, depth: usize) {
        match node {
            Node::Html(html) => self.write_html(html, depth),
            Node::ErbCode(code) => self.write_inline_erb("<%", code, "%>", depth),
            Node::ErbOutput(code) => self.write_inline_erb("<%=", code, "%>", depth),
            Node::ErbBlock { code, children, .. } => {
                self.flush_line();
                self.write_indented_line(depth, &format!("<% {code} %>"));
                self.format_nodes(children, depth + 1);
                self.flush_line();
                self.write_indented_line(depth, "<% end %>");
            }
        }
    }

    fn write_html(&mut self, html: &str, depth: usize) {
        for segment in html.split_inclusive('\n') {
            let Some(line) = segment.strip_suffix('\n') else {
                self.write_inline_text(segment, depth);
                continue;
            };

            if !line.is_empty() || !self.line.is_empty() {
                self.write_inline_text(line, depth);
                self.flush_line();
            }
        }
    }

    fn write_inline_erb(&mut self, open: &str, code: &str, close: &str, depth: usize) {
        self.write_inline_text(&format!("{open} {} {close}", code.trim()), depth);
    }

    fn write_inline_text(&mut self, text: &str, depth: usize) {
        if text.trim().is_empty() {
            return;
        }

        if self.line.is_empty() {
            self.line
                .push_str(&INDENT.repeat(self.line_depth(depth, text)));
            self.line.push_str(text.trim_start());
        } else {
            self.line.push_str(text);
        }
    }

    fn write_indented_line(&mut self, depth: usize, line: &str) {
        self.output
            .push_str(&INDENT.repeat(self.line_depth(depth, line)));
        self.output.push_str(line);
        self.output.push('\n');
        self.apply_html_depth_delta(line);
    }

    fn flush_line(&mut self) {
        let line = self.line.trim_end().to_string();
        if line.is_empty() {
            self.line.clear();
            return;
        }

        self.output.push_str(&line);
        self.output.push('\n');
        self.line.clear();
        self.apply_html_depth_delta(&line);
    }

    fn line_depth(&self, erb_depth: usize, text: &str) -> usize {
        if !self.options.indent_html {
            return erb_depth;
        }

        erb_depth + self.html_depth.saturating_sub(leading_close_count(text))
    }

    fn apply_html_depth_delta(&mut self, line: &str) {
        if !self.options.indent_html {
            return;
        }

        let (opens, closes) = html_tag_delta(line);
        self.html_depth = self.html_depth.saturating_add(opens).saturating_sub(closes);
    }

    fn finish(mut self) -> String {
        self.flush_line();
        self.output
    }
}

fn leading_close_count(text: &str) -> usize {
    for token in html::tokenize(text) {
        match token {
            HtmlToken::Text(text) if text.trim().is_empty() => {}
            HtmlToken::CloseTag(_) => return 1,
            _ => return 0,
        }
    }

    0
}

fn html_tag_delta(line: &str) -> (usize, usize) {
    let mut opens = 0;
    let mut closes = 0;

    for token in html::tokenize(line) {
        match token {
            HtmlToken::OpenTag(_) => opens += 1,
            HtmlToken::CloseTag(_) => closes += 1,
            HtmlToken::Text(_)
            | HtmlToken::SelfClosingTag(_)
            | HtmlToken::VoidTag(_)
            | HtmlToken::Comment(_)
            | HtmlToken::Doctype(_) => {}
        }
    }

    (opens, closes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::tokenize, parser::parse};

    fn format(input: &str) -> String {
        let tokens = tokenize(input).unwrap();
        let document = parse(&tokens).unwrap();

        format_document(&document)
    }

    fn format_without_html_indent(input: &str) -> String {
        let tokens = tokenize(input).unwrap();
        let document = parse(&tokens).unwrap();

        format_document_with_options(&document, FormatOptions { indent_html: false })
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
}
