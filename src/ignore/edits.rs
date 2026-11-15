use crate::assist::{TextEdit, TextRange};
use crate::syntax::{
    erb,
    template::{self, Node, ParserOptions},
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct IgnoreTarget {
    pub range: TextRange,
    pub line: usize,
    pub format: Option<TextEdit>,
    #[serde(skip)]
    insertion: usize,
    #[serde(skip)]
    indentation: String,
}

impl IgnoreTarget {
    pub fn lint_edit(&self, source: &str, rule: &str) -> TextEdit {
        self.edit(source, &format!("lint/{rule}"))
    }

    fn edit(&self, source: &str, selector: &str) -> TextEdit {
        let eol = if source.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        TextEdit {
            range: TextRange {
                start: self.insertion,
                end: self.insertion,
            },
            replacement: format!(
                "{}<%# erbfmt-ignore {selector}: intentionally preserved %>{eol}",
                self.indentation
            ),
        }
    }
}

/// Only real, standalone template nodes are insertion boundaries. Text inside
/// attributes, Ruby strings, comments, and raw-text elements is never a target.
pub fn ignore_targets(source: &str, options: ParserOptions) -> Vec<IgnoreTarget> {
    let Ok(tokens) = erb::tokenize_with_spans(source) else {
        return Vec::new();
    };
    let Ok(document) = template::parse_spanned_with_options(&tokens, options) else {
        return Vec::new();
    };
    let mut targets = Vec::new();
    collect(&document.children, source, &mut targets);
    targets
}

fn collect(nodes: &[Node], source: &str, targets: &mut Vec<IgnoreTarget>) {
    for node in nodes {
        if matches!(
            node.unspanned(),
            Node::HtmlText(_) | Node::HtmlComment(_) | Node::ErbComment(_) | Node::HtmlDoctype(_)
        ) {
            continue;
        }
        if let Some(range) = node.source_range() {
            let insertion = source[..range.start].rfind('\n').map_or(0, |i| i + 1);
            let indentation = &source[insertion..range.start];
            if indentation.chars().all(|ch| matches!(ch, ' ' | '\t')) {
                let mut target = IgnoreTarget {
                    range: TextRange {
                        start: range.start,
                        end: range.end,
                    },
                    line: erb::source_location(source, range.start).line,
                    format: None,
                    insertion,
                    indentation: indentation.to_string(),
                };
                let end = source[range.end..]
                    .find('\n')
                    .map_or(source.len(), |i| range.end + i);
                if source[range.end..end]
                    .chars()
                    .all(|ch| matches!(ch, ' ' | '\t' | '\r'))
                {
                    target.format = Some(target.edit(source, "format"));
                }
                targets.push(target);
            }
        }
        match node.unspanned() {
            Node::HtmlElement {
                name,
                open,
                children,
                ..
            } => {
                // Comments in these elements can change their literal contents.
                let name = name.to_ascii_lowercase();
                let open = open.to_ascii_lowercase();
                if !matches!(
                    name.as_str(),
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
                ) && !open.contains("contenteditable")
                    && !open.contains("white-space")
                {
                    collect(children, source, targets);
                }
            }
            Node::ErbBlock {
                children, branches, ..
            } => {
                collect(children, source, targets);
                for branch in branches {
                    collect(&branch.children, source, targets);
                }
            }
            _ => {}
        }
    }
}
