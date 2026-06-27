use std::collections::HashMap;

use crate::mixed_parser::{Document, Node, SourceRange};

use crate::lexer::{ErbTag, ErbTagSyntax};

use super::erb::{format_erb_comment, format_erb_tag_inline, formatted_erb_code_lines};
use super::erb_block::ErbBlockParts;
use super::ignore_directive::formatter_ignore_ranges;
use super::inline::{
    FormattingNode, can_render_inline, formatting_node_source_range, is_inline_formatting_node,
    leading_inline_boundary_nodes, render_inline_formatting_nodes,
    render_inline_formatting_nodes_untrimmed, render_inline_nodes_untrimmed,
    split_formatting_nodes, trailing_inline_boundary_nodes,
};
use super::options::{FormatOptions, IndentStyle};
use super::preserve::{is_format_sensitive_html_element, render_preserved_nodes};
use super::tag::{TagRenderContext, normalize_tag, render_tag};

#[allow(dead_code)]
pub fn format_document(document: &Document) -> String {
    format_document_with_options(document, FormatOptions::default())
}

pub fn format_document_with_options(document: &Document, options: FormatOptions) -> String {
    let mut formatter = Formatter::new(document, None, options);
    formatter.format_nodes(&document.children, 0);
    formatter.finish()
}

pub fn format_document_with_source(
    document: &Document,
    source: &str,
    options: FormatOptions,
) -> String {
    let mut formatter = Formatter::new(document, Some(source), options);
    formatter.format_nodes(&document.children, 0);
    formatter.finish()
}

pub(super) struct Formatter<'a> {
    options: FormatOptions,
    output: String,
    pub(super) source: Option<&'a str>,
    preserved_ranges: HashMap<SourceRange, SourceRange>,
}

impl<'a> Formatter<'a> {
    fn new(document: &Document, source: Option<&'a str>, options: FormatOptions) -> Self {
        let preserved_ranges = source
            .map(|source| formatter_ignore_ranges(&document.children, source))
            .unwrap_or_default();

        Self {
            options,
            output: String::new(),
            source,
            preserved_ranges,
        }
    }

    pub(super) fn format_nodes(&mut self, nodes: &[Node], depth: usize) {
        let nodes = split_formatting_nodes(nodes, &self.preserved_ranges);
        self.format_split_nodes(&nodes, depth);
    }

    fn format_split_nodes(&mut self, nodes: &[FormattingNode<'_>], depth: usize) {
        let mut index = 0;

        while index < nodes.len() {
            if let FormattingNode::Node(node) = nodes[index]
                && self.write_ignored_node(node)
            {
                index += 1;
                continue;
            }

            if let Some(next_index) =
                self.write_erb_block_with_inline_boundaries(nodes, index, depth)
            {
                index = next_index;
                continue;
            }

            if is_inline_formatting_node(nodes[index]) {
                let start = index;

                while index < nodes.len() && is_inline_formatting_node(nodes[index]) {
                    index += 1;
                }

                if index - start == 1 {
                    self.format_formatting_node(nodes[start], depth);
                } else {
                    self.write_inline_formatting_nodes(&nodes[start..index], depth);
                }
            } else {
                self.format_formatting_node(nodes[index], depth);
                index += 1;
            }
        }
    }

    fn format_formatting_node(&mut self, node: FormattingNode<'_>, depth: usize) {
        match node {
            FormattingNode::Node(node) => self.format_node(node, depth),
            FormattingNode::HtmlText { text, .. } => self.write_text(text, depth),
        }
    }

    fn format_node(&mut self, node: &Node, depth: usize) {
        let range = node.source_range();

        match node.unspanned() {
            Node::HtmlText(text) => self.write_text(text, depth),
            Node::HtmlElement {
                name,
                open,
                close,
                children,
            } => self.write_html_element(name, open, close, children, range, depth),
            Node::HtmlSelfClosing { raw, .. } | Node::HtmlVoid { raw, .. } => {
                self.write_tag(raw, depth)
            }
            Node::HtmlComment(comment) | Node::HtmlDoctype(comment) => {
                self.write_indented_line(depth, comment);
            }
            Node::ErbCode(tag) => self.write_erb_tag(depth, tag),
            Node::ErbComment(comment) => {
                self.write_indented_line(depth, &format_erb_comment(comment));
            }
            Node::ErbOutput(tag) => self.write_erb_tag(depth, tag),
            Node::ErbBlock {
                tag,
                end_tag,
                children,
                branches,
                ..
            } => {
                self.write_erb_block(
                    node,
                    ErbBlockParts {
                        tag,
                        end_tag,
                        children,
                        branches,
                        range,
                    },
                    depth,
                    "",
                    "",
                );
            }
            Node::Spanned { .. } => unreachable!("unspanned node cannot remain wrapped"),
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
        range: Option<SourceRange>,
        depth: usize,
    ) {
        if is_format_sensitive_html_element(name, open)
            || self.has_erb_block_inline_child_boundary(children)
        {
            self.write_format_sensitive_html_element(open, close, children, depth);
            return;
        }

        if !open.contains('\n') && can_render_inline(children) {
            let content = render_inline_nodes_untrimmed(children);
            let open = normalize_tag(open).unwrap_or_else(|| open.to_string());
            let inline = format!("{open}{content}{close}");
            let inline_width_target = if children.is_empty() {
                inline.as_str()
            } else {
                open.as_str()
            };

            if self.can_keep_html_element_inline(inline_width_target, depth) {
                self.write_indented_line(depth, &inline);
                return;
            }
        }

        self.write_html_element_multiline(open, close, children, range, depth);
    }

    fn write_html_element_multiline(
        &mut self,
        open: &str,
        close: &str,
        children: &[Node],
        range: Option<SourceRange>,
        depth: usize,
    ) {
        let boundaries = self.html_element_boundaries(range, open, close);
        let children = split_formatting_nodes(children, &self.preserved_ranges);
        let prefix_count = if boundaries.open_child_same_line {
            leading_inline_boundary_nodes(&children)
        } else {
            0
        };
        let suffix_start = if boundaries.child_close_same_line {
            trailing_inline_boundary_nodes(&children, prefix_count)
        } else {
            children.len()
        };
        let prefix = render_inline_formatting_nodes_untrimmed(&children[..prefix_count]);

        if prefix_count == children.len() {
            let suffix = if boundaries.child_close_same_line {
                format!("{prefix}{close}")
            } else {
                prefix
            };
            self.write_tag_with_suffix(open, depth, &suffix);

            if !boundaries.child_close_same_line {
                self.write_indented_line(depth, close);
            }

            return;
        }

        self.write_tag_with_suffix(open, depth, &prefix);
        self.format_split_nodes(
            &children[prefix_count..suffix_start],
            self.html_child_depth(depth),
        );

        if suffix_start < children.len() {
            let suffix = render_inline_formatting_nodes_untrimmed(&children[suffix_start..]);
            self.write_indented_line(self.html_child_depth(depth), &format!("{suffix}{close}"));
        } else {
            self.write_indented_line(depth, close);
        }
    }

    fn write_format_sensitive_html_element(
        &mut self,
        open: &str,
        close: &str,
        children: &[Node],
        depth: usize,
    ) {
        let mut rendered = self.render_tag_with_indent(open, depth);
        rendered.push_str(&render_preserved_nodes(children));
        rendered.push_str(close);

        self.output.push_str(&rendered);

        if !rendered.ends_with(['\n', '\r']) {
            self.output.push_str(self.options.line_ending.as_str());
        }
    }

    fn write_inline_formatting_nodes(&mut self, nodes: &[FormattingNode<'_>], depth: usize) {
        let inline = render_inline_formatting_nodes(nodes);

        if inline.is_empty() {
            return;
        }

        self.write_indented_line(depth, &inline);
    }

    fn can_keep_html_element_inline(&self, inline: &str, depth: usize) -> bool {
        let trimmed = inline.trim();

        !trimmed.contains('\n') && self.fits_on_line(depth, trimmed)
    }

    fn write_tag(&mut self, raw: &str, depth: usize) {
        self.write_tag_with_suffix(raw, depth, "");
    }

    fn write_tag_with_suffix(&mut self, raw: &str, depth: usize, suffix: &str) {
        let Some(rendered) = self.render_tag_with_suffix(raw, depth, suffix) else {
            return;
        };

        self.output.push_str(&rendered);
        self.output.push_str(self.options.line_ending.as_str());
    }

    fn render_tag_with_indent(&self, raw: &str, depth: usize) -> String {
        self.render_tag_with_suffix(raw, depth, "")
            .unwrap_or_default()
    }

    fn render_tag_with_suffix(&self, raw: &str, depth: usize, suffix: &str) -> Option<String> {
        let indent = self.indent(depth);
        let child_indent = self.indent(depth + 1);

        render_tag(
            raw,
            suffix,
            TagRenderContext {
                indent: &indent,
                child_indent: &child_indent,
                line_ending: self.options.line_ending.as_str(),
                line_width: self.options.line_width,
            },
        )
    }

    pub(super) fn write_erb_tag(&mut self, depth: usize, tag: &ErbTag) {
        self.write_erb_tag_with_syntax(depth, tag.syntax, &tag.code);
    }

    pub(super) fn write_erb_tag_with_syntax(
        &mut self,
        depth: usize,
        syntax: ErbTagSyntax,
        code: &str,
    ) {
        let code = code.trim();
        let inline = format_erb_tag_inline(syntax, code);

        if !code.contains('\n') && self.fits_on_line(depth, &inline) {
            self.write_indented_line(depth, &inline);
            return;
        }

        self.write_indented_line(depth, syntax.open.as_str());

        for line in formatted_erb_code_lines(code) {
            self.write_indented_code_line(depth + 1, &line);
        }

        self.write_indented_line(depth, syntax.close.as_str());
    }

    fn html_child_depth(&self, depth: usize) -> usize {
        depth + usize::from(self.options.indent_html)
    }

    fn fits_on_line(&self, depth: usize, text: &str) -> bool {
        self.indent(depth).chars().count() + text.chars().count() <= self.options.line_width
    }

    pub(super) fn write_indented_line(&mut self, depth: usize, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }

        self.output.push_str(&self.indent(depth));
        self.output.push_str(trimmed);
        self.output.push_str(self.options.line_ending.as_str());
    }

    fn write_blank_line(&mut self) {
        let line_ending = self.options.line_ending.as_str();
        let double_line_ending = format!("{line_ending}{line_ending}");

        if self.output.is_empty() || self.output.ends_with(&double_line_ending) {
            return;
        }

        if !self.output.ends_with(line_ending) {
            self.output.push_str(line_ending);
        }

        self.output.push_str(line_ending);
    }

    fn write_indented_code_line(&mut self, depth: usize, line: &str) {
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            return;
        }

        self.output.push_str(&self.indent(depth));
        self.output.push_str(trimmed);
        self.output.push_str(self.options.line_ending.as_str());
    }

    fn indent(&self, depth: usize) -> String {
        match self.options.indent_style {
            IndentStyle::Space => " ".repeat(self.options.indent_width * depth),
            IndentStyle::Tab => "\t".repeat(depth),
        }
    }

    fn finish(mut self) -> String {
        if !self.options.trailing_newline {
            self.output = self.output.trim_end_matches(['\r', '\n']).to_string();
        }

        self.output
    }

    fn write_ignored_node(&mut self, node: &Node) -> bool {
        if !self.is_ignored_node(node) {
            return false;
        }

        let Some(source) = self.source else {
            return false;
        };
        let Some(node_range) = node.source_range() else {
            return false;
        };
        let Some(preserved_range) = self.preserved_ranges.get(&node_range) else {
            return false;
        };
        let Some(raw) = source.get(preserved_range.start..preserved_range.end) else {
            return false;
        };

        self.output.push_str(raw);
        true
    }

    pub(super) fn is_ignored_node(&self, node: &Node) -> bool {
        node.source_range()
            .is_some_and(|range| self.preserved_ranges.contains_key(&range))
    }

    pub(super) fn formatting_nodes_share_source_line(
        &self,
        left: FormattingNode<'_>,
        right: FormattingNode<'_>,
    ) -> bool {
        let Some(left) = formatting_node_source_range(left) else {
            return false;
        };
        let Some(right) = formatting_node_source_range(right) else {
            return false;
        };

        self.source_ranges_share_line(left, right)
    }

    fn has_erb_block_inline_child_boundary(&self, children: &[Node]) -> bool {
        children.iter().enumerate().any(|(index, child)| {
            if !matches!(child.unspanned(), Node::ErbBlock { .. }) {
                return false;
            }

            let previous_is_inline = index
                .checked_sub(1)
                .and_then(|previous| children.get(previous))
                .is_some_and(|previous| self.nodes_share_source_line(previous, child));
            let next_is_inline = children
                .get(index + 1)
                .is_some_and(|next| self.nodes_share_source_line(child, next));

            previous_is_inline || next_is_inline
        })
    }

    fn nodes_share_source_line(&self, left: &Node, right: &Node) -> bool {
        let Some(left) = left.source_range() else {
            return false;
        };
        let Some(right) = right.source_range() else {
            return false;
        };

        self.source_ranges_share_line(left, right)
    }

    fn source_ranges_share_line(&self, left: SourceRange, right: SourceRange) -> bool {
        let Some(source) = self.source else {
            return false;
        };
        if left.end > right.start || right.end > source.len() {
            return false;
        }

        !source[left.end..right.start].contains(['\n', '\r'])
    }

    fn html_element_boundaries(
        &self,
        range: Option<SourceRange>,
        open: &str,
        close: &str,
    ) -> HtmlElementBoundaries {
        let Some(source) = self.source else {
            return HtmlElementBoundaries::default();
        };
        let Some(range) = range else {
            return HtmlElementBoundaries::default();
        };
        let content_start = range.start + open.len();
        let Some(content_end) = range.end.checked_sub(close.len()) else {
            return HtmlElementBoundaries::default();
        };
        if content_start >= content_end || content_end > source.len() {
            return HtmlElementBoundaries::default();
        }

        let content = &source[content_start..content_end];

        HtmlElementBoundaries {
            open_child_same_line: content
                .chars()
                .next()
                .is_some_and(|ch| !is_line_ending_char(ch)),
            child_close_same_line: content
                .chars()
                .next_back()
                .is_some_and(|ch| !is_line_ending_char(ch)),
        }
    }
}

#[derive(Default)]
struct HtmlElementBoundaries {
    open_child_same_line: bool,
    child_close_same_line: bool,
}

fn is_line_ending_char(ch: char) -> bool {
    matches!(ch, '\n' | '\r')
}
