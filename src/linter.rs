use crate::{
    html::{self, HtmlToken},
    ignore::{IgnoreSelector, parse_ignore_directive},
    lexer,
    lexer::SourceLocation,
    mixed_parser,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub location: Option<SourceLocation>,
    pub severity: DiagnosticSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Warning,
    Error,
}

impl Diagnostic {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            location: None,
            severity: DiagnosticSeverity::Error,
        }
    }

    #[cfg(test)]
    fn located(message: impl Into<String>, location: SourceLocation) -> Self {
        Self::located_with_severity(message, location, DiagnosticSeverity::Error)
    }

    fn located_with_severity(
        message: impl Into<String>,
        location: SourceLocation,
        severity: DiagnosticSeverity,
    ) -> Self {
        Self {
            message: message.into(),
            location: Some(location),
            severity,
        }
    }

    pub fn message_with_location(&self) -> String {
        match self.location {
            Some(location) => format!("{} at {}", self.message, location),
            None => self.message.clone(),
        }
    }

    pub fn is_error(&self) -> bool {
        self.severity == DiagnosticSeverity::Error
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LintOptions {
    pub enabled: bool,
    pub rules: LintRules,
    pub rule_severities: LintRuleSeverities,
}

impl Default for LintOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            rules: LintRules::default(),
            rule_severities: LintRuleSeverities::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LintRules {
    pub empty_erb_branch: bool,
    pub empty_erb_code_tag: bool,
    pub empty_erb_control_block: bool,
    pub no_deprecated_html_tag: bool,
    pub no_duplicate_html_attribute: bool,
    pub no_invalid_html_boolean_attribute: bool,
    pub no_invalid_html_nesting: bool,
    pub no_self_closing_html_tag: bool,
    pub unsupported_erb_block_starter: bool,
}

impl Default for LintRules {
    fn default() -> Self {
        Self {
            empty_erb_branch: true,
            empty_erb_code_tag: true,
            empty_erb_control_block: true,
            no_deprecated_html_tag: true,
            no_duplicate_html_attribute: true,
            no_invalid_html_boolean_attribute: true,
            no_invalid_html_nesting: true,
            no_self_closing_html_tag: true,
            unsupported_erb_block_starter: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LintRuleSeverities {
    pub empty_erb_branch: DiagnosticSeverity,
    pub empty_erb_code_tag: DiagnosticSeverity,
    pub empty_erb_control_block: DiagnosticSeverity,
    pub no_deprecated_html_tag: DiagnosticSeverity,
    pub no_duplicate_html_attribute: DiagnosticSeverity,
    pub no_invalid_html_boolean_attribute: DiagnosticSeverity,
    pub no_invalid_html_nesting: DiagnosticSeverity,
    pub no_self_closing_html_tag: DiagnosticSeverity,
    pub unsupported_erb_block_starter: DiagnosticSeverity,
}

impl Default for LintRuleSeverities {
    fn default() -> Self {
        Self {
            empty_erb_branch: DiagnosticSeverity::Error,
            empty_erb_code_tag: DiagnosticSeverity::Error,
            empty_erb_control_block: DiagnosticSeverity::Error,
            no_deprecated_html_tag: DiagnosticSeverity::Error,
            no_duplicate_html_attribute: DiagnosticSeverity::Error,
            no_invalid_html_boolean_attribute: DiagnosticSeverity::Error,
            no_invalid_html_nesting: DiagnosticSeverity::Error,
            no_self_closing_html_tag: DiagnosticSeverity::Error,
            unsupported_erb_block_starter: DiagnosticSeverity::Error,
        }
    }
}

#[allow(dead_code)]
pub fn lint(input: &str) -> Vec<Diagnostic> {
    lint_with_options(input, LintOptions::default())
}

pub fn lint_with_options(input: &str, options: LintOptions) -> Vec<Diagnostic> {
    if !options.enabled {
        return Vec::new();
    }

    let tokens = match lexer::tokenize_with_spans(input) {
        Ok(tokens) => tokens,
        Err(error) => {
            return vec![Diagnostic::new(error.to_string())];
        }
    };

    match mixed_parser::parse_spanned(&tokens) {
        Ok(_) => apply_lint_ignore_directives(input, lint_tokens(input, &tokens, options)),
        Err(error) => vec![Diagnostic::new(error.to_string())],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LintIgnoreDirective {
    target_line: usize,
    rule: Option<String>,
}

fn apply_lint_ignore_directives(input: &str, diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    let directives = lint_ignore_directives(input);

    if directives.is_empty() {
        return diagnostics;
    }

    diagnostics
        .into_iter()
        .filter(|diagnostic| !is_lint_diagnostic_ignored(diagnostic, &directives))
        .collect()
}

fn is_lint_diagnostic_ignored(diagnostic: &Diagnostic, directives: &[LintIgnoreDirective]) -> bool {
    let Some(location) = diagnostic.location else {
        return false;
    };

    directives.iter().any(|directive| {
        directive.target_line == location.line
            && directive.rule.as_deref().is_none_or(|rule| {
                diagnostic_rule_id(&diagnostic.message).is_some_and(|id| id == rule)
            })
    })
}

fn lint_ignore_directives(input: &str) -> Vec<LintIgnoreDirective> {
    input
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let directive = parse_ignore_directive(line)?;
            let rule = match directive.selector {
                IgnoreSelector::Lint { rule } => rule,
                IgnoreSelector::All => None,
                IgnoreSelector::Format => return None,
            };

            Some(LintIgnoreDirective {
                target_line: index + 2,
                rule,
            })
        })
        .collect()
}

fn diagnostic_rule_id(message: &str) -> Option<&'static str> {
    if message.starts_with("empty ERB branch") {
        Some("emptyErbBranch")
    } else if message.starts_with("empty ERB code tag")
        || message.starts_with("empty ERB output tag")
    {
        Some("emptyErbCodeTag")
    } else if message.starts_with("empty ERB control block") {
        Some("emptyErbControlBlock")
    } else if message.starts_with("deprecated HTML tag") {
        Some("noDeprecatedHtmlTag")
    } else if message.starts_with("duplicate HTML attribute") {
        Some("noDuplicateHtmlAttribute")
    } else if message.starts_with("invalid HTML boolean attribute value")
        || message.starts_with("redundant HTML boolean attribute value")
    {
        Some("noInvalidHtmlBooleanAttribute")
    } else if message.starts_with("invalid HTML nesting") {
        Some("noInvalidHtmlNesting")
    } else if message.starts_with("self-closing HTML tag") {
        Some("noSelfClosingHtmlTag")
    } else if message.starts_with("unsupported ERB block starter") {
        Some("unsupportedErbBlockStarter")
    } else {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ErbBlockLintFrame {
    code: String,
    output: bool,
    location: SourceLocation,
    has_meaningful_content: bool,
    active_branch: Option<ErbBranchLintFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ErbBranchLintFrame {
    code: String,
    location: SourceLocation,
    has_meaningful_content: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HtmlElementLintFrame {
    name: String,
}

fn lint_tokens(
    input: &str,
    tokens: &[lexer::SpannedToken],
    options: LintOptions,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut erb_stack: Vec<ErbBlockLintFrame> = Vec::new();
    let mut html_stack: Vec<HtmlElementLintFrame> = Vec::new();

    for spanned in tokens {
        match &spanned.token {
            lexer::Token::Html(fragment) => {
                let html_tokens = html::tokenize_with_spans(fragment);
                lint_html_tokens(
                    input,
                    spanned.span.start,
                    &html_tokens,
                    &mut html_stack,
                    options,
                    &mut diagnostics,
                );

                if html_tokens_have_meaningful_content(&html_tokens) {
                    mark_current_block_meaningful(&mut erb_stack);
                }
            }
            lexer::Token::ErbCode(code) => {
                lint_empty_erb_code_tag(
                    ErbCodeTagKind::Code,
                    code,
                    spanned.span.location,
                    options,
                    &mut diagnostics,
                );
                lint_erb_code(code, spanned.span.location, options, &mut diagnostics);
                if !code.trim().is_empty() {
                    mark_current_block_meaningful(&mut erb_stack);
                }
            }
            lexer::Token::ErbComment(_) => {}
            lexer::Token::ErbOutput(code) => {
                lint_empty_erb_code_tag(
                    ErbCodeTagKind::Output,
                    code,
                    spanned.span.location,
                    options,
                    &mut diagnostics,
                );
                if !code.trim().is_empty() {
                    mark_current_block_meaningful(&mut erb_stack);
                }
            }
            lexer::Token::ErbBlockStart { code, output, .. } => {
                mark_current_block_meaningful(&mut erb_stack);
                erb_stack.push(ErbBlockLintFrame {
                    code: code.clone(),
                    output: *output,
                    location: spanned.span.location,
                    has_meaningful_content: false,
                    active_branch: None,
                });
            }
            lexer::Token::ErbBranch { code, .. } => {
                if let Some(frame) = erb_stack.last_mut() {
                    finish_active_branch(frame, options, &mut diagnostics);
                    frame.active_branch = Some(ErbBranchLintFrame {
                        code: code.clone(),
                        location: spanned.span.location,
                        has_meaningful_content: false,
                    });
                }
            }
            lexer::Token::ErbBlockEnd(_) => {
                let Some(mut frame) = erb_stack.pop() else {
                    continue;
                };

                finish_active_branch(&mut frame, options, &mut diagnostics);

                if options.rules.empty_erb_control_block && !frame.has_meaningful_content {
                    diagnostics.push(Diagnostic::located_with_severity(
                        format!(
                            "empty ERB control block `{}`",
                            format_erb_block_open(frame.output, &frame.code)
                        ),
                        frame.location,
                        options.rule_severities.empty_erb_control_block,
                    ));
                }
            }
        }
    }

    diagnostics
}

fn mark_current_block_meaningful(stack: &mut [ErbBlockLintFrame]) {
    if let Some(frame) = stack.last_mut() {
        frame.has_meaningful_content = true;

        if let Some(branch) = &mut frame.active_branch {
            branch.has_meaningful_content = true;
        }
    }
}

fn finish_active_branch(
    frame: &mut ErbBlockLintFrame,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(branch) = frame.active_branch.take() else {
        return;
    };

    if options.rules.empty_erb_branch && !branch.has_meaningful_content {
        diagnostics.push(Diagnostic::located_with_severity(
            format!("empty ERB branch `<% {} %>`", branch.code.trim()),
            branch.location,
            options.rule_severities.empty_erb_branch,
        ));
    }
}

fn html_tokens_have_meaningful_content(tokens: &[html::SpannedHtmlToken]) -> bool {
    tokens.iter().any(|spanned| match &spanned.token {
        HtmlToken::Text(text) => !text.trim().is_empty(),
        HtmlToken::Comment(_) => false,
        HtmlToken::OpenTag(_)
        | HtmlToken::CloseTag(_)
        | HtmlToken::SelfClosingTag(_)
        | HtmlToken::VoidTag(_)
        | HtmlToken::Doctype(_) => true,
    })
}

fn lint_html_tokens(
    input: &str,
    fragment_start: usize,
    tokens: &[html::SpannedHtmlToken],
    stack: &mut Vec<HtmlElementLintFrame>,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for spanned in tokens {
        match &spanned.token {
            HtmlToken::OpenTag(tag) => {
                lint_html_content_model(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    stack,
                    options,
                    diagnostics,
                );
                lint_deprecated_html_tag(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                lint_duplicate_html_attributes(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                lint_invalid_html_boolean_attributes(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                stack.push(HtmlElementLintFrame {
                    name: tag.name.clone(),
                });
            }
            HtmlToken::VoidTag(tag) => {
                lint_html_content_model(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    stack,
                    options,
                    diagnostics,
                );
                lint_deprecated_html_tag(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                lint_duplicate_html_attributes(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                lint_invalid_html_boolean_attributes(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
            }
            HtmlToken::SelfClosingTag(tag) => {
                lint_html_content_model(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    stack,
                    options,
                    diagnostics,
                );
                lint_self_closing_html_tag(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                lint_deprecated_html_tag(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                lint_duplicate_html_attributes(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
                lint_invalid_html_boolean_attributes(
                    input,
                    fragment_start,
                    spanned.span.start,
                    tag,
                    options,
                    diagnostics,
                );
            }
            HtmlToken::CloseTag(tag) => close_html_lint_frame(stack, &tag.name),
            HtmlToken::Text(text) => {
                lint_html_text_content_model(
                    input,
                    fragment_start,
                    spanned.span.start,
                    text,
                    stack,
                    options,
                    diagnostics,
                );
            }
            HtmlToken::Comment(_) | HtmlToken::Doctype(_) => {}
        }
    }
}

fn close_html_lint_frame(stack: &mut Vec<HtmlElementLintFrame>, name: &str) {
    let Some(frame) = stack.pop() else {
        return;
    };

    if !frame.name.eq_ignore_ascii_case(name) {
        stack.push(frame);
    }
}

fn lint_html_content_model(
    input: &str,
    fragment_start: usize,
    html_token_start: usize,
    tag: &html::HtmlTag,
    stack: &[HtmlElementLintFrame],
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !options.rules.no_invalid_html_nesting {
        return;
    }

    let Some(parent) = stack.last() else {
        return;
    };

    let parent_name = parent.name.to_ascii_lowercase();
    let child_name = tag.name.to_ascii_lowercase();
    let Some(message) = invalid_html_child_message(&parent_name, &child_name) else {
        return;
    };

    diagnostics.push(Diagnostic::located_with_severity(
        message,
        lexer::source_location(input, fragment_start + html_token_start),
        options.rule_severities.no_invalid_html_nesting,
    ));
}

fn lint_html_text_content_model(
    input: &str,
    fragment_start: usize,
    html_token_start: usize,
    text: &str,
    stack: &[HtmlElementLintFrame],
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !options.rules.no_invalid_html_nesting || text.trim().is_empty() {
        return;
    }

    let Some(parent) = stack.last() else {
        return;
    };

    let parent_name = parent.name.to_ascii_lowercase();

    if !matches!(
        parent_name.as_str(),
        "ul" | "ol" | "menu" | "table" | "thead" | "tbody" | "tfoot" | "tr" | "colgroup"
    ) {
        return;
    }

    diagnostics.push(Diagnostic::located_with_severity(
        format!("invalid HTML nesting: <{parent_name}> cannot have text as a direct child"),
        lexer::source_location(
            input,
            fragment_start + html_token_start + first_non_whitespace_offset(text),
        ),
        options.rule_severities.no_invalid_html_nesting,
    ));
}

fn invalid_html_child_message(parent: &str, child: &str) -> Option<String> {
    if parent == "p" && !is_phrasing_html_tag(child) {
        return Some(format!(
            "invalid HTML nesting: <p> cannot contain <{child}>"
        ));
    }

    if matches!(parent, "ul" | "ol" | "menu") && !matches!(child, "li" | "script" | "template") {
        return Some(format!(
            "invalid HTML nesting: <{parent}> cannot have <{child}> as a direct child"
        ));
    }

    if parent == "table"
        && !matches!(
            child,
            "caption" | "colgroup" | "thead" | "tbody" | "tfoot" | "tr" | "script" | "template"
        )
    {
        return Some(format!(
            "invalid HTML nesting: <table> cannot have <{child}> as a direct child"
        ));
    }

    if matches!(parent, "thead" | "tbody" | "tfoot")
        && !matches!(child, "tr" | "script" | "template")
    {
        return Some(format!(
            "invalid HTML nesting: <{parent}> cannot have <{child}> as a direct child"
        ));
    }

    if parent == "tr" && !matches!(child, "td" | "th" | "script" | "template") {
        return Some(format!(
            "invalid HTML nesting: <tr> cannot have <{child}> as a direct child"
        ));
    }

    if parent == "colgroup" && !matches!(child, "col" | "template") {
        return Some(format!(
            "invalid HTML nesting: <colgroup> cannot have <{child}> as a direct child"
        ));
    }

    None
}

fn is_phrasing_html_tag(name: &str) -> bool {
    matches!(
        name,
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
            | "datalist"
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
            | "link"
            | "map"
            | "mark"
            | "math"
            | "meta"
            | "meter"
            | "noscript"
            | "object"
            | "output"
            | "picture"
            | "progress"
            | "q"
            | "ruby"
            | "s"
            | "samp"
            | "script"
            | "select"
            | "slot"
            | "small"
            | "span"
            | "strong"
            | "sub"
            | "sup"
            | "svg"
            | "template"
            | "textarea"
            | "time"
            | "u"
            | "var"
            | "video"
            | "wbr"
    )
}

fn first_non_whitespace_offset(text: &str) -> usize {
    text.char_indices()
        .find_map(|(index, ch)| (!ch.is_whitespace()).then_some(index))
        .unwrap_or(0)
}

fn lint_self_closing_html_tag(
    input: &str,
    fragment_start: usize,
    html_token_start: usize,
    tag: &html::HtmlTag,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !options.rules.no_self_closing_html_tag {
        return;
    }

    diagnostics.push(Diagnostic::located_with_severity(
        format!("self-closing HTML tag `{}` is not valid HTML5", tag.raw),
        lexer::source_location(input, fragment_start + html_token_start),
        options.rule_severities.no_self_closing_html_tag,
    ));
}

fn lint_deprecated_html_tag(
    input: &str,
    fragment_start: usize,
    html_token_start: usize,
    tag: &html::HtmlTag,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !options.rules.no_deprecated_html_tag || !is_deprecated_html_tag(&tag.name) {
        return;
    }

    diagnostics.push(Diagnostic::located_with_severity(
        format!("deprecated HTML tag `{}`", tag.raw),
        lexer::source_location(input, fragment_start + html_token_start),
        options.rule_severities.no_deprecated_html_tag,
    ));
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HtmlAttribute {
    name: String,
    offset: usize,
    value: Option<HtmlAttributeValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HtmlAttributeValue {
    raw: String,
}

fn lint_duplicate_html_attributes(
    input: &str,
    fragment_start: usize,
    html_token_start: usize,
    tag: &html::HtmlTag,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !options.rules.no_duplicate_html_attribute || tag.raw.contains("<%") {
        return;
    }

    let attributes = html_attributes(tag);
    let mut seen: Vec<HtmlAttribute> = Vec::new();

    for attribute in attributes {
        if seen
            .iter()
            .any(|seen_attribute| seen_attribute.name == attribute.name)
        {
            diagnostics.push(Diagnostic::located_with_severity(
                format!("duplicate HTML attribute `{}`", attribute.name),
                lexer::source_location(input, fragment_start + html_token_start + attribute.offset),
                options.rule_severities.no_duplicate_html_attribute,
            ));
        } else {
            seen.push(attribute);
        }
    }
}

fn lint_invalid_html_boolean_attributes(
    input: &str,
    fragment_start: usize,
    html_token_start: usize,
    tag: &html::HtmlTag,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !options.rules.no_invalid_html_boolean_attribute || tag.raw.contains("<%") {
        return;
    }

    for attribute in html_attributes(tag) {
        if !is_html_boolean_attribute(&attribute.name) {
            continue;
        }

        let Some(value) = attribute.value else {
            continue;
        };

        let message = if value.raw.eq_ignore_ascii_case("false") {
            Some(format!(
                "invalid HTML boolean attribute value `{}=\"{}\"`",
                attribute.name, value.raw
            ))
        } else if value.raw.eq_ignore_ascii_case(&attribute.name) {
            Some(format!(
                "redundant HTML boolean attribute value `{}=\"{}\"`",
                attribute.name, value.raw
            ))
        } else {
            None
        };

        if let Some(message) = message {
            diagnostics.push(Diagnostic::located_with_severity(
                message,
                lexer::source_location(input, fragment_start + html_token_start + attribute.offset),
                options.rule_severities.no_invalid_html_boolean_attribute,
            ));
        }
    }
}

fn html_attributes(tag: &html::HtmlTag) -> Vec<HtmlAttribute> {
    let Some(mut cursor) = tag.raw.find(&tag.name).map(|index| index + tag.name.len()) else {
        return Vec::new();
    };

    let mut attributes = Vec::new();
    let raw = tag.raw.as_str();

    while cursor < raw.len() {
        cursor = skip_html_attribute_spacing(raw, cursor);

        if cursor >= raw.len() || raw[cursor..].starts_with('>') || raw[cursor..].starts_with("/>")
        {
            break;
        }

        let name_start = cursor;
        let Some(name_end) = read_html_attribute_name_end(raw, name_start) else {
            break;
        };

        if name_end == name_start {
            break;
        }

        let mut attribute = HtmlAttribute {
            name: raw[name_start..name_end].to_ascii_lowercase(),
            offset: name_start,
            value: None,
        };

        cursor = skip_html_attribute_spacing(raw, name_end);

        if raw[cursor..].starts_with('=') {
            let (next_cursor, value) = read_html_attribute_value(raw, cursor + '='.len_utf8());
            attribute.value = value;
            cursor = next_cursor;
        }

        attributes.push(attribute);
    }

    attributes
}

fn skip_html_attribute_spacing(raw: &str, mut cursor: usize) -> usize {
    while cursor < raw.len() {
        let ch = raw[cursor..]
            .chars()
            .next()
            .expect("cursor is inside raw tag");

        if !ch.is_whitespace() {
            break;
        }

        cursor += ch.len_utf8();
    }

    cursor
}

fn read_html_attribute_name_end(raw: &str, start: usize) -> Option<usize> {
    let mut cursor = start;

    while cursor < raw.len() {
        let ch = raw[cursor..]
            .chars()
            .next()
            .expect("cursor is inside raw tag");

        if ch.is_whitespace() || matches!(ch, '=' | '>' | '/') {
            break;
        }

        if matches!(ch, '"' | '\'' | '<') {
            return None;
        }

        cursor += ch.len_utf8();
    }

    Some(cursor)
}

fn read_html_attribute_value(raw: &str, cursor: usize) -> (usize, Option<HtmlAttributeValue>) {
    let mut cursor = skip_html_attribute_spacing(raw, cursor);

    let Some(first) = raw[cursor..].chars().next() else {
        return (cursor, None);
    };

    if first == '"' || first == '\'' {
        cursor += first.len_utf8();
        let value_start = cursor;

        while cursor < raw.len() {
            let ch = raw[cursor..]
                .chars()
                .next()
                .expect("cursor is inside raw tag");
            cursor += ch.len_utf8();

            if ch == first {
                let value_end = cursor - ch.len_utf8();
                return (
                    cursor,
                    Some(HtmlAttributeValue {
                        raw: raw[value_start..value_end].to_string(),
                    }),
                );
            }
        }

        return (
            cursor,
            Some(HtmlAttributeValue {
                raw: raw[value_start..cursor].to_string(),
            }),
        );
    }

    let value_start = cursor;

    while cursor < raw.len() {
        let ch = raw[cursor..]
            .chars()
            .next()
            .expect("cursor is inside raw tag");

        if ch.is_whitespace() || ch == '>' {
            break;
        }

        cursor += ch.len_utf8();
    }

    if cursor == value_start {
        (cursor, None)
    } else {
        (
            cursor,
            Some(HtmlAttributeValue {
                raw: raw[value_start..cursor].to_string(),
            }),
        )
    }
}

fn is_html_boolean_attribute(name: &str) -> bool {
    matches!(
        name,
        "allowfullscreen"
            | "async"
            | "autofocus"
            | "autoplay"
            | "checked"
            | "controls"
            | "default"
            | "defer"
            | "disabled"
            | "formnovalidate"
            | "hidden"
            | "inert"
            | "ismap"
            | "itemscope"
            | "loop"
            | "multiple"
            | "muted"
            | "nomodule"
            | "novalidate"
            | "open"
            | "playsinline"
            | "readonly"
            | "required"
            | "reversed"
            | "selected"
    )
}

fn is_deprecated_html_tag(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "acronym"
            | "applet"
            | "basefont"
            | "big"
            | "center"
            | "dir"
            | "font"
            | "frame"
            | "frameset"
            | "isindex"
            | "marquee"
            | "noframes"
            | "strike"
            | "tt"
            | "xmp"
    )
}

fn format_erb_block_open(output: bool, code: &str) -> String {
    if output {
        format!("<%= {} %>", code.trim())
    } else {
        format!("<% {} %>", code.trim())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErbCodeTagKind {
    Code,
    Output,
}

fn lint_empty_erb_code_tag(
    kind: ErbCodeTagKind,
    code: &str,
    location: SourceLocation,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if !options.rules.empty_erb_code_tag || !code.trim().is_empty() {
        return;
    }

    let message = match kind {
        ErbCodeTagKind::Code => "empty ERB code tag `<% %>`",
        ErbCodeTagKind::Output => "empty ERB output tag `<%= %>`",
    };

    diagnostics.push(Diagnostic::located_with_severity(
        message,
        location,
        options.rule_severities.empty_erb_code_tag,
    ));
}

fn lint_erb_code(
    code: &str,
    location: SourceLocation,
    options: LintOptions,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if options.rules.unsupported_erb_block_starter
        && let Some(keyword @ ("while" | "for" | "until")) = first_keyword(code)
    {
        diagnostics.push(Diagnostic::located_with_severity(
            format!("unsupported ERB block starter `{keyword}`"),
            location,
            options.rule_severities.unsupported_erb_block_starter,
        ));
    }
}

fn first_keyword(code: &str) -> Option<&str> {
    code.split_whitespace().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_no_diagnostics_for_valid_template() {
        let diagnostics = lint("<% if user %>\n<p>Hello</p>\n<% end %>\n");

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn reports_unterminated_erb_tag() {
        let diagnostics = lint("<div><% if user");

        assert_eq!(
            diagnostics,
            vec![Diagnostic::new("unterminated ERB tag at line 1, column 6")]
        );
    }

    #[test]
    fn reports_unexpected_block_end() {
        let diagnostics = lint("<% end %>");

        assert_eq!(
            diagnostics,
            vec![Diagnostic::new(
                "unexpected ERB block end `end` at line 1, column 1"
            )]
        );
    }

    #[test]
    fn reports_unclosed_block() {
        let diagnostics = lint("<% if user %>\n<p>Hello</p>\n");

        assert_eq!(
            diagnostics,
            vec![Diagnostic::new(
                "unclosed ERB block `if user` at line 1, column 1"
            )]
        );
    }

    #[test]
    fn reports_unbalanced_html_tags() {
        let diagnostics = lint("<div><span>Hello</div>");

        assert_eq!(
            diagnostics,
            vec![Diagnostic::new(
                "mismatched HTML close tag `</div>`, expected `</span>` at line 1, column 17"
            )]
        );
    }

    #[test]
    fn reports_empty_erb_control_blocks() {
        let diagnostics = lint("<% if show_empty_state %>\n<% end %>\n");

        assert_eq!(
            diagnostics,
            vec![Diagnostic::located(
                "empty ERB control block `<% if show_empty_state %>`",
                SourceLocation { line: 1, column: 1 }
            )]
        );
    }

    #[test]
    fn reports_empty_erb_code_tags() {
        let diagnostics = lint("<p>Before</p>\n  <% %>\n  <%=   %>\n");

        assert_eq!(
            diagnostics,
            vec![
                Diagnostic::located(
                    "empty ERB code tag `<% %>`",
                    SourceLocation { line: 2, column: 3 }
                ),
                Diagnostic::located(
                    "empty ERB output tag `<%= %>`",
                    SourceLocation { line: 3, column: 3 }
                )
            ]
        );
    }

    #[test]
    fn reports_self_closing_html_tags() {
        let diagnostics = lint("<section>\n  <div />\n  <br />\n</section>\n");

        assert_eq!(
            diagnostics,
            vec![
                Diagnostic::located(
                    "self-closing HTML tag `<div />` is not valid HTML5",
                    SourceLocation { line: 2, column: 3 }
                ),
                Diagnostic::located(
                    "self-closing HTML tag `<br />` is not valid HTML5",
                    SourceLocation { line: 3, column: 3 }
                )
            ]
        );
    }

    #[test]
    fn reports_deprecated_html_tags() {
        let diagnostics = lint(
            "<main>\n  <center>Legacy</center>\n  <font color=\"red\">Alert</font>\n</main>\n",
        );

        assert_eq!(
            diagnostics,
            vec![
                Diagnostic::located(
                    "deprecated HTML tag `<center>`",
                    SourceLocation { line: 2, column: 3 }
                ),
                Diagnostic::located(
                    "deprecated HTML tag `<font color=\"red\">`",
                    SourceLocation { line: 3, column: 3 }
                )
            ]
        );
    }

    #[test]
    fn reports_rule_warning_severity() {
        let diagnostics = lint_with_options(
            "<center>Legacy</center>\n",
            LintOptions {
                rule_severities: LintRuleSeverities {
                    no_deprecated_html_tag: DiagnosticSeverity::Warning,
                    ..LintRuleSeverities::default()
                },
                ..LintOptions::default()
            },
        );

        assert_eq!(
            diagnostics,
            vec![Diagnostic::located_with_severity(
                "deprecated HTML tag `<center>`",
                SourceLocation { line: 1, column: 1 },
                DiagnosticSeverity::Warning
            )]
        );
    }

    #[test]
    fn reports_duplicate_html_attributes() {
        let diagnostics = lint(
            "<main>\n  <article class=\"card\" id=\"one\" class=\"wide\" data-user-id=\"1\" DATA-USER-ID=\"2\"></article>\n</main>\n",
        );

        assert_eq!(
            diagnostics,
            vec![
                Diagnostic::located(
                    "duplicate HTML attribute `class`",
                    SourceLocation {
                        line: 2,
                        column: 34
                    }
                ),
                Diagnostic::located(
                    "duplicate HTML attribute `data-user-id`",
                    SourceLocation {
                        line: 2,
                        column: 64
                    }
                )
            ]
        );
    }

    #[test]
    fn does_not_report_duplicate_html_attributes_when_tag_contains_erb() {
        let diagnostics = lint(r#"<div class="card" <%= tag_options %> class="wide"></div>"#);

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn reports_invalid_html_boolean_attribute_values() {
        let diagnostics =
            lint(r#"<button disabled="false" checked="checked" hidden>Save</button>"#);

        assert_eq!(
            diagnostics,
            vec![
                Diagnostic::located(
                    "invalid HTML boolean attribute value `disabled=\"false\"`",
                    SourceLocation { line: 1, column: 9 }
                ),
                Diagnostic::located(
                    "redundant HTML boolean attribute value `checked=\"checked\"`",
                    SourceLocation {
                        line: 1,
                        column: 26
                    }
                )
            ]
        );
    }

    #[test]
    fn does_not_report_html_boolean_attribute_values_when_tag_contains_erb() {
        let diagnostics = lint(r#"<input disabled="<%= disabled? %>" checked="checked">"#);

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn reports_invalid_list_children() {
        let diagnostics = lint(
            "<ul>\n  <div>Bad</div>\n  <% items.each do |item| %>\n    <li><%= item.name %></li>\n  <% end %>\n</ul>\n<ol>\n  Text\n</ol>\n",
        );

        assert_eq!(
            diagnostics,
            vec![
                Diagnostic::located(
                    "invalid HTML nesting: <ul> cannot have <div> as a direct child",
                    SourceLocation { line: 2, column: 3 }
                ),
                Diagnostic::located(
                    "invalid HTML nesting: <ol> cannot have text as a direct child",
                    SourceLocation { line: 8, column: 3 }
                )
            ]
        );
    }

    #[test]
    fn reports_invalid_table_structure() {
        let diagnostics = lint(
            "<table>\n  <div>Bad</div>\n  <thead><td>Bad</td></thead>\n  <tr><div>Bad</div></tr>\n</table>\n",
        );

        assert_eq!(
            diagnostics,
            vec![
                Diagnostic::located(
                    "invalid HTML nesting: <table> cannot have <div> as a direct child",
                    SourceLocation { line: 2, column: 3 }
                ),
                Diagnostic::located(
                    "invalid HTML nesting: <thead> cannot have <td> as a direct child",
                    SourceLocation {
                        line: 3,
                        column: 10
                    }
                ),
                Diagnostic::located(
                    "invalid HTML nesting: <tr> cannot have <div> as a direct child",
                    SourceLocation { line: 4, column: 7 }
                )
            ]
        );
    }

    #[test]
    fn reports_block_html_inside_paragraphs() {
        let diagnostics = lint("<p>\n  <span>OK</span>\n  <div>Bad</div>\n</p>\n");

        assert_eq!(
            diagnostics,
            vec![Diagnostic::located(
                "invalid HTML nesting: <p> cannot contain <div>",
                SourceLocation { line: 3, column: 3 }
            )]
        );
    }

    #[test]
    fn does_not_report_valid_list_and_table_structure() {
        let diagnostics = lint(
            "<ul>\n  <% items.each do |item| %>\n    <li><%= item.name %></li>\n  <% end %>\n</ul>\n<table>\n  <thead><tr><th>Name</th></tr></thead>\n  <tbody><tr><td>A</td></tr></tbody>\n</table>\n<p><span>OK</span><a href=\"#\">Link</a></p>\n",
        );

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn reports_html_rule_locations_after_erb_tags() {
        let diagnostics = lint("<% if user %>\n  <center>Legacy</center>\n<% end %>\n");

        assert_eq!(
            diagnostics,
            vec![Diagnostic::located(
                "deprecated HTML tag `<center>`",
                SourceLocation { line: 2, column: 3 }
            )]
        );
    }

    #[test]
    fn ignores_lint_diagnostics_on_the_next_line() {
        let diagnostics =
            lint("<!-- erbfmt-ignore lint: legacy markup -->\n<center>Legacy</center>\n");

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn ignores_only_the_selected_lint_rule() {
        let diagnostics = lint(
            "<!-- erbfmt-ignore lint/noDeprecatedHtmlTag: legacy markup -->\n<center><div /></center>\n",
        );

        assert_eq!(
            diagnostics,
            vec![Diagnostic::located(
                "self-closing HTML tag `<div />` is not valid HTML5",
                SourceLocation { line: 2, column: 9 }
            )]
        );
    }

    #[test]
    fn ignores_lint_diagnostics_from_erb_comments() {
        let diagnostics =
            lint("<%# erbfmt-ignore lint/emptyErbCodeTag: generated placeholder %>\n<% %>\n");

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn ignores_lint_diagnostics_with_combined_directives() {
        let diagnostics =
            lint("<!-- erbfmt-ignore all: generated markup -->\n<center>Legacy</center>\n");

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn empty_erb_code_tags_do_not_count_as_meaningful_block_content() {
        let diagnostics = lint("<% if show_empty_state %>\n  <% %>\n<% end %>\n");

        assert_eq!(
            diagnostics,
            vec![
                Diagnostic::located(
                    "empty ERB code tag `<% %>`",
                    SourceLocation { line: 2, column: 3 }
                ),
                Diagnostic::located(
                    "empty ERB control block `<% if show_empty_state %>`",
                    SourceLocation { line: 1, column: 1 }
                )
            ]
        );
    }

    #[test]
    fn does_not_report_supported_erb_branches() {
        let diagnostics =
            lint("<% if current_user %>\n<% else %>\n<p>Please sign in</p>\n<% end %>");

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn reports_empty_erb_branches() {
        let diagnostics = lint(
            "<% if current_user %>\n<p>Hello</p>\n<% else %>\n<% end %>\n\
             <% case role %>\n<% when \"admin\" %>\n<% when \"member\" %>\n<p>Member</p>\n<% end %>\n",
        );

        assert_eq!(
            diagnostics,
            vec![
                Diagnostic::located(
                    "empty ERB branch `<% else %>`",
                    SourceLocation { line: 3, column: 1 }
                ),
                Diagnostic::located(
                    "empty ERB branch `<% when \"admin\" %>`",
                    SourceLocation { line: 6, column: 1 }
                )
            ]
        );
    }

    #[test]
    fn empty_erb_code_tags_do_not_count_as_meaningful_branch_content() {
        let diagnostics =
            lint("<% if current_user %>\n<p>Hello</p>\n<% else %>\n  <% %>\n<% end %>\n");

        assert_eq!(
            diagnostics,
            vec![
                Diagnostic::located(
                    "empty ERB code tag `<% %>`",
                    SourceLocation { line: 4, column: 3 }
                ),
                Diagnostic::located(
                    "empty ERB branch `<% else %>`",
                    SourceLocation { line: 3, column: 1 }
                )
            ]
        );
    }

    #[test]
    fn does_not_report_non_empty_erb_branches() {
        let diagnostics = lint(
            "<% if current_user %>\n<p>Hello</p>\n<% elsif guest? %>\n<p>Guest</p>\n<% else %>\n<p>Please sign in</p>\n<% end %>\n\
             <% begin %>\n<% rescue StandardError %>\n<p>Failed</p>\n<% ensure %>\n<% cleanup %>\n<% end %>\n",
        );

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn reports_unsupported_erb_block_starters() {
        let diagnostics = lint("<% while job.running? %>\n<p>Waiting</p>\n");

        assert_eq!(
            diagnostics,
            vec![Diagnostic::located(
                "unsupported ERB block starter `while`",
                SourceLocation { line: 1, column: 1 }
            )]
        );
    }

    #[test]
    fn reports_unsupported_erb_block_starter_keywords() {
        let diagnostics = lint(
            "<% for user in users %>\n<p><%= user.name %></p>\n<% until done? %>\n<p>Waiting</p>\n",
        );

        assert_eq!(
            diagnostics,
            vec![
                Diagnostic::located(
                    "unsupported ERB block starter `for`",
                    SourceLocation { line: 1, column: 1 }
                ),
                Diagnostic::located(
                    "unsupported ERB block starter `until`",
                    SourceLocation { line: 3, column: 1 }
                )
            ]
        );
    }

    #[test]
    fn respects_disabled_linter() {
        let diagnostics = lint_with_options(
            "<% if show_empty_state %>\n<% end %>\n",
            LintOptions {
                enabled: false,
                ..LintOptions::default()
            },
        );

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn respects_disabled_empty_block_rule() {
        let diagnostics = lint_with_options(
            "<% if show_empty_state %>\n<% end %>\n",
            LintOptions {
                rules: LintRules {
                    empty_erb_control_block: false,
                    ..LintRules::default()
                },
                ..LintOptions::default()
            },
        );

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn respects_disabled_empty_erb_branch_rule() {
        let diagnostics = lint_with_options(
            "<% if current_user %>\n<p>Hello</p>\n<% else %>\n<% end %>\n",
            LintOptions {
                rules: LintRules {
                    empty_erb_branch: false,
                    ..LintRules::default()
                },
                ..LintOptions::default()
            },
        );

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn respects_disabled_html_rules() {
        let diagnostics = lint_with_options(
            "<center><div /></center>\n",
            LintOptions {
                rules: LintRules {
                    no_deprecated_html_tag: false,
                    no_self_closing_html_tag: false,
                    ..LintRules::default()
                },
                ..LintOptions::default()
            },
        );

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn respects_disabled_duplicate_html_attribute_rule() {
        let diagnostics = lint_with_options(
            r#"<div class="card" class="wide"></div>"#,
            LintOptions {
                rules: LintRules {
                    no_duplicate_html_attribute: false,
                    ..LintRules::default()
                },
                ..LintOptions::default()
            },
        );

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn respects_disabled_invalid_html_boolean_attribute_rule() {
        let diagnostics = lint_with_options(
            r#"<button disabled="false" checked="checked">Save</button>"#,
            LintOptions {
                rules: LintRules {
                    no_invalid_html_boolean_attribute: false,
                    ..LintRules::default()
                },
                ..LintOptions::default()
            },
        );

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn respects_disabled_invalid_html_nesting_rule() {
        let diagnostics = lint_with_options(
            "<ul><div>Bad</div></ul>\n<p><div>Bad</div></p>\n",
            LintOptions {
                rules: LintRules {
                    no_invalid_html_nesting: false,
                    ..LintRules::default()
                },
                ..LintOptions::default()
            },
        );

        assert_eq!(diagnostics, Vec::new());
    }

    #[test]
    fn respects_disabled_empty_erb_code_tag_rule() {
        let diagnostics = lint_with_options(
            "<% %>\n<%= %>\n",
            LintOptions {
                rules: LintRules {
                    empty_erb_code_tag: false,
                    ..LintRules::default()
                },
                ..LintOptions::default()
            },
        );

        assert_eq!(diagnostics, Vec::new());
    }
}
