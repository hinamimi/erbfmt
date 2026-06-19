#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IgnoreSelector {
    Lint { rule: Option<String> },
    Format,
    All,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IgnoreDirective {
    pub selector: IgnoreSelector,
}

pub fn parse_ignore_directive(line: &str) -> Option<IgnoreDirective> {
    let body = html_comment_body(line).or_else(|| erb_comment_body(line))?;
    let body = body.trim();
    let rest = body
        .strip_prefix("erbfmt-ignore-next-line")
        .or_else(|| body.strip_prefix("erbfmt-ignore"))?
        .trim();
    let selector = rest.split(':').next().unwrap_or("").trim();
    let token = selector.split_whitespace().next().unwrap_or("");

    let selector = if token.is_empty() || token == "lint" {
        IgnoreSelector::Lint { rule: None }
    } else if let Some(rule) = token.strip_prefix("lint/") {
        IgnoreSelector::Lint {
            rule: Some(rule.to_string()),
        }
    } else if token == "format" {
        IgnoreSelector::Format
    } else if token == "all" {
        IgnoreSelector::All
    } else if token.starts_with("format/") {
        return None;
    } else {
        IgnoreSelector::Lint {
            rule: Some(token.to_string()),
        }
    };

    Some(IgnoreDirective { selector })
}

fn html_comment_body(line: &str) -> Option<&str> {
    let line = line.trim();
    line.strip_prefix("<!--")?.strip_suffix("-->")
}

fn erb_comment_body(line: &str) -> Option<&str> {
    let line = line.trim();
    line.strip_prefix("<%#")?.strip_suffix("%>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lint_directives() {
        assert_eq!(
            parse_ignore_directive("<!-- erbfmt-ignore lint/noDeprecatedHtmlTag: legacy -->"),
            Some(IgnoreDirective {
                selector: IgnoreSelector::Lint {
                    rule: Some("noDeprecatedHtmlTag".to_string())
                }
            })
        );
    }

    #[test]
    fn parses_formatter_directives_from_html_and_erb_comments() {
        assert_eq!(
            parse_ignore_directive("  <!-- erbfmt-ignore format: legacy -->  "),
            Some(IgnoreDirective {
                selector: IgnoreSelector::Format
            })
        );
        assert_eq!(
            parse_ignore_directive("  <%# erbfmt-ignore-next-line format: generated %>  "),
            Some(IgnoreDirective {
                selector: IgnoreSelector::Format
            })
        );
    }

    #[test]
    fn rejects_unknown_formatter_selectors() {
        assert_eq!(
            parse_ignore_directive("<!-- erbfmt-ignore format/rule: reason -->"),
            None
        );
    }

    #[test]
    fn parses_combined_ignore_directives() {
        assert_eq!(
            parse_ignore_directive("<!-- erbfmt-ignore all: generated -->"),
            Some(IgnoreDirective {
                selector: IgnoreSelector::All
            })
        );
    }
}
