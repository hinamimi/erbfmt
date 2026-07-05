use crate::lexer::{ErbTag, ErbTagOpen, ErbTagSyntax};
use crate::mixed_parser::Node;

use super::erb::formatted_erb_code_lines;
use super::inline::FormattingNode;
use super::ruby_wrap;

#[derive(Default)]
pub(super) struct HtmlElementBoundaries {
    pub(super) open_child_same_line: bool,
    pub(super) child_close_same_line: bool,
}

impl HtmlElementBoundaries {
    pub(super) fn preserve_inline_children(&self) -> bool {
        self.open_child_same_line && self.child_close_same_line
    }
}

pub(super) fn has_inline_boundary_content(node: &Node) -> bool {
    match node.unspanned() {
        Node::HtmlText(text) => !text.trim().is_empty(),
        _ => true,
    }
}

pub(super) fn has_multiline_unfoldable_erb_output_child(children: &[Node]) -> bool {
    children.iter().any(|child| {
        matches!(
            child.unspanned(),
            Node::ErbOutput(tag) if is_multiline_unfoldable_erb_output_tag(tag)
        )
    })
}

pub(super) fn is_multiline_unfoldable_erb_output_tag(tag: &ErbTag) -> bool {
    tag.code.contains('\n') && ruby_wrap::fold_command_call(&tag.code).is_none()
}

pub(super) fn single_foldable_erb_output_child(children: &[Node]) -> Option<&ErbTag> {
    let mut erb_output = None;

    for child in children {
        match child.unspanned() {
            Node::HtmlText(text) if text.trim().is_empty() => {}
            Node::ErbOutput(tag) if wrapped_erb_output_lines(tag.syntax, &tag.code).is_some() => {
                if erb_output.replace(tag).is_some() {
                    return None;
                }
            }
            _ => return None,
        }
    }

    erb_output
}

pub(super) fn single_foldable_erb_output_inline_child(
    children: &[Node],
) -> Option<(usize, &ErbTag, Vec<String>)> {
    let mut foldable = None;

    for (index, child) in children.iter().enumerate() {
        if let Node::ErbOutput(tag) = child.unspanned() {
            let Some(lines) = wrapped_erb_output_lines(tag.syntax, &tag.code) else {
                continue;
            };

            if foldable.replace((index, tag, lines)).is_some() {
                return None;
            }
        }
    }

    foldable
}

pub(super) fn single_foldable_erb_output_formatting_node<'a>(
    nodes: &'a [FormattingNode<'a>],
) -> Option<(&'a ErbTag, Vec<String>)> {
    let mut foldable = None;

    for node in nodes {
        match node {
            FormattingNode::HtmlText { text, .. } if text.trim().is_empty() => {}
            FormattingNode::Node(node) => match node.unspanned() {
                Node::HtmlText(text) if text.trim().is_empty() => {}
                Node::ErbOutput(tag) => {
                    let lines = wrapped_erb_output_lines(tag.syntax, &tag.code)?;

                    if foldable.replace((tag, lines)).is_some() {
                        return None;
                    }
                }
                _ => return None,
            },
            _ => return None,
        }
    }

    foldable
}

pub(super) fn wrapped_erb_output_lines(syntax: ErbTagSyntax, code: &str) -> Option<Vec<String>> {
    if !is_erb_output_syntax(syntax) {
        return None;
    }

    let lines = formatted_erb_code_lines(code.trim());
    (lines.len() > 1).then_some(lines)
}

pub(super) fn is_erb_output_syntax(syntax: ErbTagSyntax) -> bool {
    matches!(syntax.open, ErbTagOpen::Output | ErbTagOpen::RawOutput)
}

pub(super) fn is_line_ending_char(ch: char) -> bool {
    matches!(ch, '\n' | '\r')
}

pub(super) fn contains_heredoc_marker(code: &str) -> bool {
    code.contains("<<")
}
