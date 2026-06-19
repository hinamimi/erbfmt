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
        let mut index = 0;

        while index < nodes.len() {
            if is_inline_node(&nodes[index]) {
                let start = index;

                while index < nodes.len() && is_inline_node(&nodes[index]) {
                    index += 1;
                }

                if index - start == 1 {
                    self.format_node(&nodes[start], depth);
                } else {
                    self.write_inline_nodes(&nodes[start..index], depth);
                }
            } else {
                self.format_node(&nodes[index], depth);
                index += 1;
            }
        }
    }

    fn format_node(&mut self, node: &Node, depth: usize) {
        match node {
            Node::HtmlText(text) => self.write_text(text, depth),
            Node::HtmlElement {
                name,
                open,
                close,
                children,
                ..
            } => self.write_html_element(name, open, close, children, depth),
            Node::HtmlSelfClosing { raw, .. } | Node::HtmlVoid { raw, .. } => {
                self.write_tag(raw, depth)
            }
            Node::HtmlComment(comment) | Node::HtmlDoctype(comment) => {
                self.write_indented_line(depth, comment);
            }
            Node::ErbCode(code) => self.write_erb_tag(depth, ErbTagMarker::Code, code),
            Node::ErbOutput(code) => self.write_erb_tag(depth, ErbTagMarker::Output, code),
            Node::ErbBlock {
                code,
                output,
                children,
                branches,
                ..
            } => {
                self.write_erb_tag(depth, ErbTagMarker::from_output(*output), code);
                self.format_nodes(children, depth + 1);
                self.format_erb_branches(branches, depth);
                self.write_indented_line(depth, "<% end %>");
            }
        }
    }

    fn format_erb_branches(&mut self, branches: &[ErbBranch], depth: usize) {
        for branch in branches {
            self.write_erb_tag(depth, ErbTagMarker::Code, &branch.code);
            self.format_nodes(&branch.children, depth + 1);
        }
    }

    fn write_text(&mut self, text: &str, depth: usize) {
        let mut pending_line_breaks = 0;

        for segment in text.split_inclusive('\n') {
            let has_line_break = segment.ends_with('\n');
            let line = segment.strip_suffix('\n').unwrap_or(segment);
            let trimmed = line.trim();

            if !trimmed.is_empty() {
                if pending_line_breaks >= 2 {
                    self.write_blank_line();
                }

                self.write_indented_line(depth, trimmed);
                pending_line_breaks = usize::from(has_line_break);
            } else if has_line_break {
                pending_line_breaks += 1;
            } else {
                // Whitespace after a newline is indentation, not another blank line.
            }
        }

        if pending_line_breaks >= 2 {
            self.write_blank_line();
        }
    }

    fn write_html_element(
        &mut self,
        name: &str,
        open: &str,
        close: &str,
        children: &[Node],
        depth: usize,
    ) {
        if is_format_sensitive_html_tag(name) {
            self.write_preserved_block(
                depth,
                &render_preserved_html_element(open, close, children),
            );
            return;
        }

        if can_render_inline(children) && self.can_keep_html_element_inline(open, depth) {
            let content = render_inline_nodes(children);
            self.write_indented_line(depth, &format!("{open}{content}{close}"));
        } else {
            self.write_tag(open, depth);
            self.format_nodes(children, self.html_child_depth(depth));
            self.write_indented_line(depth, close);
        }
    }

    fn write_inline_nodes(&mut self, nodes: &[Node], depth: usize) {
        let inline = render_inline_nodes(nodes);

        if inline.is_empty() {
            return;
        }

        self.write_indented_line(depth, &inline);
    }

    fn can_keep_html_element_inline(&self, open: &str, depth: usize) -> bool {
        let trimmed = open.trim();

        !trimmed.contains('\n') && self.fits_on_line(depth, trimmed)
    }

    fn write_tag(&mut self, raw: &str, depth: usize) {
        let trimmed = raw.trim();
        let is_multiline = trimmed.contains('\n');

        if !is_multiline && self.fits_on_line(depth, trimmed) {
            self.write_indented_line(depth, trimmed);
            return;
        }

        let Some(tag) = ParsedTag::parse(trimmed) else {
            self.write_indented_line(depth, trimmed);
            return;
        };

        if tag.attributes.is_empty() {
            self.write_indented_line(depth, &tag.inline());
            return;
        }

        self.write_indented_line(depth, &format!("<{}", tag.name));

        for attribute in &tag.attributes {
            self.write_indented_line(depth + 1, attribute);
        }

        self.write_indented_line(depth, tag.closing_marker());
    }

    fn write_erb_tag(&mut self, depth: usize, marker: ErbTagMarker, code: &str) {
        let code = code.trim();
        let inline = format_erb_tag_inline(marker, code);

        if !code.contains('\n') && self.fits_on_line(depth, &inline) {
            self.write_indented_line(depth, &inline);
            return;
        }

        self.write_indented_line(depth, marker.as_str());

        for line in normalized_erb_code_lines(code) {
            self.write_indented_code_line(depth + 1, &line);
        }

        self.write_indented_line(depth, "%>");
    }

    fn html_child_depth(&self, depth: usize) -> usize {
        depth + usize::from(self.options.indent_html)
    }

    fn fits_on_line(&self, depth: usize, text: &str) -> bool {
        self.indent(depth).chars().count() + text.chars().count() <= self.options.line_width
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

    fn write_preserved_block(&mut self, depth: usize, block: &str) {
        if block.is_empty() {
            return;
        }

        self.output.push_str(&self.indent(depth));
        self.output.push_str(block);

        if !block.ends_with('\n') {
            self.output.push('\n');
        }
    }

    fn write_blank_line(&mut self) {
        if self.output.is_empty() || self.output.ends_with("\n\n") {
            return;
        }

        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }

        self.output.push('\n');
    }

    fn write_indented_code_line(&mut self, depth: usize, line: &str) {
        let trimmed = line.trim_end();
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErbTagMarker {
    Code,
    Output,
}

impl ErbTagMarker {
    fn from_output(output: bool) -> Self {
        if output { Self::Output } else { Self::Code }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Code => "<%",
            Self::Output => "<%=",
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

    fn closing_marker(&self) -> &'static str {
        if self.self_closing { "/>" } else { ">" }
    }

    fn inline(&self) -> String {
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
        Node::ErbCode(code) => format_erb_tag_inline(ErbTagMarker::Code, code.trim()),
        Node::ErbOutput(code) => format_erb_tag_inline(ErbTagMarker::Output, code.trim()),
        Node::HtmlElement { .. }
        | Node::HtmlComment(_)
        | Node::HtmlDoctype(_)
        | Node::ErbBlock { .. } => unreachable!("node cannot render inline"),
    }
}

fn render_preserved_html_element(open: &str, close: &str, children: &[Node]) -> String {
    format!("{open}{}{close}", render_preserved_nodes(children))
}

fn render_preserved_nodes(nodes: &[Node]) -> String {
    nodes.iter().map(render_preserved_node).collect()
}

fn render_preserved_node(node: &Node) -> String {
    match node {
        Node::HtmlText(text) => text.clone(),
        Node::HtmlElement {
            open,
            close,
            children,
            ..
        } => render_preserved_html_element(open, close, children),
        Node::HtmlSelfClosing { raw, .. } | Node::HtmlVoid { raw, .. } => raw.clone(),
        Node::HtmlComment(comment) | Node::HtmlDoctype(comment) => comment.clone(),
        Node::ErbCode(code) => format_erb_tag_inline(ErbTagMarker::Code, code.trim()),
        Node::ErbOutput(code) => format_erb_tag_inline(ErbTagMarker::Output, code.trim()),
        Node::ErbBlock {
            code,
            output,
            children,
            branches,
            ..
        } => {
            let mut rendered = format_erb_tag_inline(ErbTagMarker::from_output(*output), code);
            rendered.push_str(&render_preserved_nodes(children));

            for branch in branches {
                rendered.push_str(&format_erb_tag_inline(ErbTagMarker::Code, &branch.code));
                rendered.push_str(&render_preserved_nodes(&branch.children));
            }

            rendered.push_str("<% end %>");
            rendered
        }
    }
}

fn is_format_sensitive_html_tag(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "pre" | "textarea" | "script" | "style" | "xmp" | "listing"
    )
}

fn format_erb_tag_inline(marker: ErbTagMarker, code: &str) -> String {
    if code.is_empty() {
        return format!("{} %>", marker.as_str());
    }

    format!("{} {} %>", marker.as_str(), code.trim())
}

fn normalized_erb_code_lines(code: &str) -> Vec<String> {
    let lines = trim_blank_edges(code.lines().collect());
    let common_indent = common_erb_code_indent(&lines);

    lines
        .into_iter()
        .map(|line| {
            strip_leading_whitespace(line, common_indent)
                .trim_end()
                .to_string()
        })
        .collect()
}

fn common_erb_code_indent(lines: &[&str]) -> usize {
    let non_empty_lines = lines.iter().copied().filter(|line| !line.trim().is_empty());

    if lines
        .first()
        .is_some_and(|line| leading_whitespace_count(line) == 0)
    {
        let skipped_first = lines
            .iter()
            .copied()
            .skip(1)
            .filter(|line| !line.trim().is_empty())
            .map(leading_whitespace_count)
            .min();

        if let Some(indent) = skipped_first {
            return indent;
        }
    }

    non_empty_lines
        .map(leading_whitespace_count)
        .min()
        .unwrap_or(0)
}

fn trim_blank_edges(mut lines: Vec<&str>) -> Vec<&str> {
    while lines.first().is_some_and(|line| line.trim().is_empty()) {
        lines.remove(0);
    }

    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }

    lines
}

fn leading_whitespace_count(line: &str) -> usize {
    line.chars().take_while(|ch| ch.is_whitespace()).count()
}

fn strip_leading_whitespace(line: &str, count: usize) -> &str {
    if count == 0 {
        return line;
    }

    for (stripped, (index, ch)) in line.char_indices().enumerate() {
        if stripped == count || !ch.is_whitespace() {
            return &line[index..];
        }
    }

    ""
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
    fn preserves_single_intentional_blank_lines() {
        assert_eq!(
            format("<section>\n<h1>Title</h1>\n\n<p>Body</p>\n</section>\n"),
            "<section>\n  <h1>Title</h1>\n\n  <p>Body</p>\n</section>\n"
        );
    }

    #[test]
    fn collapses_multiple_blank_lines_to_one() {
        assert_eq!(
            format("<section>\n<h1>Title</h1>\n\n\n<p>Body</p>\n</section>\n"),
            "<section>\n  <h1>Title</h1>\n\n  <p>Body</p>\n</section>\n"
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
    fn preserves_adjacent_erb_outputs_on_one_line() {
        assert_eq!(
            format("<%= form.radio_button :status, :draft %><%= form.label :status_draft %>\n"),
            "<%= form.radio_button :status, :draft %><%= form.label :status_draft %>\n"
        );
    }

    #[test]
    fn preserves_adjacent_erb_outputs_inside_blocks() {
        assert_eq!(
            format(
                "<% if form %>\n<%= form.radio_button :status, :draft %><%= form.label :status_draft %>\n<% end %>\n"
            ),
            "<% if form %>\n  <%= form.radio_button :status, :draft %><%= form.label :status_draft %>\n<% end %>\n"
        );
    }

    #[test]
    fn preserves_preformatted_html_content() {
        assert_eq!(
            format("<div>\n<pre>\n  line one\n    line two\n</pre>\n</div>\n"),
            "<div>\n  <pre>\n  line one\n    line two\n</pre>\n</div>\n"
        );
    }

    #[test]
    fn preserves_inline_preformatted_html_content() {
        assert_eq!(
            format("<pre>  line one\n    line two</pre>\n"),
            "<pre>  line one\n    line two</pre>\n"
        );
    }

    #[test]
    fn preserves_textarea_content() {
        assert_eq!(
            format("<form>\n<textarea>\n  keep me\n</textarea>\n</form>\n"),
            "<form>\n  <textarea>\n  keep me\n</textarea>\n</form>\n"
        );
    }

    #[test]
    fn preserves_script_and_style_content() {
        assert_eq!(
            format(
                "<script>\n  console.log(\"hello\");\n</script>\n<style>\n  body { color: red; }\n</style>\n"
            ),
            "<script>\n  console.log(\"hello\");\n</script>\n<style>\n  body { color: red; }\n</style>\n"
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
            "<article\n  class=\"card\"\n  data-user-id=\"<%= user.id %>\"\n  aria-label=\"Current user profile\"\n>\n  <p>Hello</p>\n</article>\n"
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
            "<img\n  src=\"<%= avatar_url %>\"\n  alt=\"<%= user.name %>\"\n  data-controller=\"avatar-preview\"\n>\n"
        );
    }

    #[test]
    fn wraps_long_self_closing_tags_with_marker_on_own_line() {
        assert_eq!(
            format_with_options(
                r#"<custom-input name="profile[display_name]" value="<%= user.display_name %>" data-controller="autosave" />"#,
                FormatOptions {
                    line_width: 48,
                    ..FormatOptions::default()
                }
            ),
            "<custom-input\n  name=\"profile[display_name]\"\n  value=\"<%= user.display_name %>\"\n  data-controller=\"autosave\"\n/>\n"
        );
    }

    #[test]
    fn normalizes_existing_multiline_html_tags() {
        assert_eq!(
            format("<div\nclass=\"card\"\ndata-controller=\"profile\"\n>\n<p>Hello</p>\n</div>\n"),
            "<div\n  class=\"card\"\n  data-controller=\"profile\"\n>\n  <p>Hello</p>\n</div>\n"
        );
    }

    #[test]
    fn normalizes_multiline_html_tags_with_erb_attributes() {
        assert_eq!(
            format(
                "<a\nhref=\"/users/<%= user.id %>\"\naria-label=\"<%= user.name %>\"\n>Profile</a>\n"
            ),
            "<a\n  href=\"/users/<%= user.id %>\"\n  aria-label=\"<%= user.name %>\"\n>\n  Profile\n</a>\n"
        );
    }

    #[test]
    fn wraps_long_erb_output_tags_without_splitting_ruby() {
        assert_eq!(
            format_with_options(
                r#"<%= link_to "Edit profile", edit_user_path(user), class: "button button--primary", data: { turbo_frame: "_top" } %>"#,
                FormatOptions {
                    line_width: 60,
                    ..FormatOptions::default()
                }
            ),
            "<%=\n  link_to \"Edit profile\", edit_user_path(user), class: \"button button--primary\", data: { turbo_frame: \"_top\" }\n%>\n"
        );
    }

    #[test]
    fn wraps_long_erb_code_tags_without_splitting_ruby() {
        assert_eq!(
            format_with_options(
                r#"<% cache ["profile-card", user.cache_key_with_version, current_user.cache_key_with_version] %>"#,
                FormatOptions {
                    line_width: 60,
                    ..FormatOptions::default()
                }
            ),
            "<%\n  cache [\"profile-card\", user.cache_key_with_version, current_user.cache_key_with_version]\n%>\n"
        );
    }

    #[test]
    fn wraps_long_erb_block_opening_tags_without_splitting_ruby() {
        assert_eq!(
            format_with_options(
                "<% if current_user.admin? && feature_enabled?(:new_dashboard) && account.active? %>\n<p>Hello</p>\n<% end %>\n",
                FormatOptions {
                    line_width: 60,
                    ..FormatOptions::default()
                }
            ),
            "<%\n  if current_user.admin? && feature_enabled?(:new_dashboard) && account.active?\n%>\n  <p>Hello</p>\n<% end %>\n"
        );
    }

    #[test]
    fn preserves_existing_multiline_erb_output_shape() {
        assert_eq!(
            format(
                "<%=\n  link_to(\n    \"Edit profile\",\n    edit_user_path(user),\n    class: \"button\"\n  )\n%>\n"
            ),
            "<%=\n  link_to(\n    \"Edit profile\",\n    edit_user_path(user),\n    class: \"button\"\n  )\n%>\n"
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

    #[test]
    fn snapshots_formatter_edge_cases_fixture() {
        insta::assert_snapshot!(
            "formatter_edge_cases_fixture",
            format(formatter_edge_cases_fixture())
        );
    }

    #[test]
    fn snapshots_real_template_audit_fixture() {
        insta::assert_snapshot!(
            "real_template_audit_fixture",
            format(real_template_audit_fixture())
        );
    }

    #[test]
    fn formatted_sample_fixture_is_idempotent() {
        assert_format_is_idempotent(sample_fixture());
    }

    #[test]
    fn formatted_stability_fixture_is_idempotent() {
        assert_format_is_idempotent(stability_fixture());
    }

    #[test]
    fn formatted_formatter_audit_fixture_is_idempotent() {
        assert_format_is_idempotent(formatter_audit_fixture());
    }

    #[test]
    fn formatted_formatter_edge_cases_fixture_is_idempotent() {
        assert_format_is_idempotent(formatter_edge_cases_fixture());
    }

    #[test]
    fn formatted_real_template_audit_fixture_is_idempotent() {
        assert_format_is_idempotent(real_template_audit_fixture());
    }

    fn assert_format_is_idempotent(input: &str) {
        let once = format(input);
        let twice = format(&once);

        assert_eq!(twice, once);
    }

    fn sample_fixture() -> &'static str {
        include_str!("../samples/sample.html.erb")
    }

    fn stability_fixture() -> &'static str {
        "<!DOCTYPE html>\n<div class=\"page <%= page_class %>\">\n<!-- profile card -->\n<img src=\"<%= avatar_url %>\" alt=\"<%= user.name %>\">\n<input type=\"checkbox\" checked=\"<%= checked %>\">\n<% if user %>\n<section>\n<a href=\"/users/<%= user.id %>\"><%= user.name %></a>\n<br>\n<% unless notifications.empty? %>\n<ul>\n<% notifications.each do |notification| %>\n<li><%= notification.title %></li>\n<% end %>\n</ul>\n<% end %>\n</section>\n<% else %>\n<p>Please sign in</p>\n<% end %>\n</div>\n"
    }

    fn formatter_audit_fixture() -> &'static str {
        include_str!("../samples/formatter-audit.html.erb")
    }

    fn formatter_edge_cases_fixture() -> &'static str {
        include_str!("../samples/formatter-edge-cases.html.erb")
    }

    fn real_template_audit_fixture() -> &'static str {
        include_str!("../samples/real-template-audit.html.erb")
    }
}
