use std::collections::HashMap;

use crate::mixed_parser::{Node, SourceRange};

use super::erb::{format_erb_comment, format_erb_tag_inline};
use super::preserve::is_format_sensitive_html_element;
use super::tag::{normalize_close_tag, normalize_tag};

#[derive(Clone, Copy)]
pub(super) enum FormattingNode<'a> {
    Node(&'a Node),
    HtmlText {
        text: &'a str,
        range: Option<SourceRange>,
    },
}

pub(super) fn can_render_inline(nodes: &[Node]) -> bool {
    nodes.iter().all(is_inline_node)
}

pub(super) fn split_formatting_nodes<'a>(
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

pub(super) fn formatting_node_source_range(node: FormattingNode<'_>) -> Option<SourceRange> {
    match node {
        FormattingNode::Node(node) => node.source_range(),
        FormattingNode::HtmlText { range, .. } => range,
    }
}

pub(super) fn is_inline_formatting_node(node: FormattingNode<'_>) -> bool {
    match node {
        FormattingNode::Node(node) => is_inline_node(node),
        FormattingNode::HtmlText { text, .. } => !text.contains('\n'),
    }
}

pub(super) fn leading_inline_boundary_nodes(nodes: &[FormattingNode<'_>]) -> usize {
    nodes
        .iter()
        .take_while(|node| is_inline_boundary_node(**node))
        .count()
}

pub(super) fn trailing_inline_boundary_nodes(
    nodes: &[FormattingNode<'_>],
    prefix_count: usize,
) -> usize {
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
            close,
            children,
            ..
        } => {
            !close.is_empty()
                && !is_format_sensitive_html_element(name, open)
                && can_render_inline(children)
        }
        Node::HtmlSelfClosing { .. }
        | Node::HtmlVoid { .. }
        | Node::HtmlComment(_)
        | Node::ErbComment(_) => true,
        Node::ErbCode(tag) | Node::ErbOutput(tag) => is_single_line_erb_code(&tag.code),
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
            close,
            children,
            ..
        } => {
            !close.is_empty()
                && !is_format_sensitive_html_element(name, open)
                && is_inline_boundary_html_tag(name)
                && children.iter().all(is_inline_boundary_html_node)
        }
        Node::HtmlVoid { name, .. } | Node::HtmlSelfClosing { name, .. } => {
            is_inline_boundary_html_tag(name)
        }
        Node::HtmlComment(_) | Node::ErbComment(_) => true,
        Node::ErbCode(tag) | Node::ErbOutput(tag) => is_single_line_erb_code(&tag.code),
        Node::HtmlDoctype(_) | Node::Spanned { .. } | Node::ErbBlock { .. } => false,
    }
}

fn is_single_line_erb_code(code: &str) -> bool {
    !code.contains('\n') && !code.contains('\r')
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

pub(super) fn render_inline_formatting_nodes_untrimmed(nodes: &[FormattingNode<'_>]) -> String {
    nodes
        .iter()
        .map(|node| match node {
            FormattingNode::Node(node) => render_inline_node(node),
            FormattingNode::HtmlText { text, .. } => (*text).to_string(),
        })
        .collect()
}

pub(super) fn render_inline_nodes_untrimmed(nodes: &[Node]) -> String {
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
        } => {
            let open = normalize_tag(open).unwrap_or_else(|| open.to_string());
            let close = normalize_close_tag(close).unwrap_or_else(|| close.to_string());

            format!("{open}{}{close}", render_inline_nodes_untrimmed(children))
        }
        Node::HtmlSelfClosing { raw, .. } | Node::HtmlVoid { raw, .. } => raw.clone(),
        Node::HtmlComment(comment) => comment.clone(),
        Node::ErbCode(tag) => format_erb_tag_inline(tag.syntax, tag.code.trim()),
        Node::ErbComment(comment) => format_erb_comment(comment),
        Node::ErbOutput(tag) => format_erb_tag_inline(tag.syntax, tag.code.trim()),
        Node::HtmlDoctype(_) | Node::Spanned { .. } | Node::ErbBlock { .. } => {
            unreachable!("node cannot render inline")
        }
    }
}
