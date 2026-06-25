use std::collections::HashMap;

use crate::ignore::{IgnoreSelector, parse_ignore_directive};
use crate::mixed_parser::{Document, ErbBranch, Node, SourceRange};
use crate::ruby_format;

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
            self.write_preserved_block(
                depth,
                &render_preserved_html_element(open, close, children),
            );
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

    fn write_preserved_block(&mut self, depth: usize, block: &str) {
        if block.is_empty() {
            return;
        }

        self.output.push_str(&self.indent(depth));
        self.output.push_str(block);

        if !block.ends_with(['\n', '\r']) {
            self.output.push_str(self.options.line_ending.as_str());
        }
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

fn formatter_ignore_ranges(nodes: &[Node], source: &str) -> HashMap<SourceRange, SourceRange> {
    let mut ranges = HashMap::new();
    collect_formatter_ignore_ranges(nodes, source, &mut ranges);
    ranges
}

fn collect_formatter_ignore_ranges(
    nodes: &[Node],
    source: &str,
    ranges: &mut HashMap<SourceRange, SourceRange>,
) {
    for (index, node) in nodes.iter().enumerate() {
        if is_comment_node(node)
            && let Some(directive_range) = node.source_range()
            && let Some(raw) = source.get(directive_range.start..directive_range.end)
            && !raw.contains(['\r', '\n'])
            && parse_ignore_directive(raw).is_some_and(|directive| {
                matches!(
                    directive.selector,
                    IgnoreSelector::Format | IgnoreSelector::All
                )
            })
            && let Some(target) = nodes[index + 1..]
                .iter()
                .find(|candidate| !is_whitespace_node(candidate))
            && let Some(target_range) = target.source_range()
            && let Some(preserved_range) =
                formatter_ignore_line_range(source, directive_range, target_range)
        {
            ranges.insert(target_range, preserved_range);
        }

        match node.unspanned() {
            Node::HtmlElement { children, .. } => {
                collect_formatter_ignore_ranges(children, source, ranges);
            }
            Node::ErbBlock {
                children, branches, ..
            } => {
                collect_formatter_ignore_ranges(children, source, ranges);

                for branch in branches {
                    collect_formatter_ignore_ranges(&branch.children, source, ranges);
                }
            }
            Node::HtmlText(_)
            | Node::HtmlSelfClosing { .. }
            | Node::HtmlVoid { .. }
            | Node::HtmlComment(_)
            | Node::HtmlDoctype(_)
            | Node::ErbCode(_)
            | Node::ErbComment(_)
            | Node::ErbOutput(_)
            | Node::Spanned { .. } => {}
        }
    }
}

fn is_comment_node(node: &Node) -> bool {
    matches!(node.unspanned(), Node::HtmlComment(_) | Node::ErbComment(_))
}

fn is_whitespace_node(node: &Node) -> bool {
    matches!(node.unspanned(), Node::HtmlText(text) if text.trim().is_empty())
}

fn formatter_ignore_line_range(
    source: &str,
    directive: SourceRange,
    target: SourceRange,
) -> Option<SourceRange> {
    if directive.end > target.start || target.end > source.len() {
        return None;
    }

    let directive_line_start = source[..directive.start]
        .rfind('\n')
        .map_or(0, |index| index + 1);
    if !source[directive_line_start..directive.start]
        .chars()
        .all(is_horizontal_whitespace)
    {
        return None;
    }

    let between = &source[directive.end..target.start];
    let newline = between.find('\n')?;
    if !between[..newline].chars().all(is_horizontal_whitespace)
        || !between[newline + 1..]
            .chars()
            .all(is_indentation_whitespace)
    {
        return None;
    }

    let target_line_start = directive.end + newline + 1;
    let after_target = &source[target.end..];
    let target_line_end = if let Some(newline) = after_target.find('\n') {
        if !after_target[..newline]
            .chars()
            .all(is_horizontal_whitespace)
        {
            return None;
        }

        target.end + newline + 1
    } else {
        if !after_target.chars().all(is_horizontal_whitespace) {
            return None;
        }

        source.len()
    };

    Some(SourceRange {
        start: target_line_start,
        end: target_line_end,
    })
}

fn is_horizontal_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\r')
}

fn is_indentation_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\t')
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

fn render_preserved_html_element(open: &str, close: &str, children: &[Node]) -> String {
    format!("{open}{}{close}", render_preserved_nodes(children))
}

fn render_preserved_nodes(nodes: &[Node]) -> String {
    nodes.iter().map(render_preserved_node).collect()
}

fn render_preserved_node(node: &Node) -> String {
    match node.unspanned() {
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
        Node::ErbComment(comment) => format_erb_comment(comment),
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
        Node::Spanned { .. } => unreachable!("unspanned node cannot remain wrapped"),
    }
}

fn is_format_sensitive_html_element(name: &str, open: &str) -> bool {
    is_format_sensitive_html_tag(name)
        || has_contenteditable_attribute(open)
        || has_whitespace_sensitive_style_attribute(open)
}

fn is_format_sensitive_html_tag(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "pre"
            | "textarea"
            | "script"
            | "style"
            | "xmp"
            | "listing"
            | "svg"
            | "math"
            | "template"
            | "noscript"
    )
}

fn has_contenteditable_attribute(open: &str) -> bool {
    ParsedTag::parse(open).is_some_and(|tag| {
        tag.attributes
            .iter()
            .any(|attribute| attribute_name(attribute).eq_ignore_ascii_case("contenteditable"))
    })
}

fn has_whitespace_sensitive_style_attribute(open: &str) -> bool {
    ParsedTag::parse(open).is_some_and(|tag| {
        tag.attributes.iter().any(|attribute| {
            attribute_name(attribute).eq_ignore_ascii_case("style")
                && attribute_value(attribute)
                    .is_some_and(|value| value.to_ascii_lowercase().contains("white-space"))
        })
    })
}

fn attribute_name(attribute: &str) -> &str {
    attribute
        .split_once('=')
        .map_or(attribute, |(name, _)| name)
        .trim()
}

fn attribute_value(attribute: &str) -> Option<&str> {
    let (_, value) = attribute.split_once('=')?;
    Some(value.trim().trim_matches(['"', '\'']))
}

fn is_line_ending_char(ch: char) -> bool {
    matches!(ch, '\n' | '\r')
}

fn format_erb_tag_inline(marker: ErbTagMarker, code: &str) -> String {
    if code.is_empty() {
        return format!("{} %>", marker.as_str());
    }

    format!("{} {} %>", marker.as_str(), code.trim())
}

fn format_erb_comment(comment: &str) -> String {
    let comment = comment.trim();

    if comment.is_empty() {
        "<%# %>".to_string()
    } else {
        format!("<%# {comment} %>")
    }
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

fn formatted_erb_code_lines(code: &str) -> Vec<String> {
    if !code.contains('\n')
        && let Some(lines) = ruby_format::fold_command_call(code)
    {
        return lines;
    }

    normalized_erb_code_lines(code)
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
    use crate::{
        lexer::{tokenize, tokenize_with_spans},
        mixed_parser::{parse, parse_spanned},
    };

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

    fn format_source(input: &str) -> String {
        format_source_with_options(input, FormatOptions::default())
    }

    fn format_source_with_options(input: &str, options: FormatOptions) -> String {
        let tokens = tokenize_with_spans(input).unwrap();
        let document = parse_spanned(&tokens).unwrap();

        format_document_with_source(&document, input, options)
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
    fn preserves_text_adjacent_to_inline_html_inside_erb_blocks() {
        let code_block =
            "<% link_to(user_path(user)) do %>\n<i class=\"icon\"></i>test\n<% end %>\n";
        let output_block =
            "<%= link_to(user_path(user)) do %>\n<i class=\"icon\"></i>test\n<% end %>\n";
        let formatted =
            "<% link_to(user_path(user)) do %>\n  <i class=\"icon\"></i>test\n<% end %>\n";

        assert_eq!(format(code_block), formatted);
        assert_eq!(format(formatted), formatted);
        assert_eq!(
            format(output_block),
            "<%= link_to(user_path(user)) do %>\n  <i class=\"icon\"></i>test\n<% end %>\n"
        );

        assert_eq!(
            format_with_options(
                "<p><i class=\"long-icon-name\"></i>test</p>\n",
                FormatOptions {
                    line_width: 20,
                    ..FormatOptions::default()
                }
            ),
            "<p><i class=\"long-icon-name\"></i>test</p>\n"
        );
    }

    #[test]
    fn preserves_inline_html_adjacency_in_both_directions_across_lines() {
        let input = "<i class=\"icon\"></i>テスト\nテスト<i class=\"icon\"></i>\n";

        assert_eq!(format(input), input);

        assert_eq!(
            format(&format!("<% if visible? %>\n{input}<% end %>\n")),
            "<% if visible? %>\n  <i class=\"icon\"></i>テスト\n  テスト<i class=\"icon\"></i>\n<% end %>\n"
        );

        let long_input =
            "<i class=\"long-icon-name\"></i>テスト\nテスト<i class=\"long-icon-name\"></i>\n";
        assert_eq!(
            format_with_options(
                long_input,
                FormatOptions {
                    line_width: 20,
                    ..FormatOptions::default()
                }
            ),
            long_input
        );
    }

    #[test]
    fn preserves_whitespace_boundaries_around_inline_html() {
        let input = "<p>Hello <strong>world</strong>!</p>\n<span> padded </span>\n<i></i>\ntext\n";

        assert_eq!(format(input), input);
    }

    #[test]
    fn preserves_inline_boundaries_around_custom_elements() {
        let options = FormatOptions {
            line_width: 36,
            ..FormatOptions::default()
        };
        let input = "<a class=\"button button--primary button--wide\"><ui-icon name=\"check\"></ui-icon>Done</a>\n";
        let expected = "<a\n  class=\"button button--primary button--wide\"\n><ui-icon name=\"check\"></ui-icon>Done</a>\n";

        assert_eq!(format_source_with_options(input, options), expected);
        assert_eq!(format_source_with_options(expected, options), expected);
    }

    #[test]
    fn preserves_source_boundaries_between_opening_tags_and_inline_children() {
        let options = FormatOptions {
            line_width: 36,
            ..FormatOptions::default()
        };
        let input = "<a class=\"button button--primary\" href=\"/profile\">Profile</a>\n";
        let expected =
            "<a\n  class=\"button button--primary\"\n  href=\"/profile\"\n>Profile</a>\n";

        assert_eq!(format_source_with_options(input, options), expected);
        assert_eq!(format_source_with_options(expected, options), expected);
    }

    #[test]
    fn preserves_source_boundaries_between_closing_tags_and_inline_children() {
        let input = "<p>Lead\n<strong>Body</strong>\nTail</p>\n";
        let expected = "<p>Lead\n  <strong>Body</strong>\n  Tail</p>\n";

        assert_eq!(format_source(input), expected);
        assert_eq!(format_source(expected), expected);
    }

    #[test]
    fn preserves_text_adjacency_across_inline_comments() {
        let input = "<p>first<!-- separator -->second<%# note %>third</p>\n";

        assert_eq!(format(input), input);
    }

    #[test]
    fn preserves_erb_comment_markers() {
        assert_eq!(
            format_source("<%# generated note %>\n"),
            "<%# generated note %>\n"
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
    fn preserves_single_line_erb_blocks_inline() {
        let options = FormatOptions {
            line_width: 24,
            ..FormatOptions::default()
        };
        let code_block = "<% if visible? %><span>Visible</span><% end %>\n";
        let output_block = "<%= link_to profile_path do %><i class=\"icon\"></i>Profile<% end %>\n";

        assert_eq!(format_source_with_options(code_block, options), code_block);
        assert_eq!(
            format_source_with_options(output_block, options),
            output_block
        );
    }

    #[test]
    fn preserves_inline_text_boundaries_around_multiline_erb_blocks() {
        let input = "Hello<% if user %>\n<span><%= user.name %></span>\n<% end %>!\n";
        let expected = "Hello<% if user %>\n  <span><%= user.name %></span>\n<% end %>!\n";

        assert_eq!(format_source(input), expected);
        assert_eq!(format_source(expected), expected);

        let paragraph = "<p>Hello<% if user %>\n<span>Admin</span>\n<% end %>!</p>\n";
        assert_eq!(format_source(paragraph), paragraph);
    }

    #[test]
    fn preserves_formatter_ignored_html_and_erb_nodes() {
        assert_eq!(
            format_source(formatter_ignore_fixture()),
            formatter_ignore_fixture()
        );
    }

    #[test]
    fn preserves_formatter_ignored_erb_block_subtrees() {
        let input = "<!-- erbfmt-ignore format: generated block -->\n<% if user %>\n<p   class=\"legacy\">Keep   spacing</p>\n<% else %>\n<p>Also  keep</p>\n<% end %>\n";

        assert_eq!(format_source(input), input);
    }

    #[test]
    fn preserves_formatter_ignored_nodes_with_combined_directives() {
        let input = "<!-- erbfmt-ignore all: generated markup -->\n<div   class=\"generated\"><center>Keep this</center></div>\n";

        assert_eq!(format_source(input), input);
    }

    #[test]
    fn formats_nodes_surrounding_formatter_ignored_subtrees() {
        let input = "<section>\n<!-- erbfmt-ignore format: legacy -->\n    <div   class=\"legacy\">Keep   spacing</div>\n<p>Normal</p>\n</section>\n";

        assert_eq!(
            format_source(input),
            "<section>\n  <!-- erbfmt-ignore format: legacy -->\n    <div   class=\"legacy\">Keep   spacing</div>\n  <p>Normal</p>\n</section>\n"
        );
    }

    #[test]
    fn falls_back_when_formatter_ignore_target_is_not_on_the_next_line() {
        let input = "<!-- erbfmt-ignore format: separated -->\n\n<article class=\"card\" data-controller=\"profile\" aria-label=\"Profile card\"></article>\n";

        assert_eq!(
            format_source_with_options(
                input,
                FormatOptions {
                    line_width: 48,
                    ..FormatOptions::default()
                }
            ),
            "<!-- erbfmt-ignore format: separated -->\n\n<article\n  class=\"card\"\n  data-controller=\"profile\"\n  aria-label=\"Profile card\"\n>\n</article>\n"
        );
    }

    #[test]
    fn formatter_ignore_is_idempotent() {
        let once = format_source(formatter_ignore_fixture());
        let twice = format_source(&once);

        assert_eq!(twice, once);
    }

    #[test]
    fn formatter_ignore_preserves_source_line_endings() {
        let input = "<section>\r\n<!-- erbfmt-ignore format: legacy -->\r\n    <div   class=\"legacy\">Keep   spacing</div>\r\n<p>Normal</p>\r\n</section>\r\n";

        assert_eq!(
            format_source_with_options(
                input,
                FormatOptions {
                    line_ending: LineEnding::Lf,
                    ..FormatOptions::default()
                }
            ),
            "<section>\n  <!-- erbfmt-ignore format: legacy -->\n    <div   class=\"legacy\">Keep   spacing</div>\r\n  <p>Normal</p>\n</section>\n"
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
    fn preserves_svg_math_and_contenteditable_subtrees() {
        let input = "<section>\n<svg viewBox=\"0 0 10 10\">\n  <path   d=\"M0 0L10 10\"></path>\n</svg>\n<math><mi>x</mi>  <mo>=</mo><mn>1</mn></math>\n<div contenteditable=\"true\"><p> keep  spacing</p></div>\n<div style=\"white-space: pre-line\"><span>A</span>\n    B</div>\n</section>\n";

        assert_eq!(
            format(input),
            "<section>\n  <svg viewBox=\"0 0 10 10\">\n  <path   d=\"M0 0L10 10\"></path>\n</svg>\n  <math><mi>x</mi>  <mo>=</mo><mn>1</mn></math>\n  <div contenteditable=\"true\"><p> keep  spacing</p></div>\n  <div style=\"white-space: pre-line\"><span>A</span>\n    B</div>\n</section>\n"
        );
    }

    #[test]
    fn preserves_template_and_noscript_subtrees() {
        let input = "<section>\n<template><div   class=\"legacy\">Keep  spacing</div></template>\n<noscript><p> keep  spacing</p></noscript>\n</section>\n";

        assert_eq!(
            format(input),
            "<section>\n  <template><div   class=\"legacy\">Keep  spacing</div></template>\n  <noscript><p> keep  spacing</p></noscript>\n</section>\n"
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
    fn wraps_long_erb_output_command_calls() {
        assert_eq!(
            format_with_options(
                r#"<%= link_to "Edit profile", edit_user_path(user), class: "button button--primary", data: { turbo_frame: "_top" } %>"#,
                FormatOptions {
                    line_width: 60,
                    ..FormatOptions::default()
                }
            ),
            "<%=\n  link_to(\n    \"Edit profile\",\n    edit_user_path(user),\n    class: \"button button--primary\",\n    data: { turbo_frame: \"_top\" }\n  )\n%>\n"
        );
    }

    #[test]
    fn preserves_long_erb_code_tags_when_arguments_are_not_safely_splittable() {
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
    fn preserves_long_erb_block_opening_tags_when_they_are_control_flow() {
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
    fn wraps_long_erb_code_command_calls() {
        assert_eq!(
            format_with_options(
                r#"<% tag.div class: "card", data: { controller: "profile" }, aria: { label: "Profile" } %>"#,
                FormatOptions {
                    line_width: 48,
                    ..FormatOptions::default()
                }
            ),
            "<%\n  tag.div(\n    class: \"card\",\n    data: { controller: \"profile\" },\n    aria: { label: \"Profile\" }\n  )\n%>\n"
        );
    }

    #[test]
    fn wraps_long_erb_output_command_calls_with_do_blocks() {
        assert_eq!(
            format_with_options(
                r#"<%= form_with model: user, url: user_path(user), data: { turbo_frame: "profile" } do |form| %><div></div><% end %>"#,
                FormatOptions {
                    line_width: 60,
                    ..FormatOptions::default()
                }
            ),
            "<%=\n  form_with(\n    model: user,\n    url: user_path(user),\n    data: { turbo_frame: \"profile\" }\n  ) do |form|\n%>\n  <div></div>\n<% end %>\n"
        );
    }

    #[test]
    fn wraps_long_parenthesized_rails_helper_calls() {
        let options = FormatOptions {
            line_width: 60,
            ..FormatOptions::default()
        };

        assert_eq!(
            format_with_options(
                r#"<%= image_tag("user-placeholder.png", alt: "User profile image", class: "avatar avatar--large") %>"#,
                options
            ),
            "<%=\n  image_tag(\n    \"user-placeholder.png\",\n    alt: \"User profile image\",\n    class: \"avatar avatar--large\"\n  )\n%>\n"
        );

        assert_eq!(
            format_with_options(
                r#"<%= video_tag(["intro.mp4", "intro.webm"], controls: true, autoplay: false, class: "hero-video") %>"#,
                options
            ),
            "<%=\n  video_tag(\n    [\"intro.mp4\", \"intro.webm\"],\n    controls: true,\n    autoplay: false,\n    class: \"hero-video\"\n  )\n%>\n"
        );

        let formatted = format_with_options(
            r#"<%= form_with(model: user, url: user_path(user), data: { turbo_frame: "profile" }) do |form| %><div></div><% end %>"#,
            options,
        );
        let expected = "<%=\n  form_with(\n    model: user,\n    url: user_path(user),\n    data: { turbo_frame: \"profile\" }\n  ) do |form|\n%>\n  <div></div>\n<% end %>\n";

        assert_eq!(formatted, expected);
        assert_eq!(format_with_options(&formatted, options), expected);
    }

    #[test]
    fn preserves_long_erb_output_when_expression_is_not_safely_splittable() {
        assert_eq!(
            format_with_options(
                r#"<%= current_user.admin? ? link_to("Admin", admin_path) : link_to("Home", root_path) %>"#,
                FormatOptions {
                    line_width: 48,
                    ..FormatOptions::default()
                }
            ),
            "<%=\n  current_user.admin? ? link_to(\"Admin\", admin_path) : link_to(\"Home\", root_path)\n%>\n"
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

    fn formatter_ignore_fixture() -> &'static str {
        include_str!("../samples/formatter-ignore-next.html.erb")
    }
}
