use std::collections::HashMap;

use crate::ignore::{IgnoreSelector, parse_ignore_directive};
use crate::syntax::template::{Node, SourceRange};

pub(super) fn formatter_ignore_ranges(
    nodes: &[Node],
    source: &str,
) -> HashMap<SourceRange, SourceRange> {
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
            && let Some(target) = nodes[index + 1..].iter().find(|candidate| {
                !is_whitespace_node(candidate) && !is_ignore_comment(candidate, source)
            })
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

fn is_ignore_comment(node: &Node, source: &str) -> bool {
    is_comment_node(node)
        && node
            .source_range()
            .and_then(|range| source.get(range.start..range.end))
            .is_some_and(|raw| parse_ignore_directive(raw).is_some())
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
    if !between[..newline].chars().all(is_horizontal_whitespace) {
        return None;
    }
    let remaining = &between[newline + 1..];
    let mut lines = remaining.split('\n').collect::<Vec<_>>();
    let indentation = lines.pop()?;
    if !indentation.chars().all(is_indentation_whitespace)
        || !lines
            .iter()
            .all(|line| parse_ignore_directive(line).is_some())
    {
        return None;
    }
    let target_line_start = target.start - indentation.len();
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
