use crate::mixed_parser::Node;

use super::erb::{format_erb_comment, format_erb_tag_inline};
use super::tag::{ParsedTag, attribute_name, attribute_value};

fn render_preserved_html_element(open: &str, close: &str, children: &[Node]) -> String {
    format!("{open}{}{close}", render_preserved_nodes(children))
}

pub(super) fn render_preserved_nodes(nodes: &[Node]) -> String {
    nodes.iter().map(render_preserved_node).collect()
}

pub(super) fn render_preserved_node(node: &Node) -> String {
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
        Node::ErbCode(tag) => format_erb_tag_inline(tag.syntax, tag.code.trim()),
        Node::ErbComment(comment) => format_erb_comment(comment),
        Node::ErbOutput(tag) => format_erb_tag_inline(tag.syntax, tag.code.trim()),
        Node::ErbBlock {
            tag,
            end_tag,
            children,
            branches,
            ..
        } => {
            let mut rendered = format_erb_tag_inline(tag.syntax, &tag.code);
            rendered.push_str(&render_preserved_nodes(children));

            for branch in branches {
                rendered.push_str(&format_erb_tag_inline(branch.tag.syntax, &branch.tag.code));
                rendered.push_str(&render_preserved_nodes(&branch.children));
            }

            rendered.push_str(&format_erb_tag_inline(end_tag.syntax, &end_tag.code));
            rendered
        }
        Node::Spanned { .. } => unreachable!("unspanned node cannot remain wrapped"),
    }
}

pub(super) fn is_format_sensitive_html_element(name: &str, open: &str) -> bool {
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
