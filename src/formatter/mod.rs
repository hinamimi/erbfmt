use std::collections::HashMap;

use crate::mixed_parser::{Document, ErbBranch, Node, SourceRange};

mod erb;
mod ignore_directive;
mod options;
mod preserve;
mod tag;

use erb::{ErbTagMarker, format_erb_comment, format_erb_tag_inline, formatted_erb_code_lines};
use ignore_directive::formatter_ignore_ranges;
pub use options::{FormatOptions, IndentStyle, LineEnding};
use preserve::{is_format_sensitive_html_element, render_preserved_node, render_preserved_nodes};
use tag::ParsedTag;

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

struct Formatter<'a> {
    options: FormatOptions,
    output: String,
    source: Option<&'a str>,
    preserved_ranges: HashMap<SourceRange, SourceRange>,
}

#[derive(Clone, Copy)]
enum FormattingNode<'a> {
    Node(&'a Node),
    HtmlText {
        text: &'a str,
        range: Option<SourceRange>,
    },
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

    fn format_nodes(&mut self, nodes: &[Node], depth: usize) {
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
            Node::ErbCode(code) => self.write_erb_tag(depth, ErbTagMarker::Code, code),
            Node::ErbComment(comment) => {
                self.write_indented_line(depth, &format_erb_comment(comment));
            }
            Node::ErbOutput(code) => self.write_erb_tag(depth, ErbTagMarker::Output, code),
            Node::ErbBlock {
                code,
                output,
                children,
                branches,
                ..
            } => {
                self.write_erb_block(
                    node,
                    ErbBlockParts {
                        code,
                        output: *output,
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

    fn write_erb_block_with_inline_boundaries(
        &mut self,
        nodes: &[FormattingNode<'_>],
        index: usize,
        depth: usize,
    ) -> Option<usize> {
        let (prefix_start, prefix_end, block_index) = if is_inline_formatting_node(nodes[index]) {
            let start = index;
            let mut end = index;

            while end < nodes.len() && is_inline_formatting_node(nodes[end]) {
                end += 1;
            }

            if end >= nodes.len()
                || !self.formatting_nodes_share_source_line(nodes[end - 1], nodes[end])
            {
                return None;
            }

            (start, end, end)
        } else {
            (index, index, index)
        };

        let FormattingNode::Node(block) = nodes[block_index] else {
            return None;
        };
        let Node::ErbBlock {
            code,
            output,
            children,
            branches,
            ..
        } = block.unspanned()
        else {
            return None;
        };

        if self.is_ignored_node(block) {
            return None;
        }

        let mut suffix_start = block_index + 1;
        let mut suffix_end = suffix_start;

        if suffix_start < nodes.len()
            && self.formatting_nodes_share_source_line(nodes[block_index], nodes[suffix_start])
        {
            while suffix_end < nodes.len() && is_inline_formatting_node(nodes[suffix_end]) {
                suffix_end += 1;
            }
        } else {
            suffix_start = suffix_end;
        }

        if prefix_start == prefix_end && suffix_start == suffix_end {
            return None;
        }

        let prefix = render_inline_formatting_nodes_untrimmed(&nodes[prefix_start..prefix_end]);
        let suffix = render_inline_formatting_nodes_untrimmed(&nodes[suffix_start..suffix_end]);

        self.write_erb_block(
            block,
            ErbBlockParts {
                code,
                output: *output,
                children,
                branches,
                range: block.source_range(),
            },
            depth,
            &prefix,
            &suffix,
        );

        Some(suffix_end)
    }

    fn write_erb_block(
        &mut self,
        node: &Node,
        parts: ErbBlockParts<'_>,
        depth: usize,
        prefix: &str,
        suffix: &str,
    ) {
        if self.can_keep_erb_block_inline(parts.range) {
            self.write_indented_line(
                depth,
                &format!("{prefix}{}{suffix}", render_preserved_node(node)),
            );
            return;
        }

        let marker = ErbTagMarker::from_output(parts.output);
        if prefix.is_empty() {
            self.write_erb_tag(depth, marker, parts.code);
        } else {
            self.write_indented_line(
                depth,
                &format!("{prefix}{}", format_erb_tag_inline(marker, parts.code)),
            );
        }

        self.format_nodes(parts.children, depth + 1);
        self.format_erb_branches(parts.branches, depth);
        self.write_indented_line(depth, &format!("<% end %>{suffix}"));
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
        range: Option<SourceRange>,
        depth: usize,
    ) {
        if is_format_sensitive_html_element(name, open)
            || self.has_erb_block_inline_child_boundary(children)
        {
            self.write_format_sensitive_html_element(open, close, children, depth);
            return;
        }

        if can_render_inline(children) && self.can_keep_html_element_inline(open, depth) {
            let content = render_inline_nodes_untrimmed(children);
            self.write_indented_line(depth, &format!("{open}{content}{close}"));
        } else {
            self.write_html_element_multiline(open, close, children, range, depth);
        }
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

    fn can_keep_html_element_inline(&self, open: &str, depth: usize) -> bool {
        let trimmed = open.trim();

        !trimmed.contains('\n') && self.fits_on_line(depth, trimmed)
    }

    fn write_tag(&mut self, raw: &str, depth: usize) {
        self.write_tag_with_suffix(raw, depth, "");
    }

    fn write_tag_with_suffix(&mut self, raw: &str, depth: usize, suffix: &str) {
        let trimmed = raw.trim();
        let is_multiline = trimmed.contains('\n');

        if !is_multiline && self.fits_on_line(depth, trimmed) {
            self.write_indented_line(depth, &format!("{trimmed}{suffix}"));
            return;
        }

        let Some(tag) = ParsedTag::parse(trimmed) else {
            self.write_indented_line(depth, &format!("{trimmed}{suffix}"));
            return;
        };

        if tag.attributes.is_empty() {
            self.write_indented_line(depth, &format!("{}{}", tag.inline(), suffix));
            return;
        }

        self.write_indented_line(depth, &format!("<{}", tag.name));

        for attribute in &tag.attributes {
            self.write_indented_line(depth + 1, attribute);
        }

        self.write_indented_line(depth, &format!("{}{suffix}", tag.closing_marker()));
    }

    fn render_tag_with_indent(&self, raw: &str, depth: usize) -> String {
        let trimmed = raw.trim();
        let indent = self.indent(depth);
        let is_multiline = trimmed.contains('\n');

        if !is_multiline && self.fits_on_line(depth, trimmed) {
            return format!("{indent}{trimmed}");
        }

        let Some(tag) = ParsedTag::parse(trimmed) else {
            return format!("{indent}{trimmed}");
        };

        if tag.attributes.is_empty() {
            return format!("{indent}{}", tag.inline());
        }

        let line_ending = self.options.line_ending.as_str();
        let mut rendered = format!("{indent}<{}{}", tag.name, line_ending);

        for attribute in &tag.attributes {
            rendered.push_str(&self.indent(depth + 1));
            rendered.push_str(attribute);
            rendered.push_str(line_ending);
        }

        rendered.push_str(&indent);
        rendered.push_str(tag.closing_marker());
        rendered
    }

    fn write_erb_tag(&mut self, depth: usize, marker: ErbTagMarker, code: &str) {
        let code = code.trim();
        let inline = format_erb_tag_inline(marker, code);

        if !code.contains('\n') && self.fits_on_line(depth, &inline) {
            self.write_indented_line(depth, &inline);
            return;
        }

        self.write_indented_line(depth, marker.as_str());

        for line in formatted_erb_code_lines(code) {
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

    fn is_ignored_node(&self, node: &Node) -> bool {
        node.source_range()
            .is_some_and(|range| self.preserved_ranges.contains_key(&range))
    }

    fn formatting_nodes_share_source_line(
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

    fn can_keep_erb_block_inline(&self, range: Option<SourceRange>) -> bool {
        let Some(source) = self.source else {
            return false;
        };
        let Some(range) = range else {
            return false;
        };
        let Some(raw) = source.get(range.start..range.end) else {
            return false;
        };

        !raw.contains(['\n', '\r'])
    }
}

#[derive(Default)]
struct HtmlElementBoundaries {
    open_child_same_line: bool,
    child_close_same_line: bool,
}

struct ErbBlockParts<'a> {
    code: &'a str,
    output: bool,
    children: &'a [Node],
    branches: &'a [ErbBranch],
    range: Option<SourceRange>,
}

fn can_render_inline(nodes: &[Node]) -> bool {
    nodes.iter().all(is_inline_node)
}

fn split_formatting_nodes<'a>(
    nodes: &'a [Node],
    preserved_ranges: &HashMap<SourceRange, SourceRange>,
) -> Vec<FormattingNode<'a>> {
    let mut split = Vec::new();

    for node in nodes {
        let is_preserved = node
            .source_range()
            .is_some_and(|range| preserved_ranges.contains_key(&range));

        if !is_preserved
            && let Node::HtmlText(text) = node.unspanned()
            && text.contains('\n')
        {
            split_multiline_html_text(text, node.source_range(), &mut split);
        } else {
            split.push(FormattingNode::Node(node));
        }
    }

    split
}

fn split_multiline_html_text<'a>(
    text: &'a str,
    range: Option<SourceRange>,
    nodes: &mut Vec<FormattingNode<'a>>,
) {
    let mut characters = text.char_indices();
    let Some((_, first)) = characters.next() else {
        return;
    };
    let mut start = 0;
    let mut is_whitespace = first.is_whitespace();

    for (index, character) in characters {
        if character.is_whitespace() == is_whitespace {
            continue;
        }

        nodes.push(FormattingNode::HtmlText {
            text: &text[start..index],
            range: split_html_text_range(range, start, index),
        });
        start = index;
        is_whitespace = !is_whitespace;
    }

    nodes.push(FormattingNode::HtmlText {
        text: &text[start..],
        range: split_html_text_range(range, start, text.len()),
    });
}

fn split_html_text_range(
    range: Option<SourceRange>,
    start: usize,
    end: usize,
) -> Option<SourceRange> {
    range.map(|range| SourceRange {
        start: range.start + start,
        end: range.start + end,
    })
}

fn formatting_node_source_range(node: FormattingNode<'_>) -> Option<SourceRange> {
    match node {
        FormattingNode::Node(node) => node.source_range(),
        FormattingNode::HtmlText { range, .. } => range,
    }
}

fn is_inline_formatting_node(node: FormattingNode<'_>) -> bool {
    match node {
        FormattingNode::Node(node) => is_inline_node(node),
        FormattingNode::HtmlText { text, .. } => !text.contains('\n'),
    }
}

fn leading_inline_boundary_nodes(nodes: &[FormattingNode<'_>]) -> usize {
    nodes
        .iter()
        .take_while(|node| is_inline_boundary_node(**node))
        .count()
}

fn trailing_inline_boundary_nodes(nodes: &[FormattingNode<'_>], prefix_count: usize) -> usize {
    nodes
        .iter()
        .enumerate()
        .rev()
        .take_while(|(_, node)| is_inline_boundary_node(**node))
        .map(|(index, _)| index)
        .last()
        .unwrap_or(nodes.len())
        .max(prefix_count)
}

fn is_inline_node(node: &Node) -> bool {
    match node.unspanned() {
        Node::HtmlText(text) => !text.contains('\n'),
        Node::HtmlElement {
            name,
            open,
            children,
            ..
        } => !is_format_sensitive_html_element(name, open) && can_render_inline(children),
        Node::HtmlSelfClosing { .. }
        | Node::HtmlVoid { .. }
        | Node::HtmlComment(_)
        | Node::ErbComment(_)
        | Node::ErbCode(_)
        | Node::ErbOutput(_) => true,
        Node::HtmlDoctype(_) | Node::Spanned { .. } | Node::ErbBlock { .. } => false,
    }
}

fn is_inline_boundary_node(node: FormattingNode<'_>) -> bool {
    match node {
        FormattingNode::HtmlText { text, .. } => !text.contains('\n'),
        FormattingNode::Node(node) => is_inline_boundary_html_node(node),
    }
}

fn is_inline_boundary_html_node(node: &Node) -> bool {
    match node.unspanned() {
        Node::HtmlText(text) => !text.contains('\n'),
        Node::HtmlElement {
            name,
            open,
            children,
            ..
        } => {
            !is_format_sensitive_html_element(name, open)
                && is_inline_boundary_html_tag(name)
                && children.iter().all(is_inline_boundary_html_node)
        }
        Node::HtmlVoid { name, .. } | Node::HtmlSelfClosing { name, .. } => {
            is_inline_boundary_html_tag(name)
        }
        Node::HtmlComment(_) | Node::ErbComment(_) | Node::ErbCode(_) | Node::ErbOutput(_) => true,
        Node::HtmlDoctype(_) | Node::Spanned { .. } | Node::ErbBlock { .. } => false,
    }
}

fn is_inline_boundary_html_tag(name: &str) -> bool {
    let normalized = name.to_ascii_lowercase();

    is_custom_html_element_name(&normalized)
        || matches!(
            normalized.as_str(),
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
                | "mark"
                | "meter"
                | "object"
                | "output"
                | "picture"
                | "progress"
                | "q"
                | "ruby"
                | "s"
                | "samp"
                | "select"
                | "slot"
                | "small"
                | "span"
                | "strong"
                | "sub"
                | "sup"
                | "svg"
                | "textarea"
                | "time"
                | "u"
                | "var"
                | "video"
                | "wbr"
        )
}

fn is_custom_html_element_name(name: &str) -> bool {
    name.contains('-')
}

fn render_inline_formatting_nodes(nodes: &[FormattingNode<'_>]) -> String {
    render_inline_formatting_nodes_untrimmed(nodes)
        .trim()
        .to_string()
}

fn render_inline_formatting_nodes_untrimmed(nodes: &[FormattingNode<'_>]) -> String {
    nodes
        .iter()
        .map(|node| match node {
            FormattingNode::Node(node) => render_inline_node(node),
            FormattingNode::HtmlText { text, .. } => (*text).to_string(),
        })
        .collect()
}

fn render_inline_nodes_untrimmed(nodes: &[Node]) -> String {
    nodes.iter().map(render_inline_node).collect::<String>()
}

fn render_inline_node(node: &Node) -> String {
    match node.unspanned() {
        Node::HtmlText(text) => text.clone(),
        Node::HtmlElement {
            open,
            close,
            children,
            ..
        } => format!("{open}{}{close}", render_inline_nodes_untrimmed(children)),
        Node::HtmlSelfClosing { raw, .. } | Node::HtmlVoid { raw, .. } => raw.clone(),
        Node::HtmlComment(comment) => comment.clone(),
        Node::ErbCode(code) => format_erb_tag_inline(ErbTagMarker::Code, code.trim()),
        Node::ErbComment(comment) => format_erb_comment(comment),
        Node::ErbOutput(code) => format_erb_tag_inline(ErbTagMarker::Output, code.trim()),
        Node::HtmlDoctype(_) | Node::Spanned { .. } | Node::ErbBlock { .. } => {
            unreachable!("node cannot render inline")
        }
    }
}

fn is_line_ending_char(ch: char) -> bool {
    matches!(ch, '\n' | '\r')
}

#[cfg(test)]
mod tests;
