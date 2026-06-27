use crate::{
    lexer::ErbTag,
    mixed_parser::{ErbBranch, Node, SourceRange},
};

use super::engine::Formatter;
use super::erb::format_erb_tag_inline;
use super::inline::{
    FormattingNode, is_inline_formatting_node, render_inline_formatting_nodes_untrimmed,
};
use super::preserve::render_preserved_node;

pub(super) struct ErbBlockParts<'a> {
    pub(super) tag: &'a ErbTag,
    pub(super) end_tag: &'a ErbTag,
    pub(super) children: &'a [Node],
    pub(super) branches: &'a [ErbBranch],
    pub(super) range: Option<SourceRange>,
}

impl<'a> Formatter<'a> {
    pub(super) fn write_erb_block_with_inline_boundaries(
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
            tag,
            end_tag,
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
                tag,
                end_tag,
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

    pub(super) fn write_erb_block(
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

        if prefix.is_empty() {
            self.write_erb_tag(depth, parts.tag);
        } else {
            self.write_indented_line(
                depth,
                &format!(
                    "{prefix}{}",
                    format_erb_tag_inline(parts.tag.syntax, &parts.tag.code)
                ),
            );
        }

        self.format_nodes(parts.children, depth + 1);
        self.format_erb_branches(parts.branches, depth);
        self.write_indented_line(
            depth,
            &format!(
                "{}{suffix}",
                format_erb_tag_inline(parts.end_tag.syntax, &parts.end_tag.code)
            ),
        );
    }

    fn format_erb_branches(&mut self, branches: &[ErbBranch], depth: usize) {
        for branch in branches {
            self.write_erb_tag(depth, &branch.tag);
            self.format_nodes(&branch.children, depth + 1);
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
